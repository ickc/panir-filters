#!/usr/bin/env node
// Definition lists to bullet lists, each term in bold before its
// definitions: a pandoc filter, for Markdown without definition lists.
//
// A port of pandocfilters' examples/deflists.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { BulletList, Para, Strong } from "panir";
import { runFilter } from "panir/node";

await runFilter({
  DefinitionList: (dl) => BulletList(dl.content.map(([term, defs]) => [Para(Strong(term)), ...defs.flat()])),
});
