#!/usr/bin/env julia
# Interpolate metadata fields into a document: a pandoc filter.
#
# A word that is all `%{field}` becomes that field's value, if it is
# inlines (in a span) or a string. What it puts in is walked too, so a
# field may name another.
#
# A port of pandocfilters' examples/metavars.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

function metavars(s::Str, ctx)
    m = match(r"^%\{(.*)\}$", s.text)
    m === nothing && return nothing
    field = m[1]
    value = ctx.doc.meta[field]  # a field that isn't there is an error, as in the original
    if value isa MetaInlines
        # the original's attributes() reads "class" as an attribute, not a class
        Span(deepcopy(value.content); attributes = ["class" => "interpolated", "field" => field])
    elseif value isa MetaString
        Str(value.text)
    end
end

# pandocfilters' order: what is put in is walked
run_filter(metavars; traverse = :topdown)
