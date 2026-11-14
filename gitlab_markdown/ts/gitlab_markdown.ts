#!/usr/bin/env node
// GitLab's math to pandoc's: a pandoc filter. A code block whose first
// class is `math` becomes display math; inline math loses the backticks
// around it (GitLab writes $`x`$).
//
// A port of pandocfilters' examples/gitlab_markdown.py (Copyright (c) 2013,
// John MacFarlane; BSD-3-Clause, see ../LICENSE).

import * as pandoc from "panir";
import { runFilter } from "panir/node";

await runFilter({
  CodeBlock: (code) => {
    if (code.attr.classes[0] === "math") return pandoc.Para(pandoc.Math("DisplayMath", code.text));
  },
  Math: (math) => {
    if (math.mathType === "InlineMath") math.text = math.text.replace(/^`+|`+$/g, "");
  },
});
