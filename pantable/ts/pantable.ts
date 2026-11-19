#!/usr/bin/env node
// Code blocks of class `table`, CSV with a YAML header of options, made
// tables: a pandoc filter.
//
// The block holds CSV, after a YAML header of options (between `---` and
// `---` or `...`, starting the block): `caption`, `short-caption`,
// `alignment` (a letter a column among L, R, C, D), `alignment-cells` (a
// word of letters a row), `width` (a number or fraction a column, or D),
// `table-width` (then the columns without a width get one from the length
// of their text), `ms` (rows of the head, of each body's head and rows, of
// the foot), `ns-head` (row head columns of each body), `markdown` (cells,
// captions and attributes read as markdown), `fancy-table` (with markdown:
// the first column marks the head, bodies and foot, with attributes; a
// cell's first line its span and attributes), `format` (csv only). The
// document's metadata under `pantable` gives defaults, a header's keys
// replacing them. `csv-kwargs` (Python's csv.reader's arguments) is not
// supported: a warning, then the CSV read as it is.
//
// The file a block's `include` attribute names is panclude's to include
// (`pandoc -L panclude.lua`, or libpandoc's `panclude` step, run before this
// filter): a block still carrying the attribute is left as it is.
//
// The header is read by pandoc, as a metadata block (`-f markdown-smart`;
// panir's `yamlFilter`), the markdown of the cells too: in libpandoc's
// JavaScript host, the instance the filter runs in (`hosted`, for panir's
// `libpandocFilter`), in one call; run as a JSON filter, the `pandoc`
// command, a process a text. The CSV is csv-parse's.
//
// A port of ickc/pantable's `pantable` filter, 0.15.0 with the include
// attribute (© 2016-2026 Kolen Cheung; BSD-3-Clause, see ../LICENSE).

import { execFileSync } from "node:child_process";
import { pathToFileURL } from "node:url";
import { parse as parseCSV } from "csv-parse/sync";
import {
  Attr, blocksOf, Caption, Cell, ColSpec, ColWidth, ColWidthDefault, fromJSON, inlinesOf, luaNumber, Options, Para,
  parse, Plain, Row, Str, Table, TableBody, TableFoot, TableHead, toData, yamlFilter,
  type Alignment, type Block, type CodeBlock, type Conversion, type DataValue, type Filter, type Inline,
  type MetaValue, type Pandoc,
} from "panir";
import { runFilter } from "panir/node";

const warn = (message: string) => process.stderr.write(`pantable: ${message}\n`);

/** What reads texts into documents, a document each, in the conversion the
 *  filter runs in, as the format says. */
type Reader = (texts: string[], conversion: Conversion, from: string) => Pandoc[];

/** libpandoc's JavaScript host, as much as this filter uses of it. */
interface Host {
  readMany(texts: string[], from: unknown, options?: Record<string, unknown>): unknown[];
}

/** The instance the filter runs in: all the texts in one call, their tabs
 *  expanded, as the original's `pandoc` command does (a read with a
 *  conversion keeps them). */
const byHost = (pandoc: Host): Reader => (texts, conversion, from) =>
  pandoc.readMany(texts, conversion, { from, "preserve-tabs": false }).map((doc) => {
    const error = (doc as { error?: { kind: string; message: string } }).error;
    if (error) throw new Error(`${error.kind}: ${error.message}`);
    return fromJSON(doc) as Pandoc;
  });

/** The `pandoc` command: a process a text. */
const byCommand: Reader = (texts, _, from) =>
  texts.map((text) => parse(execFileSync("pandoc", ["-f", from, "-t", "json"], { input: text }).toString("utf8")));

class EmptyTable extends Error {}

// The options, from the metadata pandoc read: a key as written or with `_`
// for `-` (the hyphenated one if both); a value of another shape than the
// option's is its default, with a warning.

type Shape = "get" | "text" | "boolean" | "list";

/** An option's value as `Options.text`, `.boolean` or `.list` reads it,
 *  `.get` raw; undefined if missing or of another shape. */
function get<S extends Shape>(options: Options, key: string, shape: S): ReturnType<Options[S]> {
  const k = options.get(key) !== undefined ? key : key.replaceAll("-", "_");
  const raw = options.get(k);
  if (raw === undefined) return undefined as ReturnType<Options[S]>;
  const value = (options[shape] as (key: string) => ReturnType<Options[S]>).call(options, k);
  if (value === undefined) warn(`option ${k} with value ${JSON.stringify(toData(raw))} has an invalid type: the default`);
  return value;
}

/** A number as Lua reads one, or two with `/` between (`1/3`); null for
 *  anything else. A zero denominator fails, as the original. */
function fraction(text: DataValue | undefined): number | null {
  if (typeof text !== "string") return null;
  const slash = text.indexOf("/");
  const a = luaNumber(slash < 0 ? text : text.slice(0, slash));
  const b = slash < 0 ? 1 : luaNumber(text.slice(slash + 1));
  if (a === undefined || b === undefined) return null;
  if (b === 0) throw new Error(`division by zero: ${text}`);
  return a / b;
}

/** Whole numbers (`ms`, `ns-head`), else the default. */
function integers(values: DataValue[] | undefined, key: string): number[] | null {
  if (values === undefined) return null;
  const out = values.map((x) => (typeof x === "string" ? luaNumber(x) : undefined));
  if (out.every((x) => x !== undefined && Number.isInteger(x))) return out as number[];
  warn(`option ${key} with value ${JSON.stringify(values)} has an invalid type: the default`);
  return null;
}

/** Words of text: split at spaces, tabs and line breaks. */
const words = (text: string) => text.match(/[^ \t\n\r]+/g) ?? [];

interface Settings {
  caption: MetaValue | undefined;
  shortCaption: MetaValue | undefined;
  alignment: string;
  alignmentCells: string[];
  width: (number | null)[] | null;
  tableWidth: number | null;
  ms: number[] | null;
  nsHead: number[] | null;
  markdown: boolean;
  fancyTable: boolean;
  format: string;
}

function settingsOf(options: Options): Settings {
  if (options.get("include") !== undefined) {
    warn("the include key is not read: name the file with the include attribute, and run panclude first");
  }
  if (get(options, "csv-kwargs", "get") !== undefined) {
    warn("csv-kwargs (Python's csv.reader's) is not supported in this port: read as CSV");
  }
  return {
    caption: get(options, "caption", "get"),
    shortCaption: get(options, "short-caption", "get"),
    alignment: get(options, "alignment", "text") ?? "",
    alignmentCells: words(get(options, "alignment-cells", "text") ?? ""),
    width: get(options, "width", "list")?.map(fraction) ?? null,
    tableWidth: fraction(get(options, "table-width", "text")),
    ms: integers(get(options, "ms", "list"), "ms"),
    nsHead: integers(get(options, "ns-head", "list"), "ns-head"),
    markdown: get(options, "markdown", "boolean") ?? false,
    fancyTable: get(options, "fancy-table", "boolean") ?? false,
    format: get(options, "format", "text") || "csv",
  };
}

/** What is wrong beyond the type, for a table of m rows and n columns: the
 *  widths, null for the default. */
function normalize(settings: Settings, m: number, n: number): (number | null)[] {
  let total = 0;
  const widths: (number | null)[] = Array(n).fill(null);
  settings.width?.slice(0, n).forEach((w, i) => {
    if (w !== null && w >= 0) {
      widths[i] = w;
      total += w;
    }
  });
  if (settings.tableWidth !== null && settings.tableWidth < total) {
    warn(`table-width smaller than the sum of the widths, ${total}: dropped`);
    settings.tableWidth = null;
  }
  const ms = settings.ms;
  if (ms && (ms.length < 4 || ms.length % 2 || ms.some((x) => x < 0) || ms.reduce((a, b) => a + b, 0) !== m)) {
    warn(`ms ${ms}: too short, of odd length, negative or not adding up to the rows, ${m}: dropped`);
    settings.ms = null;
  }
  const bodies = settings.ms ? settings.ms.length / 2 - 1 : 1;
  const ns = settings.nsHead;
  if (ns && (ns.length !== bodies || ns.some((x) => x > n))) {
    warn(`ns-head ${ns}: not one a body, ${bodies}, or more than the columns, ${n}: dropped`);
    settings.nsHead = null;
  }
  return widths;
}

const ALIGN: Alignment[] = ["AlignDefault", "AlignLeft", "AlignRight", "AlignCenter"];
const mod = (a: number, b: number) => ((a % b) + b) % b;

/** A row of alignments, by the original's arithmetic on each letter's code
 *  (L, R, C, D, and anything else somewhere); all default if one isn't
 *  ASCII. */
function aligns(text: string, n: number): Alignment[] {
  const letters = text.trim().toUpperCase();
  if (/[^\x00-\x7f]/.test(letters)) {
    warn(`non-ASCII character in ${letters}: the default`);
    return Array(n).fill(ALIGN[0]);
  }
  const out = [...letters.slice(0, n)].map((c) => ALIGN[mod(mod(mod(c.charCodeAt(0) - 3, 11), 6), 4)]!);
  return [...out, ...Array(n - out.length).fill(ALIGN[0])];
}

/** Python's str.splitlines. */
function lines(s: string): string[] {
  const out = s.split(/\r\n|[\n\r\v\f\x1c-\x1e\x85\u2028\u2029]/);
  if (out.at(-1) === "") out.pop();
  return out;
}

// The data

/** The rows, as long as the longest. */
function loadCSV(data: string): string[][] {
  // (csv-parse's defaults, but for rows of other lengths, which pantable pads)
  const rows = parseCSV(data, { relax_column_count: true }) as string[][];
  if (!rows.some((row) => row.some((cell) => cell.trim()))) throw new EmptyTable();
  const n = Math.max(...rows.map((row) => row.length));
  return rows.map((row) => [...row, ...Array<string>(n - row.length).fill("")]);
}

/** A table's texts, cells spanning others: each covered cell holds its
 *  span's text and anchor, as the original's array does (a later span
 *  overwrites an earlier one's). */
class Grid {
  texts: (string | null)[][];
  spans = new Map<string, [number, number, number, number]>(); // (rows, columns, anchor)
  constructor(texts: (string | null)[][]) {
    this.texts = texts;
  }

  put(text: string, rows: number, cols: number, i: number, j: number) {
    for (let a = i; a < i + rows; a++) {
      for (let b = j; b < j + cols; b++) {
        this.texts[a]![b] = text;
        if (rows !== 1 || cols !== 1) this.spans.set(`${a},${b}`, [rows, cols, i, j]);
      }
    }
  }

  shape(i: number, j: number): [number, number] {
    const [r, c] = this.spans.get(`${i},${j}`) ?? [1, 1];
    return [r, c];
  }

  isAt(i: number, j: number): boolean {
    const [r, c, a, b] = this.spans.get(`${i},${j}`) ?? [1, 1, i, j];
    return (r === 1 && c === 1) || (a === i && b === j);
  }
}

const CELL_LINE = /^(\([0-9, ]+\))?(\{[^{}]*\})?$/;
const FANCY = /^(\{[^{}]*\})?? ?(---|===|___)? ?(\{[^{}]*\})?$/;

interface Markdown {
  grid: Grid;
  ms: number[] | null;
  blockAttrs: string[] | null;
  rowAttrs: string[];
  cellAttrs: string[][];
}

/** The cells of a table of markdown: a cell's first line may be its span,
 *  `(rows, columns)`, and attributes, `{...}`; with `fancy`, the first
 *  column marks the ends of the head (===), bodies' heads (---), bodies
 *  (___) and the foot (===), with attributes. Attributes as markdown,
 *  `[]{...}`, or "". */
function markdownCells(rows: string[][], fancy: boolean): Markdown {
  const off = Number(fancy);
  const m = rows.length, n = rows[0]!.length - off;
  const grid = new Grid(rows.map(() => Array<string | null>(n).fill(null)));
  const cellAttrs = rows.map(() => Array<string>(n).fill(""));
  for (let i = 0; i < m; i++) {
    for (let j = 0; j < n; j++) {
      if (grid.texts[i]![j] !== null) continue; // (in a span)
      const string = rows[i]![j + off]!;
      const ls = lines(string);
      let attr = "", shape: [number, number] = [1, 1], content = string;
      const found = ls.length > 0 ? CELL_LINE.exec(ls[0]!) : null;
      if (found) {
        const [, span = "", braces = ""] = found;
        const parts = span ? span.slice(1, -1).split(",").map((x) => x.trim()) : ["1", "1"];
        if (parts.some((x) => !/^\d+$/.test(x))) {
          warn(`invalid cell shape ${span}: ignored`);
        } else {
          const size = parts.map(Number);
          if (size.length !== 2 || size[0]! <= 0 || size[1]! <= 0) warn(`invalid cell shape (${size}): ignored`);
          else if (size[0]! + i > m || size[1]! + j > n) warn(`a cell overflowing the table: its attributes ignored: ${string}`);
          else [attr, shape, content] = [braces ? `[]${braces}` : "", [size[0]!, size[1]!], ls.slice(1).join("\n")];
        }
      }
      cellAttrs[i]![j] = attr;
      grid.put(content, ...shape, i, j);
    }
  }
  const rowAttrs = Array<string>(m).fill("");
  let ms: number[] | null = null, blockAttrs: string[] | null = null;
  if (fancy) {
    const markers: [number, string, string][] = []; // (row, marker, the block's attributes)
    rows.forEach((row, i) => {
      if (!row[0]!.trim()) return;
      const found = FANCY.exec(row[0]!);
      if (!found) {
        warn(`can't read the fancy-table cell ${row[0]}: ignored`);
        return;
      }
      if (found[2]) markers.push([i, found[2], found[1] ?? ""]);
      if (found[3]) rowAttrs[i] = `[]${found[3]}`;
    });
    if (markers.length > 0) [ms, blockAttrs] = fancyBlocks(markers, rows);
  }
  return { grid, ms, blockAttrs, rowAttrs, cellAttrs };
}

/** `ms` and the blocks' attributes from the markers. */
function fancyBlocks(markers: [number, string, string][], rows: string[][]): [number[], string[]] {
  const sizes = markers.map(([i], k) => i - (k > 0 ? markers[k - 1]![0] : -1));
  const head = markers[0]![1] === "===";
  const foot = markers.length > 1 && markers.at(-1)![1] === "===";
  const bodies: { head?: [number, string]; body?: [number, string] }[] = [];
  for (let k = Number(head); k < markers.length - Number(foot); k++) {
    const [i, marker, attr] = markers[k]!;
    if (marker === "___") {
      if (bodies.length > 0 && !bodies.at(-1)!.body) bodies.at(-1)!.body = [sizes[k]!, attr];
      else bodies.push({ body: [sizes[k]!, attr] });
    } else if (marker === "---") {
      bodies.push({ head: [sizes[k]!, attr] });
    } else {
      warn(`a fancy-table row neither head nor foot: ignored: ${rows[i]![0]}`);
    }
  }
  const ms = [head ? sizes[0]! : 0];
  const attrs = [head && markers[0]![2] ? `[]${markers[0]![2]}` : ""];
  for (const body of bodies) {
    let attr = "";
    for (const [size, a] of [body.head ?? [0, ""], body.body ?? [0, ""]] as const) {
      ms.push(size);
      attr = a || attr; // (the body's rows' attributes win)
    }
    attrs.push(attr ? `[]${attr}` : "");
  }
  ms.push(foot ? sizes.at(-1)! : 0);
  attrs.push(foot && markers.at(-1)![2] ? `[]${markers.at(-1)![2]}` : "");
  return [ms, attrs];
}

/** The longest line's characters (code points, as Python counts), and 3
 *  (as pandoc's widths). */
const textWidth = (s: string) => {
  const ls = lines(s);
  return ls.length > 0 ? Math.max(...ls.map((l) => [...l].length)) + 3 : 3;
};

/** The columns without a width get one, by the length of their text, so
 *  that the widths add up to `tableWidth`; a cell spanning columns gives
 *  what its first column lacks to the next. */
function autoWidth(grid: Grid, widths: (number | null)[], tableWidth: number): (number | null)[] {
  const n = widths.length;
  const found: (number | [number, number])[][] = widths.map(() => []);
  grid.texts.forEach((row, i) => {
    for (let j = 0; j < n; j++) {
      if (!grid.isAt(i, j)) continue;
      const w = textWidth(row[j] ?? ""), cols = grid.shape(i, j)[1];
      found[j]!.push(cols > 1 ? [w, cols] : w);
    }
  });
  const lengths = found.map((ws, j) => {
    const ints = ws.filter((w): w is number => typeof w === "number");
    if (ints.length === 0) throw new Error("max() arg is an empty sequence: a column of spanning cells only");
    const longest = Math.max(...ints);
    for (const w of ws) {
      if (Array.isArray(w) && w[0] > longest) found[j + 1]!.push(w[1] > 2 ? [w[0] - longest, w[1] - 1] : w[0] - longest);
    }
    return longest;
  });
  const unset = widths.flatMap((w, j) => (w === null ? [j] : []));
  const total = unset.reduce((a, j) => a + lengths[j]!, 0);
  const out = [...widths];
  if (total > 0) {
    const scale = (tableWidth - widths.reduce<number>((a, w) => a + (w ?? 0), 0)) / total;
    for (const j of unset) out[j] = lengths[j]! * scale;
  } else {
    for (const j of unset) out[j] = 0;
  }
  return out;
}

// The table

/** The attributes of `[]{...}` read: its span's; else none. */
function attrOf(blocks: Block[]): Attr {
  const first = blocks.length > 0 ? ("content" in blocks[0]! ? (blocks[0].content as Inline[])[0] : undefined) : undefined;
  if (first && !Array.isArray(first) && first.t === "Span") return first.attr;
  if (blocks.length > 0) warn("can't read the attributes: none");
  return Attr();
}

/** A cell of one paragraph is plain text. */
const plain = (blocks: Block[]): Block[] =>
  blocks.length === 1 && blocks[0]!.t === "Para" ? [Plain(blocks[0].content)] : blocks;

const TEXT = ["MetaString", "MetaInlines", "MetaBlocks"];

function textOrNone(key: string, value: MetaValue | undefined): MetaValue | undefined {
  if (value === undefined || TEXT.includes(value.t)) return value;
  warn(`option ${key} with value ${JSON.stringify(toData(value))} has an invalid type: the default`);
  return undefined;
}

/** The caption: with markdown, as pandoc read the header (the original
 *  reads its text as markdown); else its text, one `Str`, as the
 *  original's (a plain table's caption is an empty paragraph). Not text:
 *  none, with a warning; a short caption of blocks too. */
function caption(longValue: MetaValue | undefined, shortValue: MetaValue | undefined, markdown: boolean): Caption {
  const long = textOrNone("caption", longValue);
  let short = textOrNone("short-caption", shortValue);
  if (!markdown) {
    const text = (v: MetaValue | undefined) => (v ? (toData(v) as string) : "");
    return Caption([Para([Str(text(long))])], { short: [Str(text(short))] });
  }
  if (short?.t === "MetaBlocks") {
    warn("a short-caption of blocks: none");
    short = undefined;
  }
  const shortInlines = short ? inlinesOf(short) : [];
  const inlines = long && long.t !== "MetaBlocks" ? inlinesOf(long) : [];
  const blocks = long?.t === "MetaBlocks" ? blocksOf(long) : inlines.length > 0 ? [Para(inlines)] : [];
  return Caption(blocks, { short: shortInlines.length > 0 ? shortInlines : null });
}

const table = (read: Reader, conversion: Conversion) => (options: Options, data: string, block: CodeBlock): Block | Block[] | undefined => {
  if (block.attr.attributes.some(([k]) => k === "include")) {
    warn("a code block with an include attribute: left as it is (run panclude first)");
    return undefined;
  }
  const settings = settingsOf(options);
  let rows: string[][];
  try {
    if (settings.format !== "csv") throw new Error(`Unknown format: ${settings.format}`);
    rows = loadCSV(data.replace(/^\n+|\n+$/g, "")); // (blank lines around the data dropped)
  } catch (e) {
    if (e instanceof EmptyTable) {
      warn("table is empty: deleted");
      return [];
    }
    throw e;
  }
  const md = settings.markdown ? markdownCells(rows, settings.fancyTable) : undefined;
  const grid = md?.grid ?? new Grid(rows);
  const n = grid.texts[0]!.length;
  let widths = normalize(settings, grid.texts.length, n);
  const ms = md?.ms ?? settings.ms ?? [1, 0, grid.texts.length - 1, 0];
  // the table has the rows of its blocks: with fancy-table, the rows after
  // the last marker are dropped (as the original, which sizes it by ms)
  const m = ms.reduce((a, b) => a + b, 0);
  grid.texts = grid.texts.slice(0, m);
  const nsHead = settings.nsHead ?? Array<number>(ms.length / 2 - 1).fill(0);
  if (settings.tableWidth !== null) widths = autoWidth(grid, widths, settings.tableWidth);
  const specs = aligns(settings.alignment, n).map((a, j) => ColSpec(a, widths[j] !== null ? ColWidth(widths[j]!) : ColWidthDefault()));
  const cellAligns = grid.texts.map((_, i) => aligns(settings.alignmentCells[i] ?? "", n));

  const anchors = grid.texts.flatMap((row, i) => row.flatMap((_, j) => (grid.isAt(i, j) ? [[i, j] as const] : [])));
  const blocks = ms.length / 2 + 1;
  let contents: Block[][], attrs: Attr[], rowAttrs: Attr[], blockAttrs: Attr[];
  if (md) {
    // every text of the table in one call: cells and their attributes,
    // rows' and blocks' attributes
    const texts = [...anchors.map(([i, j]) => grid.texts[i]![j]!), ...anchors.map(([i, j]) => md.cellAttrs[i]![j]!),
      ...md.rowAttrs.slice(0, m), ...(md.blockAttrs ?? Array<string>(blocks).fill(""))];
    const todo = texts.filter((t) => t);
    const done = todo.length > 0 ? read(todo, conversion, "markdown") : [];
    let k = 0;
    const got = texts.map((t) => (t ? done[k++]!.blocks : []));
    const a = anchors.length;
    contents = got.slice(0, a).map(plain);
    attrs = got.slice(a, 2 * a).map(attrOf);
    rowAttrs = got.slice(2 * a, 2 * a + m).map(attrOf);
    blockAttrs = got.slice(2 * a + m).map(attrOf);
  } else {
    contents = anchors.map(([i, j]) => [Plain([Str(grid.texts[i]![j]!)])]);
    attrs = anchors.map(() => Attr());
    rowAttrs = grid.texts.map(() => Attr());
    blockAttrs = Array.from({ length: blocks }, () => Attr());
  }
  const at = new Map(anchors.map(([i, j], k) => [`${i},${j}`, k]));
  const row = (i: number) => {
    const cells = [];
    for (let j = 0; j < n; j++) {
      const k = at.get(`${i},${j}`);
      if (k === undefined) continue;
      const [rowSpan, colSpan] = grid.shape(i, j);
      cells.push(Cell(contents[k], { alignment: cellAligns[i]![j]!, rowSpan, colSpan, attr: attrs[k]! }));
    }
    return Row(cells, { attr: rowAttrs[i]! });
  };
  // the rows of each block
  const ends = ms.slice(0, -1).map((_, k) => ms.slice(0, k + 1).reduce((a, b) => a + b, 0));
  const starts = [0, ...ends], stops = [...ends, m];
  const parts = starts.map((a, k) => Array.from({ length: Math.max(0, stops[k]! - a) }, (_, x) => row(a + x)));
  const head = TableHead(parts[0], { attr: blockAttrs[0]! });
  const bodies = nsHead.map((ns, k) => {
    // (more row heads than bodies: the original fails indexing the blocks)
    if (2 + 2 * k >= parts.length) throw new Error(`ns-head ${nsHead}: more than the bodies`);
    return TableBody(parts[2 + 2 * k], parts[1 + 2 * k], ns, { attr: blockAttrs[1 + k]! });
  });
  const foot = TableFoot(parts.at(-1), { attr: blockAttrs.at(-1)! });
  const { identifier, classes, attributes } = block.attr;
  return Table(caption(settings.caption, settings.shortCaption, settings.markdown), specs, head, bodies, foot,
    { identifier, classes: classes.filter((c) => c !== "table"), attributes });
};

const filters = (read: Reader): Filter[] => [{
  Pandoc: (doc, ctx) => yamlFilter(doc, "table", "pantable", (texts) => read(texts, ctx.conversion, "markdown-smart"),
    table(read, ctx.conversion)),
}];

/** For libpandoc's JavaScript host, given the instance the filter runs in
 *  (`libpandocFilter(hosted)`): reading with it. */
export const hosted = (pandoc: Host): Filter[] => filters(byHost(pandoc));

// run as a JSON filter (the main module): the pandoc command reads
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await runFilter(filters(byCommand));
