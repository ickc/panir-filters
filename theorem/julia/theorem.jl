#!/usr/bin/env julia
# Divs of class `theorem` to LaTeX theorem environments, or to numbered
# theorems in HTML: a pandoc filter.
#
# A port of pandocfilters' examples/theorem.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

const count = Ref(0)

function theorem(div::Div, ctx)
    "theorem" in div.attr.classes || return nothing
    if ctx.format == "latex"
        id = div.attr.identifier
        label = isempty(id) ? "" : "\\label{$id}"
        # what is spliced in is not filtered itself: a theorem directly in
        # this one stays a div, as in the original
        [RawBlock("latex", "\\begin{theorem}$label"); div.content; RawBlock("latex", "\\end{theorem}")]
    elseif ctx.format in ("html", "html5")
        count[] += 1
        prepend!(div.content, [RawBlock("html", "<dt>Theorem $(count[])</dt>"), RawBlock("html", "<dd>")])
        push!(div.content, RawBlock("html", "</dd>\n</dl>"))
        nothing
    end
end

run_filter(theorem; traverse = :topdown)  # numbered in document order (the metadata first)
