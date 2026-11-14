#!/usr/bin/env python3
"""Divs of class `theorem` to LaTeX theorem environments, or to numbered
theorems in HTML: a pandoc filter.

A port of pandocfilters' examples/theorem.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

import itertools

from panir import Block, Context, Div, Filter, RawBlock

f = Filter(traverse="topdown")  # numbered in document order (the metadata first)
numbers = itertools.count(1)


@f.on(Div)
def theorems(div: Div, ctx: Context) -> list[Block] | None:
    if "theorem" not in div.classes:
        return None
    if ctx.format == "latex":
        label = f"\\label{{{div.identifier}}}" if div.identifier else ""
        # what is spliced in is not filtered itself: a theorem directly in
        # this one stays a div, as in the original
        return [RawBlock("latex", f"\\begin{{theorem}}{label}"), *div.content, RawBlock("latex", "\\end{theorem}")]
    if ctx.format in ("html", "html5"):
        div.content[:0] = [RawBlock("html", f"<dt>Theorem {next(numbers)}</dt>"), RawBlock("html", "<dd>")]
        div.content.append(RawBlock("html", "</dd>\n</dl>"))
    return None


if __name__ == "__main__":
    f.main()
