#!/usr/bin/env node
// A code block with an `include` attribute takes the file's text: a pandoc
// filter.
//
// `startLine` and `endLine` (or `start-line`, `end-line`) take a range of
// lines, the first giving `startFrom` (for `numberLines`); `dedent=n`
// removes the spaces among each line's first n bytes. The attributes it
// reads are removed. A file that can't be opened leaves the block empty,
// with a message.
//
// A port of pandoc/lua-filters' include-code-files (© 2020 Bruno BEAUFILS;
// MIT, see ../LICENSE).

import { readFileSync } from "node:fs";
import { getAttribute, removeAttribute, setAttribute } from "panir";
import { runFilter } from "panir/node";

/** A number as Lua's `tonumber` reads text: decimal or hexadecimal, blanks
 *  around it; undefined if not one. */
function luaNumber(text: string): number | undefined {
  const t = text.replace(/^[ \t\n\v\f\r]+|[ \t\n\v\f\r]+$/g, "");
  if (/^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$/.test(t)) return Number(t);
  if (/^[+-]?0[xX][0-9a-fA-F]+$/.test(t)) return (t.startsWith("-") ? -1 : 1) * parseInt(t.replace(/^[+-]/, ""), 16);
  return undefined;
}

/** Text as Lua's string functions take it for an integer argument. */
function luaInteger(text: string, what: string): number {
  const n = luaNumber(text);
  if (n === undefined) throw new Error(`bad argument to '${what}' (number expected, got string)`);
  if (!Number.isInteger(n)) throw new Error(`bad argument to '${what}' (number has no integer representation)`);
  return n;
}

/** Lua's `s:sub(i, j)`: bytes i to j, counted from 1, negative from the end. */
function luaSub(s: Buffer, i: number, j = -1): Buffer {
  const n = s.length;
  const from = Math.max(i < 0 ? n + i + 1 : i, 1);
  const to = Math.min(j < 0 ? n + j + 1 : j, n);
  return from <= to ? s.subarray(from - 1, to) : Buffer.alloc(0);
}

const SPACE = 0x20;

/** The spaces among the line's first n bytes removed (Lua's strings are
 *  bytes: an é is two). */
function dedent(line: Buffer, n: string): Buffer {
  const k = luaInteger(n, "sub");
  return Buffer.concat([luaSub(line, 1, k).filter((b) => b !== SPACE), luaSub(line, k + 1)]);
}

/** Lua's `fh:lines("L")`: each line with its newline (`\n`). */
function lines(data: Buffer): Buffer[] {
  const out: Buffer[] = [];
  let start = 0;
  for (let i = data.indexOf(0x0a); i >= 0; i = data.indexOf(0x0a, start)) {
    out.push(data.subarray(start, i + 1));
    start = i + 1;
  }
  if (start < data.length) out.push(data.subarray(start));
  return out;
}

function need(n: number | undefined): number {
  if (n === undefined) throw new Error("attempt to compare nil with number");
  return n;
}

await runFilter({
  CodeBlock: (cb) => {
    const attr = cb.attr;
    const path = getAttribute(attr, "include");
    if (path === undefined) return;
    const content: Buffer[] = [];
    let data: Buffer | undefined;
    try {
      data = readFileSync(path);
    } catch (e) {
      // (Lua opens a directory, then fails to read it)
      if ((e as NodeJS.ErrnoException).code === "EISDIR") throw e;
      process.stderr.write(`Cannot open file ${path} | Skipping includes\n`);
    }
    if (data !== undefined) {
      // hyphenated attributes to camel case
      for (const [camel, hyphen] of [["startLine", "start-line"], ["endLine", "end-line"]] as const) {
        const value = getAttribute(attr, hyphen);
        if (value !== undefined) {
          setAttribute(attr, camel, value);
          removeAttribute(attr, hyphen);
        }
      }
      let start: number | undefined = 1;
      const first = getAttribute(attr, "startLine");
      if (first !== undefined) {
        setAttribute(attr, "startFrom", first);
        start = luaNumber(first);
      }
      lines(data).forEach((raw, i) => {
        const number = i + 1;
        const n = getAttribute(attr, "dedent");
        const line = n === undefined ? raw : dedent(raw, n);
        if (number >= need(start)) {
          const last = getAttribute(attr, "endLine");
          if (last === undefined || number <= need(luaNumber(last))) content.push(line);
        }
      });
    }
    for (const key of ["include", "startLine", "endLine", "dedent"]) removeAttribute(attr, key);
    cb.text = Buffer.concat(content).toString("utf8");
  },
});
