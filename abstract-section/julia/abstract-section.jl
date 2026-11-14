#!/usr/bin/env julia
# A section headed "Abstract" moves into the metadata: a pandoc filter.
#
# A section whose heading has the identifier `abstract`, at the highest
# level met so far, in any list of blocks, runs to the next such heading or
# a horizontal rule; it becomes the metadata's `abstract`, unless that is
# set. `abstract-section.section-identifiers` lists other identifiers (each
# its own field); the configuration is removed.
#
# A port of pandoc-ext/abstract-section (Copyright © 2017-2023 Albert
# Krewinkel; MIT, see ../LICENSE).

using Panir

"Whether Lua counts the value: anything but missing or false."
given(::Nothing) = false
given(v::MetaBool) = v.value
given(::MetaValue) = true

# `abstract-section.section-identifiers`, as the original reads it
configured(config::MetaMap) = get(config.content, "section-identifiers", nothing)
configured(config::MetaBool) = config.value ? error("abstract-section: the configuration is true, not a map") : nothing
configured(_) = nothing  # (a string, a list... has no field in Lua)

const DEFAULT = Set(["abstract"])
# (each inline of text too)
identifiers(ids::Union{MetaList, MetaInlines, MetaBlocks}) = isempty(ids.content) ? DEFAULT : Set(map(stringify, ids.content))
identifiers(ids::MetaString) = isempty(ids.text) ? DEFAULT : Set{String}()  # (Lua takes its length, then finds no items)
identifiers(ids::MetaBool) = ids.value ? error("abstract-section: section-identifiers is true, not a list") : DEFAULT
identifiers(_) = DEFAULT

function abstract_section(doc::Pandoc)
    ids = identifiers(configured(pop!(doc.meta, "abstract-section", nothing)))
    collected = Dict{String, Vector{Block}}()
    toplevel = 6  # the highest level of heading met so far
    function sections(blocks::Vector{Block})
        body = Block[]
        section = nothing  # the section being collected
        for block in blocks
            if block isa Header && block.level <= toplevel
                toplevel = block.level
                id = block.attr.identifier
                section = id in ids ? (collected[id] = Block[]) : nothing
                section === nothing && push!(body, block)
            elseif section === nothing
                push!(body, block)
            elseif block isa HorizontalRule
                section = nothing
            else
                push!(section, block)
            end
        end
        body
    end
    walk!(sections, doc.blocks)  # the body's lists, inner ones first
    for (key, blocks) in collected
        !isempty(blocks) && !given(get(doc.meta, key, nothing)) && (doc.meta[key] = MetaBlocks(blocks))
    end
    nothing
end

run_filter(abstract_section)
