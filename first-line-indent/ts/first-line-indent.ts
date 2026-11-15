#!/usr/bin/env node
// First-line indents, removed after quotes, lists and the like: a pandoc
// filter.
//
// A paragraph right after a block quote, a list, a code block or a rule
// (`first-line-indent.remove-after`, `dont-remove-after`) gets `\noindent` in
// LaTeX, or a marker div before it in HTML, unless it starts with `\indent`
// or `\noindent` itself. The metadata's `indent` is set true, and
// `header-includes` gets the CSS (HTML) or the indent's `size` (LaTeX).
//
// A port of pandoc/lua-filters' first-line-indent (Copyright (c) 2021 Julien
// Dutant; MIT, see ../LICENSE).

import {
  equal, given, items, lookup, MetaBlocks, MetaBool, Plain, RawBlock, RawInline, stringify, toPlain, truthy,
  type Block, type Blocks, type Filter, type MetaValue, type Para,
} from "panir";
import { runFilter } from "panir/node";

const CSS = `  <style>
    p {
      text-indent: SIZE;
      margin: 0;
    }
    header + p {
      text-indent: ;
    }
    :is(h1, h2, h3, h4, h5, h6) + p {
      text-indent: 0;
    }
    :is(div.no-first-line-indent-after) + p {
      text-indent: 0;
    }
    :is(div.first-line-indent-after) + p {
      text-indent: SIZE;
    }
  </style>
`;

const DIVS = {
  indent: '<div class="first-line-indent-after"></div>',
  noindent: '<div class="no-first-line-indent-after"></div>',
};
type Kind = keyof typeof DIVS;

// read from the metadata by the first filter, used by the second
const options = {
  setMetadataVariable: true,
  setHeaderIncludes: true,
  autoRemove: true,
  removeAfter: ["BlockQuote", "BulletList", "CodeBlock", "DefinitionList", "HorizontalRule", "OrderedList"],
  size: "1em",
};

/** `\indent ` or `\noindent `, as the markdown reader makes them. */
const command = (kind: Kind) => RawInline("tex", `\\${kind} `);

/** Block types: text is one, a list's items each one; else none. */
function names(value: MetaValue | undefined): string[] | undefined {
  if (value?.t === "MetaInlines") return [stringify(value)];
  if (value?.t === "MetaList") return value.content.map((x) => stringify(x));
}

/** `text` as Lua's `gsub` reads a replacement, for a pattern without
 * captures: `%0` and `%1` are the match, `%%` a percent sign, any other `%`
 * an error. (So a size of `5%` fails, as in the original.) */
const luaReplacement = (text: string, match: string) =>
  text.replace(/%([\s\S]?)/g, (_, c: string) => {
    if (c === "0" || c === "1") return match;
    if (c === "%") return "%";
    throw new Error("first-line-indent: invalid use of '%' in replacement string");
  });

/** A metadata value where a block goes, as Lua converts it: text a `Plain`
 * of it, a map (a table) an empty `Plain`; blocks, a list or a boolean fail. */
const asBlock = (value: MetaValue) => (value.t === "MetaMap" ? Plain([]) : toPlain(value));

const configure: Filter = {
  Meta: (meta, ctx) => {
    const user = meta["first-line-indent"];
    if (equal(user, MetaBool(true))) throw new Error("first-line-indent: a configuration of true"); // (Lua indexes a boolean)
    if (truthy(user)) {
      const isFalse = (key: string) => equal(lookup(user, key), MetaBool(false));
      if (isFalse("set-metadata-variable")) options.setMetadataVariable = false;
      if (isFalse("set-header-includes")) options.setHeaderIncludes = false;
      if (isFalse("auto-remove")) options.autoRemove = false;
      options.removeAfter.push(...(names(lookup(user, "remove-after")) ?? []));
      const blacklist = names(lookup(user, "dont-remove-after"));
      if (blacklist) options.removeAfter = options.removeAfter.filter((x) => !blacklist.includes(x));
      const size = lookup(user, "size");
      if (size !== undefined) options.size = stringify(size);
    }

    // The original changes the metadata, then returns it only if it set
    // `indent`: otherwise (Lua's `return nil`) nothing it changed counts,
    // but what fails as it changes it (header-includes) fails all the same.
    const changes = options.setMetadataVariable && !equal(meta.indent, MetaBool(false));
    const format = ctx.format ?? "";
    let header: Block | undefined;
    // (Lua's pattern `html*`: `htm`, then any `l`s)
    if (!options.setHeaderIncludes) header = undefined;
    else if (format.includes("htm")) header = RawBlock("html", CSS.replaceAll("SIZE", luaReplacement(options.size, "SIZE")));
    else if (format.includes("latex") && options.size !== "1em") header = RawBlock("tex", `\\setlength{\\parindent}{${options.size}}`);
    // before what is there: a list's items, or one value
    const includes = header && MetaBlocks([header, ...items(given(meta["header-includes"])).map(asBlock)] as Blocks);
    if (!changes) return;
    meta.indent = MetaBool(true);
    if (includes) meta["header-includes"] = includes;
  },
};

/** The paragraph with its first line indented or not. */
function markup(kind: Kind, para: Para, format: string): Block[] {
  if (format.includes("latex")) {
    const [first] = para.content;
    if (equal(first, command("indent")) || equal(first, command("noindent"))) para.content[0] = command(kind);
    else para.content.unshift(command(kind));
    return [para];
  }
  if (format.includes("htm")) return [RawBlock("html", DIVS[kind]), para];
  return [para];
}

const body: Filter = {
  Pandoc: (doc, ctx) => {
    const format = ctx.format ?? "";
    const blocks: Block[] = [];
    let after = false; // whether the block before removes an indent
    for (const block of doc.blocks) {
      if (block.t !== "Para") {
        after = options.removeAfter.includes(block.t);
        blocks.push(block);
        continue;
      }
      const explicit = (["indent", "noindent"] as const).find((kind) => equal(block.content[0], command(kind)));
      if (explicit) blocks.push(...markup(explicit, block, format));
      else if (options.autoRemove && after) blocks.push(...markup("noindent", block, format));
      else blocks.push(block);
      after = false;
    }
    doc.blocks = blocks;
  },
};

await runFilter([configure, body]); // the options and the metadata, then the body
