#!/usr/bin/env node
// amsthm's theorem and proof environments for any output format: a pandoc
// filter.
//
// The metadata declares the environments (`amsthm: {plain: [Theorem:
// [Lemma]], definition: [Definition], ...}`, with options); a div of one's
// class is one. In LaTeX (and beamer) they are amsthm's own: `\newtheorem`s
// in `header-includes`, `\begin{...}...\end{...}` around each div, and
// citations of their identifiers `\ref`s. In any other format the filter
// numbers them as amsthm would (counters shared, within sections), sets
// their headings and bodies in their style's fonts, ends proofs with a
// symbol, and makes citations of them links.
//
// A div's note, `Theorem 1 (note)`, is a span with class `info` that starts
// its first paragraph (`[note]{.info} ...`), as pandoc has read it with the
// document; without one, its `info` attribute, as plain text, with one
// warning a document where that holds markup.
//
// A port of ickc/pandoc-amsthm's amsthm.lua (Copyright (c) 2016-2026, Kolen
// Cheung; BSD-3-Clause, see ../LICENSE).

import {
  applyFilter, Div, Emph, getAttribute, given, inlines, inlinesOf, items, LineBreak, Link, luaNumber, Math as MathNode,
  MetaBlocks, MetaList, Para, Plain, RawBlock, RawInline, SmallCaps, Space, Span, Str, stringify, Strong, toRomanNumeral,
  type Block, type Citation, type Cite, type Conversion, type Filter, type Header, type Inline, type Meta, type MetaValue,
} from "panir";
import { runFilter } from "panir/node";

// LaTeX's sectioning units, by depth: parent_counter's values.
const PARENT_COUNTERS = new Map([["part", 0], ["chapter", 1], ["section", 2], ["subsection", 3], ["subsubsection", 4], ["paragraph", 5], ["subparagraph", 6]]);
// Document classes whose level-1 headings pandoc's LaTeX writer makes chapters.
const CHAPTER_CLASSES = new Set(["memoir", "book", "report", "scrreprt", "scrreport", "scrbook", "extreport", "extbook", "tufte-book"]);
type FontName = "bold" | "italic" | "smallcaps";
const FONTS = { bold: Strong, italic: Emph, smallcaps: SmallCaps } as const;
const FONT_TAGS = { bold: "Strong", italic: "Emph", smallcaps: "SmallCaps" } as const;
const FONT_LATEX = { bold: "\\bfseries", italic: "\\itshape", smallcaps: "\\scshape" } as const;
// Keys of the metadata that are options, not styles.
const OPTION_KEYS = new Set(["styles", "name_to_text", "parent_counter", "counter_depth", "counter_ignore_headings", "css", "swapnumbers", "qed_symbol"]);
const LATEX_LIKE = new Set(["latex", "beamer"]);
// Formats whose header-includes are HTML.
const HTML_LIKE = new Set(["html", "html4", "html5", "chunkedhtml", "epub", "epub2", "epub3", "revealjs", "slidy", "slideous", "dzslides", "s5"]);
const CSS = "<style>\n:where(.amsthm-qed) { float: right; }\n</style>";
const BLANK = new Set(["Space", "SoftBreak", "LineBreak"]);

const warn = (message: string) => console.error(`[amsthm] warning: ${message}`);

const PORTING_GUIDE = "https://ickc.github.io/pandoc-amsthm/porting.html";
// An `info` attribute that reads as it did as markdown in v3: ASCII letters
// and digits, Lua's spaces, anything not ASCII and `,.;:!?()/-`; not an
// apostrophe, which markdown made a typographic one.
const PLAIN_TEXT = /^[A-Za-z0-9 \t\n\v\f\r\u0080-\u{10ffff},.;:!?()/-]*$/u;

const UNSAFE = new Set(["Math", "Cite", "Image", "RawInline"]);

/** Whether a note holds math, a citation, an image or raw TeX, where a `]`
 * would end LaTeX's optional argument: the original braces it then. */
function needsBrace(note: Inline[]): boolean {
  let found = false;
  const scan = { Math: () => void (found = true), Cite: () => void (found = true), Image: () => void (found = true),
    RawInline: () => void (found = true) };
  for (const x of note) {
    if (UNSAFE.has(x.t)) return true;
    applyFilter(x, scan);
  }
  return found;
}

// ---------------------------------------------------------------------------
// Fonts: an element of one around each Str, and what that does to nesting

type FontNode = Extract<Inline, { t: "Strong" | "Emph" | "SmallCaps" }>;
const isFont = (font: FontName, x: Inline): x is FontNode => x.t === FONT_TAGS[font];

/** In LaTeX, the font of the text around a reference applies to what `\ref`
 * and an in-text citation print; `\eqref` is upright, which undoes a shape
 * (italic, small caps) but not bold. */
function appliesTo(font: FontName, x: Inline): boolean {
  const shape = font !== "bold";
  if (x.t === "Cite") {
    const mode = x.citations.length > 1 ? "NormalCitation" : x.citations.length === 1 ? x.citations[0]!.mode : undefined;
    return mode === "AuthorInText" || (!shape && mode === "NormalCitation");
  }
  if (x.t === "RawInline") return x.format === "tex" && (/^\\ref\{.*\}$/s.test(x.text) || (!shape && /^\\eqref\{.*\}$/s.test(x.text)));
  return false;
}

/** Joins elements of the font next to each other, or with one Space between. */
function mergeConsecutive(font: FontName, content: Inline[]): void {
  const out: Inline[] = [];
  let run: Inline[] | undefined;
  content.forEach((x, i) => {
    if (isFont(font, x)) run = [...(run ?? []), ...x.content];
    else if (run && x.t === "Space" && i + 1 < content.length && isFont(font, content[i + 1]!)) run.push(Space());
    else {
      if (run) out.push(FONTS[font](run));
      run = undefined;
      out.push(x);
    }
  });
  if (run) out.push(FONTS[font](run));
  if (out.length !== content.length) content.splice(0, content.length, ...out);
}

/** Each Str in the font; an element of the font in it undone for emphasis
 * (`\emph` toggles), flattened for the others (bold in bold is bold); the
 * references the font applies to in it; then neighbouring elements of the
 * font in a block joined. */
function wrapStrs(font: FontName): Filter {
  const wrap = (x: Inline) => FONTS[font]([x]);
  const ref = (x: Inline) => (appliesTo(font, x) ? wrap(x) : undefined);
  const merge = (block: { content: Inline[] }) => mergeConsecutive(font, block.content);
  return {
    Str: wrap,
    [FONT_TAGS[font]]: (x: FontNode) => {
      if (font === "italic") return x.content.map((child) => (isFont(font, child) ? child.content : BLANK.has(child.t) ? [child] : [wrap(child)])).flat();
      x.content = x.content.flatMap((child) => (isFont(font, child) ? child.content : [child]));
      return undefined;
    },
    Cite: ref,
    RawInline: ref,
    Para: merge,
    Plain: merge,
    Header: merge,
  };
}

// ---------------------------------------------------------------------------
// The environments

interface Style {
  headfont: FontName[];
  bodyfont: FontName[];
  headpunct: string;
  headspace: string; // " ", "newline", or a length (LaTeX)
  above: string; // LaTeX only, as below and indent
  below: string;
  indent: string;
}

const style = (headfont: FontName[], bodyfont: FontName[]): Style => ({ headfont, bodyfont, headpunct: ".", headspace: " ", above: "", below: "", indent: "" });
const BUILTIN_STYLES = new Map([["plain", style(["bold"], ["italic"])], ["definition", style(["bold"], [])], ["remark", style(["italic"], [])]]);

class Theorem {
  readonly style: string;
  readonly proof: boolean;
  envName: string;
  text: string;
  numbered = true;
  parentCounter: string | undefined;
  sharedCounter: string | undefined;

  constructor(style: string, name: string, text: string, parentCounter?: string, sharedCounter?: string, proof = false) {
    this.style = style;
    this.proof = proof;
    this.envName = name;
    if (name.endsWith("*")) [this.envName, this.numbered] = [name.slice(0, -1), false];
    this.text = text || this.envName;
    this.parentCounter = parentCounter;
    if (parentCounter !== undefined && !PARENT_COUNTERS.has(parentCounter)) {
      warn(`unsupported parent_counter ${parentCounter}, ignoring`);
      this.parentCounter = undefined;
    }
    this.sharedCounter = this.numbered && this.parentCounter !== undefined ? undefined : sharedCounter;
  }

  latex(): string {
    const [env, text] = [this.envName, this.text];
    if (!this.numbered) return `\\newtheorem*{${env}}{${text}}`;
    if (this.sharedCounter !== undefined) return `\\newtheorem{${env}}[${this.sharedCounter}]{${text}}`;
    return `\\newtheorem{${env}}{${text}}${this.parentCounter !== undefined ? `[${this.parentCounter}]` : ""}`;
  }

  get className(): string {
    return this.envName.replaceAll(" ", "_");
  }

  get counterName(): string {
    return this.sharedCounter ?? this.envName;
  }
}

/** The heading levels in a counter's number, `first` to `depth`; a heading
 * at one of them restarts it. */
interface Counter {
  first: number;
  depth: number;
}

/** The writer's options the original reads (`PANDOC_WRITER_OPTIONS`): the
 * conversion's options (defaults-file keys) where a host running the filter
 * in process knows them, else its `writerOptions`, which libpandoc gives
 * every filter it runs (Lua's values under the same keys, but
 * `top-level-chapter` for `chapter`); plain pandoc gives a JSON filter
 * neither. */
interface Writer {
  numberSections: boolean;
  topLevelDivision: string;
  numberOffset: number[];
  secnumdepth: string | undefined; // (a variable, -V)
}

function writerOf(conversion: Conversion): Writer {
  const o = conversion.options ?? conversion.writerOptions ?? {};
  const depth = (o.variables as Record<string, unknown> | undefined)?.secnumdepth;
  const division = o["top-level-division"];
  return {
    numberSections: o["number-sections"] === true,
    topLevelDivision: typeof division === "string" ? division.replace(/^top-level-/, "") : "default",
    numberOffset: Array.isArray(o["number-offset"]) ? o["number-offset"] : [],
    secnumdepth: depth === undefined || depth === null ? undefined : String(depth),
  };
}

interface Options {
  css: boolean;
  styles: Map<string, Style>;
  userStyles: string[];
  swapnumbers: boolean;
  qedSymbol: string;
  counters: Map<string, Counter>;
  numberedDepth: number;
  partLevel: number | undefined;
  proofName: string | undefined;
  theorems: Theorem[]; // in order
  byClass: Map<string, Theorem>;
  counterDepth: number;
  ignoredHeadings: Set<string>;
  headerCounters: Map<number, number>;
  theoremCounters: Map<string, number>;
  identifiers: Map<string, string>; // (LaTeX: "")
  names: Map<string, string>;
  unnumbered: Set<string>;
  warned: Set<string>;
  warnedInfo: boolean; // (an info attribute with markup, once a document)
}

/** The name a metadata key stands for: a value is markdown, so
 * `Conjecture\*` in a list is `Conjecture*`, a key is as written; a
 * backslash before punctuation is dropped, as markdown drops it. */
const metaKey = (key: string) => key.replace(/\\([!-/:-@[-`{-~])/g, "$1");

function readFont(value: MetaValue | undefined, place: string): FontName[] {
  const font: FontName[] = [];
  for (const v of items(value)) {
    const name = stringify(v);
    if (name in FONTS) font.push(name as FontName);
    else if (name !== "normal") warn(`unknown font ${name} in ${place}; use bold, italic, smallcaps or normal`);
  }
  return font;
}

/** amsthm's styles, and those under `styles` (`\newtheoremstyle`'s
 * arguments; what is left out as for a theorem), with the names of the
 * user's, sorted. */
function readStyles(value: MetaValue | undefined): [Map<string, Style>, string[]] {
  const styles = new Map(BUILTIN_STYLES);
  if (value?.t !== "MetaMap") return [styles, []];
  const kept: string[] = [];
  for (const name of Object.keys(value.content).sort()) {
    const spec = value.content[name]!;
    if (OPTION_KEYS.has(name)) warn(`a style cannot be called ${name}, which is an option of its own; ignoring it`);
    else if (name === "proof") warn("a style cannot be called proof, which lists the proof environments; ignoring it");
    else if (spec.t !== "MetaMap") warn(`style ${name} is not a map of its settings; ignoring it`);
    else {
      const settings = spec.content;
      const get = (key: string, d: string) => (settings[key] !== undefined ? stringify(settings[key]) : d);
      const place = `style ${name}`;
      styles.set(name, {
        headfont: settings.headfont !== undefined ? readFont(settings.headfont, place) : ["bold"],
        bodyfont: readFont(settings.bodyfont, place),
        headpunct: get("headpunct", "."),
        headspace: get("headspace", " "),
        above: get("above", ""),
        below: get("below", ""),
        indent: get("indent", ""),
      });
      kept.push(name);
    }
  }
  return [styles, kept];
}

/** The LaTeX unit that level-1 headings are, as pandoc decides it. */
function topLevelDivision(meta: Meta, writer: Writer): string {
  if (PARENT_COUNTERS.has(writer.topLevelDivision)) return writer.topLevelDivision;
  const documentclass = given(meta.documentclass);
  return documentclass && CHAPTER_CLASSES.has(stringify(documentclass)) ? "chapter" : "section";
}

/** How many heading levels step their LaTeX counters: none without -N, else
 * as deep as secnumdepth. */
function numberedDepth(meta: Meta, writer: Writer, top: string): number {
  if (!writer.numberSections) return 0;
  const value = meta.secnumdepth;
  // (false: as if unset, but not the variable)
  const given_ = given(value);
  const text = value !== undefined ? (given_ ? stringify(given_) : undefined) : writer.secnumdepth;
  return ((text !== undefined ? luaNumber(text) : undefined) ?? 5) - PARENT_COUNTERS.get(top)! + 2;
}

function readOptions(meta: Meta, writer: Writer): Options {
  const config = meta.amsthm;
  // (an empty `amsthm:` is text, not a map: it defines nothing)
  const opt: Meta = config?.t === "MetaMap" ? config.content : {};

  const ntt = opt.name_to_text;
  const nameToText = new Map(ntt?.t === "MetaMap" ? Object.entries(ntt.content).map(([k, v]) => [metaKey(k), stringify(v)]) : []);
  // (`name_to_text:` or `""` names nothing)
  if (ntt !== undefined && ntt.t !== "MetaMap" && stringify(ntt) !== "") {
    warn("name_to_text is not a map from names to their text; ignoring it");
  }
  // one unit for every environment, or a map of them by name
  const pc = opt.parent_counter;
  const parentCounters = new Map(pc?.t === "MetaMap" ? Object.entries(pc.content).map(([k, v]) => [metaKey(k), stringify(v)]) : []);
  const parentCounter = pc !== undefined && pc.t !== "MetaMap" ? stringify(pc) : undefined;
  const parentOf = (name: string) => parentCounters.get(name) ?? parentCounter;

  const theorems: Theorem[] = [];
  const [styles, userStyles] = readStyles(opt.styles);
  for (const s of ["plain", "definition", "remark", ...userStyles.filter((s) => !BUILTIN_STYLES.has(s))]) {
    for (const entry of items(opt[s])) {
      if (entry.t === "MetaMap") {
        // a name, and the names sharing its counter (YAML's keys come sorted,
        // and so are they here)
        for (const key of Object.keys(entry.content).sort()) {
          const name = metaKey(key);
          const owner = new Theorem(s, name, nameToText.get(name) ?? "", parentOf(name));
          theorems.push(owner);
          // a name under it shares its counter; an unnumbered one has none,
          // so each is numbered on its own then
          const value = entry.content[key]!;
          for (const v of value.t === "MetaList" ? value.content : [value]) {
            const shared = stringify(v);
            const t = owner.numbered
              ? new Theorem(s, shared, nameToText.get(shared) ?? "", undefined, name)
              : new Theorem(s, shared, nameToText.get(shared) ?? "", parentOf(shared));
            if (t.numbered && !owner.numbered) {
              warn(`${owner.envName} is unnumbered, so it has no counter for ${t.envName} to share; numbering ${t.envName} on its own`);
            }
            theorems.push(t);
          }
        }
      } else {
        const name = stringify(entry);
        theorems.push(new Theorem(s, name, nameToText.get(name) ?? "", parentOf(name)));
      }
    }
  }
  // (an empty name is none, as for any environment: "Proof", and in LaTeX
  // babel's)
  const proofName = nameToText.get("proof") || undefined;
  theorems.push(new Theorem("proof", "proof*", proofName ?? "Proof", undefined, undefined, true));
  const byClass = new Map(theorems.map((t) => [t.className, t]));

  const top = topLevelDivision(meta, writer);
  // with parts at level 1: a part steps no other counter, and is in a
  // theorem's number only when theorems are numbered within parts
  const partLevel = top === "part" ? 1 : undefined;
  const depth = given(opt.counter_depth);
  const explicit = depth ? luaNumber(stringify(depth)) : undefined;
  const counters = new Map<string, Counter>();
  let counterDepth = explicit ?? 0;
  for (const t of theorems) {
    if (!t.numbered || t.sharedCounter !== undefined) continue;
    const c: Counter = { first: 1, depth: explicit ?? 0 };
    if (explicit === undefined && t.parentCounter) {
      // within the heading level that LaTeX would number within
      c.depth = Math.max(0, PARENT_COUNTERS.get(t.parentCounter)! - PARENT_COUNTERS.get(top)! + 1);
      if (partLevel && t.parentCounter !== "part") c.first = 2;
    }
    counters.set(t.counterName, c);
    counterDepth = Math.max(counterDepth, c.depth);
  }
  for (const name of parentCounters.keys()) {
    const shared = byClass.get(name.replaceAll(" ", "_"))?.sharedCounter;
    if (shared) warn(`${name} shares the counter of ${shared}, so its parent_counter is ${shared}'s; ignoring the one given`);
  }

  const ignoredHeadings = new Set(items(given(opt.counter_ignore_headings)).map((e) => stringify(e)));
  // where pandoc starts numbering sections (--number-offset)
  const headerCounters = new Map<number, number>();
  for (let i = 1; i <= counterDepth; i++) headerCounters.set(i, writer.numberOffset[i - 1] ?? 0);

  // the end-of-proof symbol as TeX math: the text of its math, raw TeX and words
  let qedSymbol = "\\Box";
  const qed = opt.qed_symbol;
  if (qed !== undefined && qed.t !== "MetaInlines" && qed.t !== "MetaString") warn("qed_symbol is not math or text; ignoring it");
  else if (qed !== undefined) {
    const parts: string[] = [];
    const collect = (x: { text: string }) => void parts.push(x.text);
    applyFilter(inlinesOf(qed), { Math: collect, RawInline: collect, Str: collect });
    if (parts.length > 0) qedSymbol = parts.join(" ");
  }

  return {
    css: opt.css === undefined || stringify(opt.css) !== "false",
    styles,
    userStyles,
    swapnumbers: opt.swapnumbers !== undefined && stringify(opt.swapnumbers) === "true",
    qedSymbol,
    counters,
    numberedDepth: numberedDepth(meta, writer, top),
    partLevel,
    proofName,
    theorems,
    byClass,
    counterDepth,
    ignoredHeadings,
    headerCounters,
    theoremCounters: new Map(),
    identifiers: new Map(),
    names: new Map(),
    unnumbered: new Set(),
    warned: new Set(),
    warnedInfo: false,
  };
}

function optionsLatex(o: Options): string {
  const lines: string[] = [];
  if (o.qedSymbol !== "\\Box") lines.push(`\\renewcommand{\\qedsymbol}{\\ensuremath{${o.qedSymbol}}}`);
  // \newtheoremstyle{name}{above}{below}{body font}{indent}{heading font}
  //   {punctuation}{space after the heading}{heading layout}
  // (\normalfont for none: an empty argument is the font around it)
  const fonts = (font: FontName[]) => font.map((f) => FONT_LATEX[f]).join("") || "\\normalfont";
  for (const name of o.userStyles) {
    const s = o.styles.get(name)!;
    const space = s.headspace === "newline" ? "\\newline" : s.headspace;
    lines.push(`\\newtheoremstyle{${name}}{${s.above}}{${s.below}}{${fonts(s.bodyfont)}}{${s.indent}}{${fonts(s.headfont)}}{${s.headpunct}}{${space}}{}`);
  }
  if (o.swapnumbers) lines.push("\\swapnumbers"); // before \newtheorem, which takes the order from it
  // at the start of the document, after babel names the proof
  if (o.proofName !== undefined) lines.push(`\\AtBeginDocument{\\renewcommand{\\proofname}{${o.proofName}}}`);
  let current = "";
  for (const t of o.theorems.filter((t) => !t.proof)) {
    if (t.style !== current) lines.push(`\\theoremstyle{${(current = t.style)}}`);
    lines.push(t.latex());
  }
  return lines.join("\n");
}

/** A block in header-includes, whatever shape it has. */
function addHeaderInclude(meta: Meta, block: Block, atStart = false): void {
  const existing = items(meta["header-includes"]);
  const added = MetaBlocks([block]);
  meta["header-includes"] = MetaList(atStart ? [added, ...existing] : [...existing, added]);
}

// ---------------------------------------------------------------------------
// The two passes

/** The environment a div is, by its classes; none for one naming two. */
function findTheorem(o: Options, classes: readonly string[], warnTwo = false): Theorem | undefined {
  let found: Theorem | undefined;
  const others: string[] = [];
  for (const c of classes) {
    const t = o.byClass.get(c);
    if (t === undefined || t === found) continue;
    if (found === undefined) found = t;
    else if (!others.includes(c)) others.push(c);
  }
  if (others.length > 0) {
    if (warnTwo) warn(`multiple environments found: ${found!.className}, ${others.join(", ")}; skipping`);
    return undefined;
  }
  return found;
}

/** The heading, as amsthm's `\thmhead` writes it: the name and the number in
 * the style's heading font, the number upright, the note in parentheses in
 * the note font, and the punctuation; then the space after the heading. */
function header(o: Options, theorem: Theorem, identifier: string, note: Inline[] | undefined): Inline[] {
  if (theorem.proof) {
    // "Proof.", in italics; or the note, in italics but its references, for
    // the whole heading
    if (!note) return [Emph(inlines(`${theorem.text}.`)), Space()];
    // each Str in italics, emphasis in it upright, references as the font
    // applies to them; then neighbouring italics joined (in the paragraph
    // itself: a walk is of what is in it)
    const para = applyFilter(Para(note), wrapStrs("italic"));
    mergeConsecutive("italic", para.content);
    return [...para.content, Emph([Str(".")]), Space()];
  }
  const s = o.styles.get(theorem.style)!;
  const head = s.headfont;
  let number: string | undefined;
  if (theorem.numbered) {
    const name = theorem.counterName;
    const count = (o.theoremCounters.get(name) ?? 0) + 1;
    o.theoremCounters.set(name, count);
    const c = o.counters.get(name) ?? { first: 1, depth: 0 };
    const parts: string[] = [];
    for (let level = c.first; level <= c.depth; level++) {
      const n = o.headerCounters.get(level)!;
      // \thepart is a Roman numeral, and empty before the first part
      parts.push(level === o.partLevel ? (n > 0 ? toRomanNumeral(n) : "") : String(n));
    }
    number = [...parts, String(count)].join(".");
    if (identifier) {
      o.identifiers.set(identifier, number);
      o.names.set(identifier, theorem.text);
    }
  }

  const runs: [Inline[], FontName[]][] = [[inlines(theorem.text), head]];
  // \swappedhead: the number first, in the heading font, then a tie
  if (number && o.swapnumbers) runs.unshift([[Str(number), Str(" ")], head]);
  else if (number) runs.push([[Space(), Str(number)], head.filter((f) => f === "bold")]); // upright
  if (note) runs.push([[Space(), Str("("), ...note, Str(")")], []]);
  runs.push([[Str(s.headpunct)], head]);
  return [...joinRuns(runs), s.headspace === "newline" ? LineBreak() : Space()];
}

const inFont = (content: Inline[], font: FontName[]): Inline[] => font.reduceRight((inner, f) => [FONTS[f](inner)], content);

/** Runs of inlines, each with its font, each stretch of one font in its
 * elements once ("Theorem 1." one Strong, not three). */
function joinRuns(runs: [Inline[], FontName[]][]): Inline[] {
  const out: Inline[] = [];
  let current: [Inline[], FontName[]] | undefined;
  for (const [run, font] of runs) {
    if (current && current[1].join() === font.join()) current[0].push(...run);
    else {
      if (current) out.push(...inFont(...current));
      current = [[...run], font];
    }
  }
  if (current) out.push(...inFont(...current));
  return out;
}

/** `name`, a no-break space, then `number`: "Theorem 1". */
const named = (name: string, number: Inline): Inline[] => [...inlines(name), Str(" "), number];

/** The div's content in its style's body font; environments in it are held
 * out, as each has a style of its own. */
function styleBody(div: Div, o: Options, bodyfont: FontName[]): void {
  const held: Div[] = [];
  // each placeholder's attribute starts with a mark no document can know
  // (the original's is the address of its list), so a div of the user's with
  // the attribute is not taken for one
  const mark = `${crypto.randomUUID()} `;
  applyFilter(div.content, {
    traverse: "topdown",
    Div: (d, ctx) => {
      if (!findTheorem(o, d.attr.classes)) return undefined;
      held.push(d);
      ctx.skipChildren();
      return Div([], { attributes: { "amsthm-held": `${mark}${held.length}` } });
    },
  });
  for (const font of bodyfont) applyFilter(div.content, wrapStrs(font));
  if (held.length > 0) {
    applyFilter(div.content, {
      Div: (d) => {
        const value = getAttribute(d.attr, "amsthm-held");
        return value?.startsWith(mark) ? held[Number(value.slice(mark.length)) - 1] : undefined;
      },
    });
  }
}

/** amsthm's \mathqed, \quad\qedsymbol (a space between where the symbol
 * starts with a letter). */
const qedMath = (o: Options) => `\\quad${/^[A-Za-z]/.test(o.qedSymbol) ? " " : ""}${o.qedSymbol}`;
const qedSpan = (o: Options) => Span([MathNode("InlineMath", qedMath(o))], { classes: ["amsthm-qed"] });

/** amsthm's \qedhere: the symbol where it is, \mathqed in math, \qed (after
 * the spaces before it) in text; environments in this one keep theirs.
 * Whether there was one. */
function placeQedhere(div: Div, o: Options): boolean {
  let placed = false;
  applyFilter(div.content, {
    traverse: "topdown",
    Div: (d, ctx) => {
      if (findTheorem(o, d.attr.classes)) ctx.skipChildren();
    },
    Math: (m) => {
      const text = m.text.replace(/\\qedhere(?![A-Za-z])/g, () => qedMath(o));
      if (text !== m.text) {
        placed = true;
        m.text = text;
      }
    },
    Inlines: (xs) => {
      const isQedhere = (x: Inline) => x.t === "RawInline" && (x.format === "tex" || x.format === "latex") && /^[ \t\n\v\f\r]*\\qedhere[ \t\n\v\f\r]*$/.test(x.text);
      if (!xs.some(isQedhere)) return undefined;
      const out: Inline[] = [];
      for (const x of xs) {
        if (!isQedhere(x)) out.push(x);
        else {
          while (out.length > 0 && BLANK.has(out.at(-1)!.t)) out.pop(); // (\qed starts with \unskip)
          out.push(qedSpan(o));
          placed = true;
        }
      }
      return out;
    },
  });
  return placed;
}

const withInlines = (block: Block | undefined) => (block?.t === "Para" || block?.t === "Plain" ? block.content : undefined);

/** A string without Lua's spaces at its ends. */
function luaTrim(s: string): string {
  const space = " \t\n\v\f\r";
  let [start, end] = [0, s.length];
  while (start < end && space.includes(s[start]!)) start++;
  while (end > start && space.includes(s[end - 1]!)) end--;
  return s.slice(start, end);
}

/** Takes a div's note out of it: a span with class `info` that starts its
 * first paragraph, as pandoc has read it, with the space after it, and the
 * paragraph if nothing is left in it (the span's other attributes ignored);
 * without one, or with a blank one, its `info` attribute, as plain text, the
 * spaces at its ends dropped. */
function takeNote(div: Div, o: Options): Inline[] | undefined {
  let note: Inline[] | undefined;
  const first = withInlines(div.content[0]);
  const span = first?.[0];
  if (span?.t === "Span" && span.attr.classes.includes("info")) {
    first!.shift();
    if (!span.content.every((x) => BLANK.has(x.t))) note = span.content;
    if (first![0]?.t === "Space" || first![0]?.t === "SoftBreak") first!.shift();
    if (first!.length === 0) div.content.shift();
  }
  const info = luaTrim(getAttribute(div.attr, "info") ?? "");
  if (info) {
    if (note) warn(`a note in both a leading [...]{.info} span and info="${info}"; using the span`);
    else {
      if (!PLAIN_TEXT.test(info) && !o.warnedInfo) {
        o.warnedInfo = true;
        warn(`the info attribute is plain text since amsthm v4, so info="${info}" is shown as typed; `
          + `write the note as a leading [...]{.info} span to keep its markup (once per document; see ${PORTING_GUIDE})`);
      }
      note = inlines(info);
    }
  }
  return note;
}

/** Not LaTeX: the heading, the body's font, the end of a proof. */
function environment(div: Div, o: Options): Div | undefined {
  const theorem = findTheorem(o, div.attr.classes, true);
  if (!theorem) return undefined;
  const note = takeNote(div, o); // (before the body is in its font, which the note is not)
  const identifier = div.attr.identifier;
  if (!theorem.numbered && identifier) o.unnumbered.add(identifier);
  const title = header(o, theorem, identifier, note);
  const space = title.pop()!;
  const head = [Span(title, { classes: ["amsthm-title"] }), space];

  const bodyfont = o.styles.get(theorem.style)?.bodyfont ?? [];
  if (bodyfont.length > 0) styleBody(div, o, bodyfont);
  const first = withInlines(div.content[0]);
  if (first) first.unshift(...head);
  else div.content.unshift(Para(head));

  if (theorem.proof && !placeQedhere(div, o)) {
    // amsthm's \qed, \nobreak\hfill\quad\openbox: as math, \quad\Box (the
    // \hfill is the CSS's)
    const last = withInlines(div.content.at(-1));
    if (last) last.push(qedSpan(o));
    else div.content.push(Para([qedSpan(o)]));
  }
  div.attr.classes.push("amsthm");
  if (!div.attr.classes.includes(`amsthm-${theorem.style}`)) div.attr.classes.push(`amsthm-${theorem.style}`);
  return div;
}

/** The environment a citation's id names, and whether to name it: `@Euler`
 * is `@euler`, the environment's name before the number. */
function referenceTarget(id: string, known: { has(id: string): boolean }): [string, boolean] | undefined {
  if (known.has(id)) return [id, false];
  const lower = id.replace(/^[A-Z]/, (c) => c.toLowerCase());
  return lower !== id && known.has(lower) ? [lower, true] : undefined;
}

/** Whether a citation has a prefix or suffix (`[see @a]`, `[@a, p. 3]`),
 * which a reference to an environment would drop. */
const hasAffix = (c: Citation) => c.prefix.length > 0 || c.suffix.length > 0;

/** The ids of `[@a; @b]`, each an environment's, none with a prefix or
 * suffix (which the list would drop); else undefined, for citeproc. */
function multiRef(cite: Cite, known: Map<string, string>): string[] | undefined {
  if (cite.citations.length < 2) return undefined;
  const ok = cite.citations.every((c) => c.mode === "NormalCitation" && known.has(c.id) && !hasAffix(c));
  return ok ? cite.citations.map((c) => c.id) : undefined;
}

const refList = (ids: string[], one: (id: string) => Inline[]): Inline[] => ids.flatMap((id, i) => [...(i > 0 ? [Str(","), Space()] : []), ...one(id)]);

const single = (cite: Cite) => (cite.citations.length === 1 ? cite.citations[0]! : undefined);

const RAW_REF = /^\\([A-Za-z]+)\{(.*)\}$/s;

/** Once per id, a reference to an unnumbered environment; and a citation of
 * several that is left to citeproc. */
function warnUnnumbered(x: Cite | Extract<Inline, { t: "RawInline" }>, o: Options): void {
  let ids: string[] = [];
  if (x.t === "Cite") ids = x.citations.map((c) => c.id);
  else if (x.format === "tex") {
    const m = RAW_REF.exec(x.text);
    if (m && (m[1] === "ref" || m[1] === "eqref")) ids = [m[2]!];
  }
  for (const id of ids) {
    const target = referenceTarget(id, o.unnumbered);
    if (target && !o.warned.has(target[0])) {
      o.warned.add(target[0]);
      warn(`reference to the unnumbered environment ${target[0]}; LaTeX prints the last number before it, other output leaves it unresolved`);
    }
  }
  if (x.t !== "Cite") return;
  // [see @a], [@a, p. 3]: one environment, with a prefix or suffix
  const c = single(x);
  if (c && hasAffix(c) && (referenceTarget(c.id, o.identifiers) || referenceTarget(c.id, o.unnumbered))) {
    warn(`a reference to an environment cannot carry a prefix or suffix, and is left to citeproc: ${stringify(x)}`);
  }
  if (x.citations.length < 2) return;
  let [ours, others, namedAny, affixed] = [false, false, false, false];
  for (const c of x.citations) {
    affixed ||= hasAffix(c);
    if (o.identifiers.has(c.id) || o.unnumbered.has(c.id)) ours = true;
    else if (referenceTarget(c.id, o.identifiers)) namedAny = true;
    else others = true;
  }
  if (namedAny) warn(`a reference to several environments cannot name them, and is left to citeproc: ${stringify(x)}`);
  else if (ours && others) warn(`a citation mixes environments with other keys, and is left to citeproc: ${stringify(x)}`);
  else if (ours && affixed) warn(`a reference to several environments cannot carry a prefix or suffix, and is left to citeproc: ${stringify(x)}`);
}

const parenthesised = (out: Inline[], paren: boolean) => (paren ? [Str("("), ...out, Str(")")] : out);

/** The number, a link to the environment, perhaps led by its name, in
 * parentheses with `paren`. */
function refLink(id: string, number: string, paren: boolean, name?: string): Inline[] {
  const link = Link([Str(number)], { url: `#${id}` });
  return parenthesised(name ? named(name, link) : [link], paren);
}

/** Not LaTeX: `[@id]`, `@id`, `\ref{id}`, `\eqref{id}` to numbers. */
function resolve(x: Cite | Extract<Inline, { t: "RawInline" }>, o: Options): Inline[] | undefined {
  const known = o.identifiers;
  if (x.t === "Cite") {
    const ids = multiRef(x, known);
    if (ids) return refList(ids, (i) => refLink(i, known.get(i)!, true));
    const c = single(x);
    // (with a prefix or suffix, for citeproc, as [@a; @b] is)
    const target = c && !hasAffix(c) && referenceTarget(c.id, known);
    if (!c || !target) return undefined;
    const [id, isNamed] = target;
    const name = isNamed ? o.names.get(id) : undefined;
    if (c.mode === "NormalCitation" || c.mode === "AuthorInText") return refLink(id, known.get(id)!, c.mode === "NormalCitation", name);
    return undefined;
  }
  const m = x.format === "tex" ? RAW_REF.exec(x.text) : null;
  if (m && (m[1] === "ref" || m[1] === "eqref") && known.has(m[2]!)) return refLink(m[2]!, known.get(m[2]!)!, m[1] === "eqref");
  return undefined;
}

/** LaTeX: a citation of an environment to `\ref` or `\eqref`, perhaps led by
 * its name (`Theorem~\ref{euler}`). */
function citeToRef(cite: Cite, o: Options): Inline[] | undefined {
  const known = o.identifiers;
  const ids = multiRef(cite, known);
  if (ids) return refList(ids, (i) => [RawInline("latex", `\\eqref{${i}}`)]);
  const c = single(cite);
  // (with a prefix or suffix, for citeproc, as [@a; @b] is)
  if (!c || hasAffix(c) || (c.mode !== "NormalCitation" && c.mode !== "AuthorInText")) return undefined;
  const paren = c.mode === "NormalCitation";
  const target = referenceTarget(c.id, known);
  if (!target) return undefined;
  const [id, isNamed] = target;
  const name = isNamed ? o.names.get(id) : undefined;
  if (!name) return [RawInline("latex", `\\${paren ? "eqref" : "ref"}{${id}}`)];
  return parenthesised(named(name, RawInline("latex", `\\ref{${id}}`)), paren);
}

/** LaTeX: the div's own blocks between `\begin{...}` and `\end{...}`, sharing
 * a line with the first and last paragraphs, in a bare div. */
function latexEnvironment(div: Div, o: Options): Div | undefined {
  const theorem = findTheorem(o, div.attr.classes);
  if (!theorem) return undefined;
  const opening: Inline[] = [RawInline("latex", `\\begin{${theorem.envName}}`)];
  const note = takeNote(div, o);
  if (note) {
    const brace = needsBrace(note);
    opening.push(RawInline("latex", brace ? "[{" : "["), ...note, RawInline("latex", brace ? "}]" : "]"));
  }
  if (div.attr.identifier) opening.push(RawInline("latex", `\\label{${div.attr.identifier}}`));
  const closing = RawInline("latex", `\\end{${theorem.envName}}`);
  const content = div.content;
  const first = content[0];
  if (first?.t === "Para" || first?.t === "Plain") first.content.unshift(...opening, RawInline("latex", "\n"));
  else content.unshift(Plain(opening));
  const last = content.at(-1)!;
  if (last.t === "Para" || last.t === "Plain") last.content.push(RawInline("latex", "\n"), closing);
  else content.push(Plain([closing]));
  return Div(content);
}

let options: Options;
let latex = false;

const first: Filter<"topdown"> = {
  traverse: "topdown",
  Pandoc: (doc, ctx) => {
    const format = ctx.format ?? "";
    latex = LATEX_LIKE.has(format);
    options = readOptions(doc.meta, writerOf(ctx.conversion));
    if ([...options.counters.values()].some((c) => c.depth >= c.first && options.numberedDepth < c.first)) {
      warn("theorems are numbered within sections that are not numbered, so their numbers start with 0 as in LaTeX; pass -N (--number-sections)");
    }
    if (latex) {
      // amsthm first, for the user's own \newtheoremstyle; the environments
      // last, to use those styles
      addHeaderInclude(doc.meta, RawBlock("latex", "\\usepackage{amsthm}"), true);
      addHeaderInclude(doc.meta, RawBlock("latex", optionsLatex(options)));
    } else if (HTML_LIKE.has(format) && options.css) addHeaderInclude(doc.meta, RawBlock("html", CSS));
  },
  // \section* and the like step no counter, nor do headings LaTeX doesn't number
  Header: (h: Header) => {
    const o = options;
    if (latex || h.level > o.counterDepth || h.level > o.numberedDepth || h.attr.classes.includes("unnumbered")) return;
    if (o.ignoredHeadings.has(stringify(h))) return;
    o.headerCounters.set(h.level, (o.headerCounters.get(h.level) ?? 0) + 1);
    if (h.level !== o.partLevel) for (let level = h.level + 1; level <= o.counterDepth; level++) o.headerCounters.set(level, 0);
    // restart each counter numbered within this level or above it
    for (const [name, c] of o.counters) if (c.first <= h.level && h.level <= c.depth) o.theoremCounters.delete(name);
  },
  Div: (d) => {
    if (!latex) return environment(d, options);
    // the ids, for the references before their environments
    const theorem = findTheorem(options, d.attr.classes, true);
    if (theorem && d.attr.identifier) {
      options.identifiers.set(d.attr.identifier, "");
      options.names.set(d.attr.identifier, theorem.text);
      if (!theorem.numbered) options.unnumbered.add(d.attr.identifier);
    }
    return undefined;
  },
};

const second: Filter<"topdown"> = {
  traverse: "topdown",
  Div: (d) => (latex ? latexEnvironment(d, options) : undefined),
  Cite: (c) => {
    warnUnnumbered(c, options);
    return latex ? citeToRef(c, options) : resolve(c, options);
  },
  RawInline: (r) => {
    warnUnnumbered(r, options);
    return latex ? undefined : resolve(r, options);
  },
};

await runFilter([first, second]);
