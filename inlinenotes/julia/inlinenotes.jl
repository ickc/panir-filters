#!/usr/bin/env julia
# Notes to Markdown's inline notes (`^[...]`): a pandoc filter, for
# Markdown output. A note's paragraphs are joined by a space; a note holding
# any other block (a list, code, a quote...) stays as it is.
#
# A port of pandocfilters' examples/inlinenotes.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

function inlinenote(note::Note, ctx)
    ctx.format == "markdown" || return nothing
    inlines = Inline[]
    for block in note.content
        if block isa Para
            isempty(inlines) || push!(inlines, Space())
            append!(inlines, block.content)
        elseif block isa Plain  # as the original: no space before it
            append!(inlines, block.content)
        elseif !(block isa HorizontalRule)  # a rule is dropped, as in the original
            return nothing
        end
    end
    [RawInline("html", "^["); inlines; Str("]")]
end

run_filter(inlinenote)
