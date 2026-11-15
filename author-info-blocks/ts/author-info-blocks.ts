#!/usr/bin/env node
// Affiliations and correspondence as blocks before the body: a pandoc
// filter, run after scholarly-metadata (whose `author` and `institute` it
// reads).
//
// The body starts with a note on equal contributors (if any), the
// affiliations, numbered, and the corresponding authors (`correspondence`
// and an `email`), linked; `author` becomes one line of names with their
// marks (in LaTeX, a list, one each), and `institute` goes.
//
// A port of pandoc/lua-filters' author-info-blocks (Copyright © 2017–2021
// Albert Krewinkel; ISC; the repository's MIT licence, see ../LICENSE).

import {
  copy, items, LineBreak, Link, MetaInlines, MetaList, Para, RawInline, Space, Str, stringify, Superscript, truthy,
  type Block, type Inline, type Meta, type MetaValue,
} from "panir";
import { runFilter } from "panir/node";

/** `author` or `institute`, as scholarly-metadata leaves them. */
const maps = (value: MetaValue | undefined): Meta[] =>
  items(value).map((x) => {
    if (x.t !== "MetaMap") throw new Error(`author-info-blocks: expected a map, got ${x.t}`);
    return x.content;
  });

/** A name or an email: inlines (the original fails on anything else). */
function inlines(value: MetaValue | undefined): Inline[] {
  if (value?.t !== "MetaInlines") throw new Error(`author-info-blocks: expected inlines, got ${value?.t}`);
  return value.content;
}

function intercalate(lists: Inline[][], sep: () => Inline[]): Inline[] {
  // a new separator each time: a node is in one place
  return lists.flatMap((xs, i) => (i ? [...sep(), ...xs] : xs));
}

const corresponding = (author: Meta) => truthy(author.correspondence) && truthy(author.email);

await runFilter({
  Pandoc: (doc, ctx) => {
    const mark = (name: "equal_contributor" | "corresponding_author"): Inline[] => {
      const latex = ctx.format === "latex";
      if (name === "corresponding_author") return [latex ? RawInline("latex", "*") : Str("✉")];
      return [latex ? RawInline("latex", "$\\dagger{}$") : Str("*")];
    };
    /** The author's name, with its marks as a superscript. */
    const authorLine = (author: Meta): Inline[] => {
      const marks = truthy(author.equal_contributor) ? [mark("equal_contributor")] : [];
      marks.push(...items(author.institute).map((i) => [Str(stringify(i))]));
      if (corresponding(author)) marks.push(mark("corresponding_author"));
      return [...inlines(author.name), Superscript(intercalate(marks, () => [Str(",")]))];
    };

    const authors = maps(doc.meta.author);
    const institutes = maps(doc.meta.institute);
    delete doc.meta.institute;

    const body: Block[] = [];
    if (authors.some((a) => truthy(a.equal_contributor))) {
      body.push(Para([Superscript(mark("equal_contributor")), Space(), Str("These authors contributed equally to this work.")]));
    }
    const lines = institutes.map((inst, i) => [Superscript([Str(String(i + 1))]), Space(), ...inlines(inst.name)]);
    body.push(Para(intercalate(lines, () => [LineBreak()])));
    // the name is in the author line too: a copy here
    const links = authors.filter(corresponding).map((a) => [
      Link([...copy(inlines(a.name)), Space(), Str("<"), ...inlines(a.email), Str(">")], { url: "mailto:" + stringify(a.email) }),
    ]);
    if (links.length > 0) {
      body.push(Para([Superscript(mark("corresponding_author")), Space(), Str("Correspondence:"), Space(),
        ...intercalate(links, () => [Str(","), Space()])]));
    }
    doc.blocks.unshift(...body);

    const names = authors.map(authorLine);
    if (ctx.format?.includes("latex")) { // (the marks above ask for "latex" exactly)
      doc.meta.author = MetaList(names.map((name) => MetaInlines(name)));
      return;
    }
    const last = names.pop();
    if (last === undefined) throw new Error("author-info-blocks: no authors"); // (the original fails too)
    const line = intercalate(names, () => [Str(","), Space()]);
    if (names.length > 0) line.push(Str(","), Space(), Str("and"), Space());
    doc.meta.author = MetaInlines([...line, ...last]);
  },
});
