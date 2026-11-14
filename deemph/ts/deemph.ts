#!/usr/bin/env node
// Emphasis shown in capitals: a pandoc filter. An emphasis is replaced by
// its content, upper-cased (its text: code, link URLs and the like stay as
// they are).
//
// A port of pandocfilters' examples/deemph.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { applyFilter } from "panir";
import type { Filter } from "panir";
import { runFilter } from "panir/node";

const caps: Filter = {
  Str: (s) => {
    s.text = s.text.toUpperCase();
  },
};

await runFilter({
  // pandocfilters' order: what replaces an emphasis is not filtered itself,
  // only what is in it, so an emphasis directly in another stays one
  traverse: "topdown",
  Emph: (emph) => applyFilter(emph.content, caps),
});
