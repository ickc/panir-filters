#!/usr/bin/env python3
"""Notes to Markdown's inline notes (`^[...]`): a pandoc filter, for
Markdown output. A note's paragraphs are joined by a space; a note holding
any other block (a list, code, a quote...) stays as it is.

A port of pandocfilters' examples/inlinenotes.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

from panir import Context, Filter, HorizontalRule, Inline, Note, Para, Plain, RawInline, Space, Str

f = Filter()


@f.on(Note)
def inline_note(note: Note, ctx: Context) -> list[Inline] | None:
    if ctx.format != "markdown":
        return None
    inlines: list[Inline] = []
    for block in note.content:
        match block:
            case Para(content):
                if inlines:
                    inlines.append(Space())
                inlines.extend(content)
            case Plain(content):  # as the original: no space before it
                inlines.extend(content)
            case HorizontalRule():  # as the original: dropped
                pass
            case _:
                return None
    return [RawInline("html", "^["), *inlines, Str("]")]


if __name__ == "__main__":
    f.main()
