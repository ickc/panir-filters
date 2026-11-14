#!/usr/bin/env node
// Leave out everything between `<!-- BEGIN COMMENT -->` and `<!-- END
// COMMENT -->`: a pandoc filter. Each marker is an HTML block of its own
// (blank lines around it).
//
// A port of pandocfilters' examples/comments.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { runFilter } from "panir/node";

let hiding = false;

await runFilter({
  traverse: "topdown", // in document order (the metadata first)
  Block: (block) => {
    if (block.t === "RawBlock" && block.format === "html") {
      if (block.text.includes("<!-- BEGIN COMMENT -->")) {
        hiding = true;
        return [];
      }
      if (block.text.includes("<!-- END COMMENT -->")) {
        hiding = false;
        return [];
      }
    }
    return hiding ? [] : undefined;
  },
  // after a beginning in a note, the rest of its paragraph
  Inline: () => (hiding ? [] : undefined),
  // as the original's walk: a list's items go, a map's values stay
  MetaValue: (_value, ctx) => (hiding && ctx.parent?.t === "MetaList" ? [] : undefined),
});
