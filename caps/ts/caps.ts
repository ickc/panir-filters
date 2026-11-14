#!/usr/bin/env node
// Upper-case all regular text: a pandoc filter. Code, link URLs and the
// like are not text, and stay as they are.
//
// A port of pandocfilters' examples/caps.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { runFilter } from "panir/node";

await runFilter({
  Str: (s) => {
    s.text = s.text.toUpperCase();
  },
});
