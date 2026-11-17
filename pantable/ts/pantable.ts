#!/usr/bin/env node
// Code blocks of class `table`, CSV with YAML options, made tables: a pandoc
// filter.
//
// The block holds CSV, and YAML blocks (each between `---` and `---` or
// `...`) of options: `caption`, `short-caption`, `alignment` (a letter a
// column among L, R, C, D), `alignment-cells` (a line of letters a row),
// `width` (a number or fraction a column, or D), `table-width` (then the
// columns without a width get one from the length of their text), `ms`
// (rows of the head, of each body's head and rows, of the foot), `ns-head`
// (row head columns of each body), `markdown` (cells, captions and
// attributes read as markdown), `fancy-table` (with markdown: the first
// column marks the head, bodies and foot, with attributes; a cell's first
// line its span and attributes), `include` (the CSV from a file, relative to
// the working directory, `include-encoding` its encoding), `format` (csv
// only). `csv-kwargs` (Python's csv.reader's arguments) is not supported:
// a warning, then the CSV read as it is.
//
// The CSV is read by csv-parse, the YAML by yaml (as YAML 1.1, PyYAML's),
// each option made the type the original wants as Python makes it
// (`str(1.5)`, `list("12")`, `bool("false")`); the markdown of the cells is
// pandoc's: in libpandoc's JavaScript host, the instance the filter runs in
// (`hosted`, for panir's `libpandocFilter`), all of a table's texts in one
// call (`readMany`), as the conversion reads its input; run as a JSON
// filter, the `pandoc` command, a process a text.
//
// A port of ickc/pantable's `pantable` filter, 0.15.0 (© 2016-2026 Kolen
// Cheung; BSD-3-Clause, see ../LICENSE).

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import { parse as parseCSV } from "csv-parse/sync";
import {
  Attr, Caption, Cell, ColSpec, ColWidth, ColWidthDefault, fromJSON, Para, parse, Plain, Row, Str, Table,
  TableBody, TableFoot, TableHead,
  type Alignment, type Block, type CodeBlock, type Conversion, type Filter, type Inline, type Pandoc,
} from "panir";
import { runFilter } from "panir/node";
import { parseDocument } from "yaml";

const warn = (message: string) => process.stderr.write(`pantable: ${message}\n`);

/** What reads many texts of markdown, in the conversion the filter runs in. */
type Reader = (texts: string[], conversion: Conversion) => Block[][];

/** libpandoc's JavaScript host, as much as this filter uses of it. */
interface Host {
  readMany(texts: string[], from: unknown, options?: Record<string, unknown>): unknown[];
}

/** The instance the filter runs in: all the texts in one call, their tabs
 *  expanded, as the original's `pandoc` command does (a read with a
 *  conversion keeps them). */
const byHost = (pandoc: Host): Reader => (texts, conversion) =>
  pandoc.readMany(texts, conversion, { from: "markdown", "preserve-tabs": false }).map((doc) => {
    const error = (doc as { error?: { kind: string; message: string } }).error;
    if (error) throw new Error(`${error.kind}: ${error.message}`);
    return (fromJSON(doc) as Pandoc).blocks;
  });

/** The `pandoc` command: a process a text. */
const byCommand: Reader = (texts) =>
  texts.map((text) => parse(execFileSync("pandoc", ["-f", "markdown", "-t", "json"], { input: text }).toString("utf8")).blocks);

// Python's exceptions that the original catches, to fall back to a default
// (ValueError, TypeError): anything else fails, as there.
class PyError extends Error {}

class EmptyTable extends Error {}

class NotFound extends Error {}

// The code block: YAML and data, as panflute's yaml_filter (strict_yaml)
// splits it: at lines of three or more dots or dashes; a chunk starting
// with --- opens YAML (even one of data), --- or ... closes it.

const SEPARATOR = /^([.]{3,}|[-]{3,})$/m;

// PyYAML reads YAML 1.1: `yes`, `on` booleans, `010` octal, `1:30` 90;
// dates stay text (as Python's str of a date)
const YAML_OPTIONS: Parameters<typeof parseDocument>[1] = {
  version: "1.1", intAsBigInt: true, uniqueKeys: false,
  customTags: (tags) => tags.filter((t) => (typeof t === "string" ? t : t.tag) !== "tag:yaml.org,2002:timestamp"),
};

function split(text: string): [Map<string, unknown>, string] | undefined {
  const options = new Map<string, unknown>();
  const data: string[] = [];
  let raw = true;
  for (let chunk of text.split(new RegExp(SEPARATOR, "gm"))) {
    chunk = chunk.replace(/^\n+|\n+$/g, "");
    if (!chunk) continue;
    if (raw) {
      if (chunk.startsWith("---")) raw = false;
      else data.push(chunk);
    } else if (chunk.startsWith("---") || chunk.startsWith("...")) {
      raw = true;
    } else {
      const doc = parseDocument(chunk, YAML_OPTIONS);
      if (doc.errors.length > 0) {
        warn("malformed YAML block: left as it is");
        return undefined;
      }
      if (doc.warnings.length > 0) throw new Error(`YAML: ${doc.warnings[0]!.message}`);
      let value: unknown;
      try {
        value = doc.toJS(); // (an alias to no anchor fails here)
      } catch (e) {
        warn(`malformed YAML block: left as it is (${(e as Error).message})`);
        return undefined;
      }
      // (Python's dict.update: a map, or a list of pairs)
      for (const [k, v] of pairs(value)) options.set(pyStr(k), v);
    }
  }
  return [options, data.join("\n")];
}

// A value as Python has it, made another type as Python makes it

/** Python's str() of a value. */
function pyStr(v: unknown): string {
  return typeof v === "string" ? v : pyRepr(v);
}

function pyRepr(v: unknown): string {
  if (v === null || v === undefined) return "None";
  if (typeof v === "boolean") return v ? "True" : "False";
  if (typeof v === "bigint") return v.toString();
  if (typeof v === "number") return pyFloat(v);
  if (typeof v === "string") {
    const q = v.includes("'") && !v.includes('"') ? '"' : "'";
    return q + v.replace(/[\\\n\r\t]|[\x00-\x1f\x7f]/g, (c) =>
      ({ "\\": "\\\\", "\n": "\\n", "\r": "\\r", "\t": "\\t" })[c] ?? `\\x${c.charCodeAt(0).toString(16).padStart(2, "0")}`)
      .replaceAll(q, `\\${q}`) + q;
  }
  if (Array.isArray(v)) return `[${v.map(pyRepr).join(", ")}]`;
  return `{${Object.entries(v as object).map(([k, x]) => `${pyRepr(k)}: ${pyRepr(x)}`).join(", ")}}`;
}

/** Python's repr of a float: the shortest digits, in exponent form below
 *  1e-4 and from 1e16. */
function pyFloat(x: number): string {
  if (Number.isNaN(x)) return "nan";
  if (!Number.isFinite(x)) return x > 0 ? "inf" : "-inf";
  const [mantissa, e] = x.toExponential().split("e") as [string, string];
  const exp = Number(e);
  if (exp < -4 || exp >= 16) return `${mantissa}e${exp < 0 ? "-" : "+"}${String(Math.abs(exp)).padStart(2, "0")}`;
  const s = String(x);
  return s.includes(".") ? s : `${s}.0`;
}

/** Python's bool(). */
function pyBool(v: unknown): boolean {
  if (v === null || v === undefined) return false;
  if (typeof v === "string" || Array.isArray(v)) return v.length > 0;
  if (typeof v === "object") return Object.keys(v).length > 0;
  return typeof v === "number" ? v !== 0 : Boolean(v);
}

/** Python's list(): of text its characters, of a map its keys. */
function pyList(v: unknown): unknown[] {
  if (typeof v === "string") return [...v];
  if (Array.isArray(v)) return v;
  if (v !== null && typeof v === "object") return Object.keys(v);
  throw new PyError(`'${typeof v}' object is not iterable`);
}

/** Python's int() of a value (`ms`, `ns-head`). */
function pyInt(v: unknown): number {
  if (typeof v === "bigint") return Number(v);
  if (typeof v === "boolean") return Number(v);
  if (typeof v === "number") {
    if (Number.isNaN(v)) throw new PyError("cannot convert float NaN to integer");
    if (!Number.isFinite(v)) throw new Error("cannot convert float infinity to integer");
    return Math.trunc(v);
  }
  if (typeof v === "string" && /^\s*[-+]?\d+(_\d+)*\s*$/.test(v)) return Number(v.replaceAll("_", ""));
  throw new PyError(`invalid literal for int(): ${pyRepr(v)}`);
}

// Python's fractions.Fraction of text
const FRACTION = /^\s*([-+]?)(?=\d|\.\d)(\d*|\d+(?:_\d+)*)(?:(?:\s*\/\s*(\d+(?:_\d+)*))?|(?:\.(\d*|\d+(?:_\d+)*))?(?:[eE]([-+]?\d+(?:_\d+)*))?)\s*$/;

/** float(Fraction(v)): a number, or a fraction or decimal as text. */
function fraction(v: unknown): number {
  if (typeof v === "bigint") return Number(v);
  if (typeof v === "boolean") return Number(v);
  if (typeof v === "number") {
    if (Number.isNaN(v)) throw new PyError("cannot convert NaN to integer ratio");
    if (!Number.isFinite(v)) throw new Error("cannot convert Infinity to integer ratio");
    return v;
  }
  if (typeof v !== "string") throw new PyError("argument should be a string or a number");
  const m = FRACTION.exec(v);
  if (!m) throw new PyError(`Invalid literal for Fraction: ${pyRepr(v)}`);
  const [, sign, num, denom] = m;
  if (denom !== undefined) {
    if (Number(denom.replaceAll("_", "")) === 0) throw new Error(`Fraction(${num}, 0)`);
    return Number(`${sign}${num!.replaceAll("_", "")}`) / Number(denom.replaceAll("_", ""));
  }
  return Number(v.trim().replaceAll("_", ""));
}

/** Python's dict(): a map, or a list of pairs. */
function pairs(v: unknown): [unknown, unknown][] {
  if (Array.isArray(v)) {
    return v.map((p: unknown) => {
      const pair = typeof p === "string" ? [...p] : p;
      if (!Array.isArray(pair) || pair.length !== 2) throw new PyError("dictionary update sequence element has not length 2");
      return pair as [unknown, unknown];
    });
  }
  if (v !== null && typeof v === "object") return Object.entries(v);
  throw new PyError(`'${pyRepr(v)}' object is not iterable`);
}

interface Options {
  "short-caption": string;
  caption: string;
  alignment: string;
  "alignment-cells": string;
  width: unknown[] | null;
  "table-width": number | null;
  header: boolean; // (read, and never used: as the original)
  ms: number[] | null;
  "ns-head": number[] | null;
  markdown: boolean;
  "fancy-table": boolean;
  include: string;
  "include-encoding": string;
  format: string;
  "csv-kwargs": boolean; // (given: not supported here)
}

const defaults = (): Options => ({
  "short-caption": "", caption: "", alignment: "", "alignment-cells": "", width: null, "table-width": null,
  header: true, ms: null, "ns-head": null, markdown: false, "fancy-table": false, include: "",
  "include-encoding": "", format: "csv", "csv-kwargs": false,
});

const maybe = <T>(f: (v: unknown) => T) => (v: unknown): T | null => (v === null ? null : f(v));

const CASTS: Record<keyof Options, (v: unknown) => unknown> = {
  "short-caption": pyStr, caption: pyStr, alignment: pyStr, "alignment-cells": pyStr, include: pyStr,
  "include-encoding": pyStr, format: pyStr, header: pyBool, markdown: pyBool, "fancy-table": pyBool,
  width: maybe(pyList), ms: maybe(pyList), "ns-head": maybe(pyList),
  "table-width": maybe((v) => (typeof v === "number" ? v : fraction(v))), // (a float as it is, infinity too)
  "csv-kwargs": () => true,
};

function optionsOf(given: Map<string, unknown>): Options {
  const options = defaults();
  // keys with - or _ alike, the later winning
  const named = new Map<string, unknown>();
  for (const [k, v] of given) named.set(k.replaceAll("_", "-"), v);
  for (const [name, value] of named) {
    if (!(name in CASTS)) continue;
    try {
      (options as unknown as Record<string, unknown>)[name] = CASTS[name as keyof Options](value);
    } catch (e) {
      if (!(e instanceof PyError)) throw e;
      warn(`option ${name} with value ${pyStr(value)} has an invalid type: the default`);
    }
  }
  for (const name of ["ms", "ns-head"] as const) {
    try {
      options[name] = options[name]?.map(pyInt) ?? null;
    } catch (e) {
      if (!(e instanceof PyError)) throw e;
      warn(`option ${name} has an invalid type: the default`);
      options[name] = null;
    }
  }
  return options;
}

/** What is wrong beyond the type, for a table of m rows and n columns. */
function normalize(options: Options, m: number, n: number): (number | null)[] {
  let total = 0;
  const widths: (number | null)[] = Array(n).fill(null);
  options.width?.slice(0, n).forEach((w, i) => {
    let x: number;
    try {
      x = fraction(w);
    } catch (e) {
      if (e instanceof PyError) return;
      throw e;
    }
    if (x >= 0) {
      widths[i] = x;
      total += x;
    }
  });
  if (options["table-width"] !== null && options["table-width"] < total) {
    warn(`table-width smaller than the sum of the widths, ${total}: dropped`);
    options["table-width"] = null;
  }
  const ms = options.ms;
  if (ms && (ms.length < 4 || ms.length % 2 || ms.some((x) => x < 0) || ms.reduce((a, b) => a + b, 0) !== m)) {
    warn(`ms ${ms}: too short, of odd length, negative or not adding up to the rows, ${m}: dropped`);
    options.ms = null;
  }
  const bodies = options.ms ? options.ms.length / 2 - 1 : 1;
  const ns = options["ns-head"];
  if (ns && (ns.length !== bodies || ns.some((x) => x > n))) {
    warn(`ns-head ${ns}: not one a body, ${bodies}, or more than the columns, ${n}: dropped`);
    options["ns-head"] = null;
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

/** Python's names of the encodings to WHATWG's, as TextDecoder knows them;
 *  Latin-1 is Node's own (WHATWG's is windows-1252). */
function decode(bytes: Buffer, name: string): string {
  const key = name.toLowerCase().replace(/[^a-z0-9.]+/g, "_");
  if (["latin_1", "latin1", "iso_8859_1", "iso8859_1", "8859", "cp819", "l1", "latin"].includes(key)) {
    return bytes.toString("latin1");
  }
  const sig = key === "utf_8_sig" || key === "utf8_sig";
  const label = sig || ["utf_8", "utf8", "u8", "utf", "cp65001"].includes(key) ? "utf-8"
    : key === "utf_16" || key === "utf16" ? (bytes[0] === 0xfe && bytes[1] === 0xff ? "utf-16be" : "utf-16le")
    : key.replaceAll("_", "-");
  return new TextDecoder(label, { fatal: true, ignoreBOM: !(sig || label.startsWith("utf-16")) }).decode(bytes);
}

/** The rows, as long as the longest. */
function loadCSV(data: string, options: Options): string[][] {
  let text = data;
  if (options.include) {
    let bytes: Buffer;
    try {
      bytes = readFileSync(options.include);
    } catch (e) {
      if ((e as NodeJS.ErrnoException).code === "ENOENT") throw new NotFound(`include path ${options.include} not found.`);
      throw e;
    }
    text = decode(bytes, options["include-encoding"] || "utf-8");
  }
  if (options["csv-kwargs"]) warn("csv-kwargs (Python's csv.reader's) is not supported in this port: read as CSV");
  // (csv-parse's defaults, but for rows of other lengths, which pantable pads)
  const rows = parseCSV(text, { relax_column_count: true }) as string[][];
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

const table = (read: Reader) => (block: CodeBlock, ctx: { conversion: Conversion }): Block | Block[] | undefined => {
  if (!block.attr.classes.includes("table")) return undefined;
  const parsed = split(block.text);
  if (!parsed) return undefined;
  const options = optionsOf(parsed[0]);
  let rows: string[][];
  try {
    if (options.format !== "csv") throw new Error(`Unknown format: ${options.format}`);
    rows = loadCSV(parsed[1], options);
  } catch (e) {
    if (e instanceof NotFound) {
      warn(`${e.message} The code block is left as it is.`);
      return undefined;
    }
    if (e instanceof EmptyTable) {
      warn("table is empty: deleted");
      return [];
    }
    throw e;
  }
  const md = options.markdown ? markdownCells(rows, options["fancy-table"]) : undefined;
  const grid = md?.grid ?? new Grid(rows);
  const n = grid.texts[0]!.length;
  let widths = normalize(options, grid.texts.length, n);
  const ms = md?.ms ?? options.ms ?? [1, 0, grid.texts.length - 1, 0];
  // the table has the rows of its blocks: with fancy-table, the rows after
  // the last marker are dropped (as the original, which sizes it by ms)
  const m = ms.reduce((a, b) => a + b, 0);
  grid.texts = grid.texts.slice(0, m);
  const nsHead = options["ns-head"] ?? Array<number>(ms.length / 2 - 1).fill(0);
  if (options["table-width"] !== null) widths = autoWidth(grid, widths, options["table-width"]);
  const specs = aligns(options.alignment, n).map((a, j) => ColSpec(a, widths[j] !== null ? ColWidth(widths[j]!) : ColWidthDefault()));
  const alignLines = lines(options["alignment-cells"].trim()).slice(0, m);
  const cellAligns = grid.texts.map((_, i) => aligns(alignLines[i] ?? "", n));

  const anchors = grid.texts.flatMap((row, i) => row.flatMap((_, j) => (grid.isAt(i, j) ? [[i, j] as const] : [])));
  const blocks = ms.length / 2 + 1;
  let caption: Caption, contents: Block[][], attrs: Attr[], rowAttrs: Attr[], blockAttrs: Attr[];
  if (md) {
    // every text of the table in one call: captions, cells and their
    // attributes, rows' and blocks' attributes
    const texts = [options.caption, options["short-caption"], ...anchors.map(([i, j]) => grid.texts[i]![j]!),
      ...anchors.map(([i, j]) => md.cellAttrs[i]![j]!), ...md.rowAttrs.slice(0, m), ...(md.blockAttrs ?? Array<string>(blocks).fill(""))];
    const todo = texts.filter((t) => t);
    const done = todo.length > 0 ? read(todo, ctx.conversion) : [];
    let k = 0;
    const got = texts.map((t) => (t ? done[k++]! : []));
    const short = got[1]!.length > 0 ? (got[1]![0] as { content: Inline[] }).content : null;
    caption = Caption(got[0], { short });
    contents = got.slice(2, 2 + anchors.length).map(plain);
    attrs = got.slice(2 + anchors.length, 2 + 2 * anchors.length).map(attrOf);
    rowAttrs = got.slice(2 + 2 * anchors.length, 2 + 2 * anchors.length + m).map(attrOf);
    blockAttrs = got.slice(2 + 2 * anchors.length + m).map(attrOf);
  } else {
    caption = Caption([Para([Str(options.caption)])], { short: [Str(options["short-caption"])] });
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
  return Table(caption, specs, head, bodies, foot, { identifier, classes: classes.filter((c) => c !== "table"), attributes });
};

const filters = (read: Reader): Filter[] => [{ CodeBlock: table(read) }];

/** For libpandoc's JavaScript host, given the instance the filter runs in
 *  (`libpandocFilter(hosted)`): reading with it. */
export const hosted = (pandoc: Host): Filter[] => filters(byHost(pandoc));

// run as a JSON filter (the main module): the pandoc command reads
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await runFilter(filters(byCommand));
