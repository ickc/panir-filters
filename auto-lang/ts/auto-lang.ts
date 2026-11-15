#!/usr/bin/env node
// Spans with `lang` (and `dir`) around runs of a mapped Unicode script: a
// pandoc filter.
//
// The metadata maps scripts to languages (`auto-lang: {Hebrew: he, Han:
// zh-Hant}`); each run of text in a mapped script becomes a span with the
// language, and `dir=rtl` for a right-to-left script. Spaces, digits and
// punctuation join a run that goes on after them; a character that
// Script_Extensions ties to the run's script (`。` to Han) joins a run it
// touches; a combining mark goes with the character before it. A span that
// has a `lang` already is left alone, and what is in it.
//
// A port of ickc/font's bin/auto-lang.lua (Copyright (c) 2026, Kolen Cheung;
// BSD-3-Clause, see ../LICENSE), with its table of Unicode 15.0.0's scripts
// (`script-ranges.ts`, made by ../tables.py from the original's).

import { hasAttribute, Span, Str, stringify, type Filter, type Inline } from "panir";
import { runFilter } from "panir/node";
import { EXT, INHERITED, NAMES, RTL, STRONG, VERSION } from "./script-ranges.ts";

const languages = new Map<string, string>(); // a mapped script's language

// Containers that carry nothing of their own: what is in them decides the
// run they belong to (a span with a `lang` aside). A note's content is a
// flow of its own; code, math and raw inlines are not prose.
const CONTAINERS = new Set(["Emph", "Link", "Quoted", "SmallCaps", "Span", "Strikeout", "Strong", "Subscript", "Superscript", "Underline"]);
type Container = Extract<Inline, { t: "Emph" | "Link" | "Quoted" | "SmallCaps" | "Span" | "Strikeout" | "Strong" | "Subscript" | "Superscript" | "Underline" }>;

const transparent = (x: Inline): x is Container =>
  CONTAINERS.has(x.t) && !(x.t === "Span" && hasAttribute(x.attr, "lang"));

/** The index of the last range starting at or before `cp`, or -1. */
function last(ranges: readonly (readonly [number, ...unknown[]])[], cp: number): number {
  let low = 0;
  let high = ranges.length;
  while (low < high) {
    const middle = (low + high) >> 1;
    if (ranges[middle]![0] <= cp) low = middle + 1;
    else high = middle;
  }
  return low - 1;
}

const within = (ranges: readonly (readonly [number, number])[], cp: number) => {
  const i = last(ranges, cp);
  return i >= 0 && cp <= ranges[i]![1];
};

/** The script of a code point's own Script property, if a script. */
function ownScript(cp: number): string | undefined {
  const i = last(STRONG, cp);
  return i >= 0 && cp <= STRONG[i]![1] ? STRONG[i]![2] : undefined;
}

/** What a code point is to a run: of a script (its own, mapped or not),
 * tied by Script_Extensions to some mapped scripts, inherited (a mark,
 * going with what precedes it), or neutral (spaces, digits, punctuation). */
type Kind =
  | { kind: "script"; script: string }
  | { kind: "tied"; scripts: ReadonlySet<string> }
  | { kind: "inherited" }
  | { kind: "neutral" };

const memo = new Map<number, Kind>();

function classify(cp: number): Kind {
  let found = memo.get(cp);
  if (found) return found;
  const own = ownScript(cp);
  // its own script first if mapped; then the ties, even of a code point of
  // an unmapped script (the Arabic-Indic digits continue a Thaana run)
  const tied = new Set([...languages.keys()].filter((s) => within(EXT.get(s) ?? [], cp)));
  if (own !== undefined && languages.has(own)) found = { kind: "script", script: own };
  else if (tied.size > 0) found = { kind: "tied", scripts: tied };
  else if (own !== undefined) found = { kind: "script", script: own };
  else if (within(INHERITED, cp)) found = { kind: "inherited" };
  else found = { kind: "neutral" };
  memo.set(cp, found);
  return found;
}

interface Piece {
  text: string;
  script?: string; // a run of a mapped script
  tied?: ReadonlySet<string>;
  foreign?: boolean; // another script's own text
}

/** How many entries, from the start, are tied to `script`. */
function leadingTied(script: string, entries: readonly { tied?: ReadonlySet<string> }[]): number {
  let n = 0;
  while (n < entries.length && entries[n]!.tied?.has(script)) n++;
  return n;
}

/** Where the entries tied to `script` at the end start. */
function trailingTied(script: string, entries: readonly { tied?: ReadonlySet<string> }[]): number {
  let i = entries.length;
  while (i > 0 && entries[i - 1]!.tied?.has(script)) i--;
  return i;
}

/** A text as maximal runs of one mapped script, and what is between them:
 * neutral text joins a run only if the run goes on after it; text tied to a
 * run's script joins the run it touches. Tied text stays in pieces of one
 * character (with its marks): it may belong to a run in a neighbouring
 * inline, which only `group` sees. */
function split(text: string): Piece[] {
  const pieces: Piece[] = [];
  let run: Piece | undefined;
  let pending: Piece[] = []; // each character since the run last took one

  const close = () => {
    if (!run) return;
    const n = leadingTied(run.script!, pending);
    run.text += pending.splice(0, n).map((p) => p.text).join("");
    pieces.push(run);
    run = undefined;
  };
  const drain = () => {
    pieces.push(...pending);
    pending = [];
  };

  for (const ch of text) {
    const c = classify(ch.codePointAt(0)!);
    if (c.kind === "script" && languages.has(c.script)) {
      if (run?.script === c.script) {
        run.text += pending.map((p) => p.text).join("") + ch;
        pending = [];
      } else {
        close();
        const carried = pending.splice(trailingTied(c.script, pending)).map((p) => p.text).join("");
        drain();
        run = { text: carried + ch, script: c.script };
      }
    } else if (c.kind === "script") {
      // another script's own text ends the run, apart from the neutral text
      // around it
      close();
      drain();
      pieces.push({ text: ch, foreign: true });
    } else if (c.kind === "inherited") {
      // with its base: the last pending character, or the run
      if (pending.length > 0) pending.at(-1)!.text += ch;
      else if (run) run.text += ch;
      else pending.push({ text: ch });
    } else {
      pending.push(c.kind === "tied" ? { text: ch, tied: c.scripts } : { text: ch });
    }
  }
  close();
  drain();

  const merged: Piece[] = [];
  for (const p of pieces) {
    const before = merged.at(-1);
    if (before && !(before.script || p.script || before.tied || p.tied) && !before.foreign === !p.foreign) {
      before.text += p.text;
    } else merged.push(p);
  }
  return merged;
}

const span = (script: string, inlines: Inline[]) =>
  Span(inlines, { attributes: [["lang", languages.get(script)!], ...(RTL.has(script) ? [["dir", "rtl"] as const] : [])] });

/** Which run a list of inlines belongs to, from the text in it: its one
 * mapped script, or none and, if all it holds is tied text, the scripts all
 * of it is tied to; `undefined` if it is opaque: it holds another script, or
 * what isn't prose, or a span with a language. */
function listScript(inlines: readonly Inline[]): { script?: string; tied?: ReadonlySet<string> } | undefined {
  let found: string | undefined;
  let ties: Set<string> | undefined;
  let untied = false;

  const scan = (items: readonly Inline[]): boolean => {
    for (const x of items) {
      if (x.t === "Str") {
        for (const ch of x.text) {
          const c = classify(ch.codePointAt(0)!);
          if (c.kind === "script") {
            if (!languages.has(c.script) || (found !== undefined && found !== c.script)) return false;
            found = c.script;
          } else if (c.kind === "tied") {
            ties = ties ? new Set([...ties].filter((s) => c.scripts.has(s))) : new Set(c.scripts);
          } else if (c.kind === "neutral") {
            untied = true;
          }
          // (a mark goes with its base: it neither ties nor unties)
        }
      } else if (x.t === "Space" || x.t === "SoftBreak" || x.t === "LineBreak") {
        untied = true;
      } else if (transparent(x)) {
        if (!scan(x.content)) return false;
      } else {
        return false;
      }
    }
    return true;
  };

  if (!scan(inlines)) return undefined;
  if (found !== undefined) return { script: found };
  return untied || !ties || ties.size === 0 ? {} : { tied: ties };
}

/** The list with its runs in spans, a run going on across spaces and the
 * containers of nothing but its script; `undefined` if there is none. */
function group(inlines: Inline[]): Inline[] | undefined {
  // A run with the whole list to itself takes the neutral text around it.
  const whole = listScript(inlines)?.script;
  if (whole !== undefined) return [span(whole, inlines)];

  const out: Inline[] = [];
  let run: Inline[] = [];
  let script: string | undefined;
  let pending: { inline: Inline; tied?: ReadonlySet<string> }[] = []; // since the run last took one
  let changed = false;

  const flush = () => {
    out.push(...pending.map((p) => p.inline));
    pending = [];
  };
  const close = () => {
    if (script !== undefined) {
      run.push(...pending.splice(0, leadingTied(script, pending)).map((p) => p.inline));
      out.push(span(script, run));
      run = [];
      script = undefined;
      changed = true;
    }
    flush();
  };
  const add = (inline: Inline, of?: string, tied?: ReadonlySet<string>) => {
    if (of === undefined) {
      pending.push({ inline, tied });
    } else if (of === script) {
      run.push(...pending.map((p) => p.inline), inline);
      pending = [];
    } else {
      // the first run, or another script's: the tied text at the end of what
      // is pending is the new run's
      const carried = pending.splice(trailingTied(of, pending)).map((p) => p.inline);
      close();
      script = of;
      run.push(...carried, inline);
    }
  };

  for (const x of inlines) {
    if (x.t === "Space" || x.t === "SoftBreak") {
      add(x);
    } else if (x.t === "Str") {
      for (const piece of split(x.text)) {
        if (piece.foreign) {
          close();
          out.push(Str(piece.text));
        } else add(Str(piece.text), piece.script, piece.tied);
      }
    } else if (transparent(x)) {
      const found = listScript(x.content);
      if (found === undefined) {
        close();
        out.push(x);
      } else add(x, found.script, found.tied);
    } else {
      close();
      out.push(x);
    }
  }
  close();
  return changed ? out : undefined;
}

// In a filter of its own, as a filter's Meta function runs after its Inlines.
const configure: Filter = {
  Meta: (meta) => {
    const map = meta["auto-lang"];
    let scripts: [string, string | undefined][] = [];
    if (map === undefined || (map.t === "MetaBool" && !map.value)) scripts = [];
    else if (map.t === "MetaMap") scripts = Object.entries(map.content).map(([k, v]) => [k, stringify(v)]);
    // (Lua's `pairs` over a list: its indices, none a script's name)
    else if (map.t === "MetaList" || map.t === "MetaInlines" || map.t === "MetaBlocks") scripts = map.content.map((_, i) => [String(i + 1), undefined]);
    else throw new Error(`auto-lang: the map of scripts to languages is ${map.t}`);
    for (const [script, language] of scripts) {
      if (NAMES.has(script) && language !== undefined) languages.set(script, language);
      else console.error(`[WARNING] auto-lang: no Unicode script named ${script} (Unicode ${VERSION})`);
    }
    memo.clear();
  },
};

const tag: Filter<"topdown"> = {
  traverse: "topdown",
  // A language written by hand wins: the span and what is in it stay.
  Span: (span, ctx) => {
    if (hasAttribute(span.attr, "lang")) ctx.skipChildren();
  },
  Inlines: (inlines) => (languages.size > 0 ? group(inlines) : undefined),
};

await runFilter([configure, tag]);
