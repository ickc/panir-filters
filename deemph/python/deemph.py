#!/usr/bin/env python3
"""Emphasis shown in capitals: a pandoc filter. An emphasis is replaced by
its content, upper-cased (its text: code, link URLs and the like stay as
they are).

A port of pandocfilters' examples/deemph.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

from panir import Emph, Filter, Inline, Str

caps = Filter()


@caps.on(Str)
def upper(s: Str) -> None:
    s.text = s.text.upper()


# pandocfilters' order: what replaces an emphasis is not filtered itself,
# only what is in it, so an emphasis directly in another stays one
f = Filter(traverse="topdown")


@f.on(Emph)
def deemph(emph: Emph) -> list[Inline]:
    return emph.content.walk(caps)


if __name__ == "__main__":
    f.main()
