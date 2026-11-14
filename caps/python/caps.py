#!/usr/bin/env python3
"""Upper-case all regular text: a pandoc filter. Code, link URLs and the
like are not text, and stay as they are.

A port of pandocfilters' examples/caps.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

from panir import Filter, Str

f = Filter()


@f.on(Str)
def caps(s: Str) -> None:
    s.text = s.text.upper()


if __name__ == "__main__":
    f.main()
