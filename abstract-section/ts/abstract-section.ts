#!/usr/bin/env node
// A section headed "Abstract" moves into the metadata: a pandoc filter.
//
// A section whose heading has the identifier `abstract`, at the highest
// level met so far, in any list of blocks, runs to the next such heading or
// a horizontal rule; it becomes the metadata's `abstract`, unless that is
// set. `abstract-section.section-identifiers` lists other identifiers (each
// its own field); the configuration is removed.
//
// A port of pandoc-ext/abstract-section (Copyright © 2017-2023 Albert
// Krewinkel; MIT, see ../LICENSE).

import { applyFilter, lookup, MetaBlocks, stringify, truthy, type Block, type MetaValue } from "panir";
import { runFilter } from "panir/node";

/** `abstract-section.section-identifiers`, as the original reads it. */
function sectionIdentifiers(config: MetaValue | undefined): Set<string> {
  if (config?.t === "MetaBool" && config.value) throw new Error("abstract-section: the configuration is true, not a map");
  const ids = lookup(config, "section-identifiers"); // (a string, a list... has no field in Lua)
  switch (ids?.t) {
    case "MetaList":
    case "MetaInlines":
    case "MetaBlocks":
      // (each inline of text too)
      if (ids.content.length > 0) return new Set(ids.content.map((x: unknown) => stringify(x)));
      break;
    case "MetaString":
      if (ids.text !== "") return new Set(); // (Lua takes its length, then finds no items)
      break;
    case "MetaBool":
      if (ids.value) throw new Error("abstract-section: section-identifiers is true, not a list");
  }
  return new Set(["abstract"]);
}

await runFilter({
  Pandoc: (doc) => {
    const identifiers = sectionIdentifiers(doc.meta["abstract-section"]);
    delete doc.meta["abstract-section"];
    const collected = new Map<string, Block[]>();
    let toplevel = 6; // the highest level of heading met so far

    // the body's lists, inner ones first
    applyFilter(doc.blocks, {
      Blocks: (blocks) => {
        const body: Block[] = [];
        let section: Block[] | undefined; // the section being collected
        for (const block of blocks) {
          if (block.t === "Header" && block.level <= toplevel) {
            toplevel = block.level;
            const id = block.attr.identifier;
            section = identifiers.has(id) ? [] : undefined;
            if (section) collected.set(id, section);
            else body.push(block);
          } else if (section === undefined) {
            body.push(block);
          } else if (block.t === "HorizontalRule") {
            section = undefined;
          } else {
            section.push(block);
          }
        }
        return body;
      },
    });
    for (const [key, blocks] of collected) {
      if (blocks.length > 0 && !truthy(doc.meta[key])) doc.meta[key] = MetaBlocks(blocks);
    }
  },
});
