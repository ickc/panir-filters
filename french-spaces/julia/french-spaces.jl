#!/usr/bin/env julia
# French punctuation's spaces for LaTeX: a pandoc filter. In LaTeX and
# beamer output, a space after `«`, `“`, `‹` or before `;`, `?`, `!`, `»`,
# `”`, `›` becomes a thin space (`\thinspace{}`), and one before `:` an
# unbreakable space (`~`).
#
# A port of chdemko/pandoc-latex-french-spaces (Copyright (c) 2018-2024,
# Christophe Demko; BSD-3-Clause, see ../LICENSE).

using Panir

text(x::Str) = x.text
text(_) = ""

function spaces(::Space, ctx)
    ctx.format in ("latex", "beamer") || return nothing
    prev, next = text(ctx.prev), text(ctx.next)
    if endswith(prev, r"[«“‹]")
        RawInline("tex", "\\thinspace{}")
    elseif startswith(next, ':')
        RawInline("tex", "~")
    elseif startswith(next, r"[;?!»”›]")
        RawInline("tex", "\\thinspace{}")
    end
end

run_filter(spaces)
