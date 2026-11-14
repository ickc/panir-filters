#!/usr/bin/env python3
"""Interpolate metadata fields into a document: a pandoc filter.

A word that is all `%{field}` becomes that field's value, if it is
inlines (in a span) or a string. What it puts in is walked too, so a
field may name another.

A port of pandocfilters' examples/metavars.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

import copy
import re

from panir import Context, Filter, MetaInlines, MetaString, Span, Str

f = Filter(traverse="topdown")  # pandocfilters' order: what is put in is walked
PATTERN = re.compile(r"%\{(.*)\}$")


@f.on(Str)
def metavars(s: Str, ctx: Context) -> Span | Str | None:
    m = PATTERN.match(s.text)
    if m is None:
        return None
    field = m.group(1)
    match ctx.doc.meta[field]:  # a field that isn't there is an error, as in the original
        case MetaInlines(content):
            # the original's attributes() reads "class" as an attribute, not a class
            return Span(*copy.deepcopy(content), attributes={"class": "interpolated", "field": field})
        case MetaString(text):
            return Str(text)
    return None


if __name__ == "__main__":
    f.main()
