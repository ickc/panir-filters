#!/usr/bin/env node
// Divs of class `theorem` to LaTeX theorem environments, or to numbered
// theorems in HTML: a pandoc filter.
//
// A port of pandocfilters' examples/theorem.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { RawBlock } from "panir";
import { runFilter } from "panir/node";

let count = 0;

await runFilter({
  traverse: "topdown", // numbered in document order (the metadata first)
  Div: (div, ctx) => {
    if (!div.attr.classes.includes("theorem")) return;
    if (ctx.format === "latex") {
      const id = div.attr.identifier;
      const label = id === "" ? "" : `\\label{${id}}`;
      // what is spliced in is not filtered itself: a theorem directly in
      // this one stays a div, as in the original
      return [RawBlock("latex", `\\begin{theorem}${label}`), ...div.content, RawBlock("latex", "\\end{theorem}")];
    }
    if (ctx.format === "html" || ctx.format === "html5") {
      count += 1;
      div.content.unshift(RawBlock("html", `<dt>Theorem ${count}</dt>`), RawBlock("html", "<dd>"));
      div.content.push(RawBlock("html", "</dd>\n</dl>"));
    }
  },
});
