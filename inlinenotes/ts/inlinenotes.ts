#!/usr/bin/env node
// Notes to Markdown's inline notes (`^[...]`): a pandoc filter, for
// Markdown output. A note's paragraphs are joined by a space; a note holding
// any other block (a list, code, a quote...) stays as it is.
//
// A port of pandocfilters' examples/inlinenotes.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { RawInline, Space, Str } from "panir";
import type { Inline } from "panir";
import { runFilter } from "panir/node";

await runFilter({
  Note: (note, ctx) => {
    if (ctx.format !== "markdown") return;
    const inlines: Inline[] = [];
    for (const block of note.content) {
      switch (block.t) {
        case "Para":
          if (inlines.length > 0) inlines.push(Space());
          inlines.push(...block.content);
          break;
        case "Plain": // as the original: no space before it
          inlines.push(...block.content);
          break;
        case "HorizontalRule": // as the original: dropped
          break;
        default:
          return;
      }
    }
    return [RawInline("html", "^["), ...inlines, Str("]")];
  },
});
