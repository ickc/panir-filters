#!/usr/bin/env node
// Interpolate metadata fields into a document: a pandoc filter.
//
// A word that is all `%{field}` becomes that field's value, if it is
// inlines (in a span) or a string. What it puts in is walked too, so a
// field may name another.
//
// A port of pandocfilters' examples/metavars.py (Copyright (c) 2013, John
// MacFarlane; BSD-3-Clause, see ../LICENSE).

import { copy, Span, Str } from "panir";
import { runFilter } from "panir/node";

const PATTERN = /^%\{(.*)\}$/;

await runFilter({
  traverse: "topdown", // pandocfilters' order: what is put in is walked
  Str: (s, ctx) => {
    const field = PATTERN.exec(s.text)?.[1];
    if (field === undefined) return;
    const value = ctx.doc.meta[field];
    // a field that isn't there is an error, as in the original
    if (value === undefined) throw new Error(`no metadata field ${field}`);
    switch (value.t) {
      case "MetaInlines":
        // the original's attributes() reads "class" as an attribute, not a class
        return Span(copy(value.content), { attributes: { class: "interpolated", field } });
      case "MetaString":
        return Str(value.text);
    }
  },
});
