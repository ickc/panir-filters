#!/usr/bin/env julia
# Leave out everything between `<!-- BEGIN COMMENT -->` and `<!-- END
# COMMENT -->`: a pandoc filter. Each marker is an HTML block of its own
# (blank lines around it).
#
# A port of pandocfilters' examples/comments.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

const hiding = Ref(false)

hidden() = hiding[] ? [] : nothing

function comment(raw::RawBlock)
    if raw.format == "html"
        occursin("<!-- BEGIN COMMENT -->", raw.text) && (hiding[] = true; return [])
        occursin("<!-- END COMMENT -->", raw.text) && (hiding[] = false; return [])
    end
    hidden()
end
# an inline: after a beginning in a note, the rest of its paragraph
comment(::Union{Block, Inline}) = hidden()
# as the original's walk: a list's items go, a map's values stay
comment(::MetaValue, ctx) = ctx.parent isa MetaList ? hidden() : nothing

run_filter(comment; traverse = :topdown)  # in document order (the metadata first)
