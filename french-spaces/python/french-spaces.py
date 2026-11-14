#!/usr/bin/env python3
"""French punctuation's spaces for LaTeX: a pandoc filter. In LaTeX and
beamer output, a space after `«`, `“`, `‹` or before `;`, `?`, `!`, `»`,
`”`, `›` becomes a thin space (`\\thinspace{}`), and one before `:` an
unbreakable space (`~`).

A port of chdemko/pandoc-latex-french-spaces (Copyright (c) 2018-2024,
Christophe Demko; BSD-3-Clause, see ../LICENSE).
"""

from panir import Context, Filter, RawInline, Space, Str

f = Filter()


@f.on(Space)
def spaces(_: Space, ctx: Context) -> RawInline | None:
    if ctx.format not in ("latex", "beamer"):
        return None
    if isinstance(ctx.prev, Str) and ctx.prev.text.endswith(("«", "“", "‹")):
        return RawInline("tex", "\\thinspace{}")
    if isinstance(ctx.next, Str):
        if ctx.next.text.startswith(":"):
            return RawInline("tex", "~")
        if ctx.next.text.startswith((";", "?", "!", "»", "”", "›")):
            return RawInline("tex", "\\thinspace{}")
    return None


if __name__ == "__main__":
    f.main()
