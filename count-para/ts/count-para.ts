#!/usr/bin/env node
// Numbered paragraphs, and citations of them: a pandoc filter.
//
// Each paragraph of the body (not one opening with an image) gets a number,
// in the margin (LaTeX) or in a span before it, per chapter with
// `resetAtChapter`. A paragraph opening with `{#name}` can be cited as
// `[@name]`: the citation becomes a link to its number. `enclosing`,
// `chapterSep`, `refName` and `addPageNr` shape the numbers and the links;
// `header-includes` gets the CSS or the LaTeX packages.
//
// A port of cysouw/count-para (Copyright © 2021, 2023 Michael Cysouw; ISC;
// the repository's CC0-1.0, see ../LICENSE).

import { Link, MetaBlocks, MetaList, RawBlock, RawInline, Space, Span, stringify, truthy, type Filter, type MetaValue } from "panir";
import { runFilter } from "panir/node";

// (as the original's, its spaces at the ends of lines too)
const CSS = `\x20
<!-- CSS added by lua-filter 'count-para' -->
<style>
.paragraph-number {\x20
  float: left;
  margin-left: -5em;
  width: 4.5em;
  text-align: right;
  color: grey;
  font-size: x-small;
  padding-top: 5px;
}
</style>
    `;

const LATEX = [
  String.raw`\usepackage{xcolor}`,
  String.raw`\usepackage{marginnote}`,
  String.raw`\reversemarginpar`,
  String.raw`\newcommand{\paragraphnumber}[1]{\marginnote{\color{lightgray}\tiny{#1}}[0pt]}`,
];

// the options
let resetAtChapter = false;
let enclosing = "[]";
let chapterSep = ".";
let refName = "paragraph ";
let addPageNr = true;
// the numbering
let count = 0;
let chapter = 0;
const numbers = new Map<string, string>(); // by the names given

/** The text's `i`th byte, as Lua's `sub` takes it and pandoc reads it back:
 * an ASCII character, else U+FFFD (or nothing past the end). */
const byte = (text: string, i: number) => new TextDecoder().decode(new TextEncoder().encode(text).subarray(i, i + 1));

const formatting: Filter = {
  Meta: (meta, ctx) => {
    const format = ctx.format ?? "";
    const added: MetaValue[] = [];
    if (format.includes("html")) added.push(MetaBlocks([RawBlock("html", CSS)]));
    if (format.includes("latex")) added.push(...LATEX.map((tex) => MetaBlocks([RawBlock("tex", tex)])));
    // The original appends to what is there as to a Lua table: a list grows,
    // a map's entries are lost (the table becomes a list of the new ones),
    // anything else fails; nothing there is an empty list, even with nothing
    // to add.
    const existing = meta["header-includes"];
    if (existing === undefined || (existing.t === "MetaMap" && added.length > 0)) meta["header-includes"] = MetaList(added);
    else if (existing.t === "MetaList") existing.content.push(...added);
    else if (added.length > 0) throw new Error(`count-para: header-includes is ${existing.t}, not a list`);
  },
};

const settings: Filter = {
  Meta: (meta, ctx) => {
    if (meta.resetAtChapter !== undefined) resetAtChapter = truthy(meta.resetAtChapter); // (Lua's truth: set and not false)
    if (meta.enclosing !== undefined) enclosing = stringify(meta.enclosing);
    if (meta.chapterSep !== undefined) chapterSep = stringify(meta.chapterSep);
    if (meta.refName !== undefined) {
      refName = stringify(meta.refName);
      if (ctx.format?.includes("latex") && refName === "#") refName = "\\#";
    }
    if (meta.addPageNr !== undefined) addPageNr = truthy(meta.addPageNr);
  },
};

const numbering: Filter = {
  Pandoc: (doc, ctx) => {
    const latex = ctx.format?.includes("latex") ?? false;
    for (const block of doc.blocks) { // (the body's own paragraphs only)
      if (block.t === "Header" && block.level === 1 && block.attr.classes[0] !== "unnumbered" && resetAtChapter) {
        chapter += 1;
        count = 0;
      }
      if (block.t !== "Para") continue;
      const [first] = block.content;
      // (an empty paragraph fails, as in the original: it has no first inline)
      if (first === undefined) throw new Error("count-para: an empty paragraph");
      if (first.t === "Image") continue;
      count += 1;
      const number = resetAtChapter ? `${chapter}${chapterSep}${count}` : String(count);
      const shown = new TextEncoder().encode(enclosing).length === 1
        ? `${enclosing}${number}${enclosing}`
        : `${byte(enclosing, 0)}${number}${byte(enclosing, 1)}`;

      // a name given at the start: anywhere in the first inline's text
      const name = /\{#([\s\S]*)\}/.exec(stringify(first))?.[1];
      if (name !== undefined) {
        numbers.set(name, number);
        block.content.shift();
        // (nothing left fails, as in the original)
        if (block.content.length === 0) throw new Error("count-para: a paragraph of a name alone");
        if (block.content[0]!.t === "Space") block.content.shift();
      }

      if (latex) { // after the first word, to keep the number on its page
        let tex = `\\paragraphnumber{${shown}}`;
        if (name !== undefined) tex = `\\hypertarget{${name}}{\n${tex}\\label{${name}}}`;
        block.content.splice(1, 0, RawInline("tex", tex));
      } else {
        block.content.unshift(Span(shown, { identifier: number, classes: ["paragraph-number"] }), Space());
      }
    }
  },
};

const references: Filter = {
  Cite: (cite, ctx) => {
    const name = cite.citations[0]!.id;
    const number = numbers.get(name);
    if (number === undefined) return;
    if (ctx.format?.includes("latex")) {
      const page = addPageNr ? ` on page~\\pageref{${name}}` : "";
      return RawInline("tex", `${refName}\\hyperlink{${name}}{${number}}${page}`);
    }
    return Link(refName + number, { url: `#${number}` });
  },
};

// header-includes, the options, the numbers, then the citations (the
// metadata's too)
await runFilter([formatting, settings, numbering, references]);
