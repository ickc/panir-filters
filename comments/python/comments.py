#!/usr/bin/env python3
"""Leave out everything between `<!-- BEGIN COMMENT -->` and `<!-- END
COMMENT -->`: a pandoc filter. Each marker is an HTML block of its own
(blank lines around it).

A port of pandocfilters' examples/comments.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

from panir import Block, Context, Filter, Inline, MetaList, MetaValue, RawBlock

BEGIN = "<!-- BEGIN COMMENT -->"
END = "<!-- END COMMENT -->"

f = Filter(traverse="topdown")  # in document order (the metadata first)
hiding = False


@f.on(Block)
def hide_block(block: Block) -> list[Block] | None:
    global hiding
    if isinstance(block, RawBlock) and block.format == "html":
        if BEGIN in block.text:
            hiding = True
            return []
        if END in block.text:
            hiding = False
            return []
    return [] if hiding else None


@f.on(Inline)
def hide_inline(_: Inline) -> list[Inline] | None:
    # after a beginning in a note, the rest of its paragraph
    return [] if hiding else None


@f.on(MetaValue)
def hide_item(_: MetaValue, ctx: Context) -> list[MetaValue] | None:
    # as the original's walk: a list's items go, a map's values stay
    return [] if hiding and isinstance(ctx.parent, MetaList) else None


if __name__ == "__main__":
    f.main()
