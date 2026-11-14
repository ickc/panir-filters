#!/usr/bin/env julia
# RST-like list tables in Markdown: a pandoc filter.
#
# A div of class `list-table` (or `list-table-body`, without a head)
# holding a bullet list of bullet lists becomes a table: each item a row,
# each item of an item a cell. A paragraph before the list is the caption.
# An empty span opening a cell gives the cell its attributes (`colspan`,
# `rowspan`, `align`), one alone before a row's list gives the row its own;
# a table in an item starts a new body with its rows. The div's attributes
# `header-rows`, `header-cols`, `aligns` (`l,c,r,d`) and `widths` shape the
# table.
#
# A port of pandoc-ext/list-table (Copyright (c) 2021 Martin Fischer,
# 2023-2024 Pandoc Extensions contributors; MIT, see ../LICENSE).

using Panir

const ALIGNMENTS = Dict("d" => AlignDefault, "l" => AlignLeft, "r" => AlignRight, "c" => AlignCenter)

# The original also shows the offending block as markdown: that needs
# pandoc's writer (LibPandoc.jl), so here only its kind is named.
struct ListTableError <: Exception
    msg::String
end
Base.showerror(io::IO, e::ListTableError) = print(io, "list-table: ", e.msg)

"A number as Lua's `tonumber` reads one (decimal only), else `nothing`."
number(text::AbstractString) =
    occursin(r"^\s*[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?\s*$", text) ? parse(Float64, text) : nothing
number(::Nothing) = nothing

"The block itself, or the first in a div the sourcepos extension wrapped it in."
skip_data_pos(block::Block) = block
skip_data_pos(div::Div) = any(((k, _),) -> k == "data-pos", div.attr.attributes) ? div.content[1] : div

"Remove an attribute, returning its value (or `nothing`)."
function pop_attribute!(attr::Attr, key)
    i = findfirst(((k, _),) -> k == key, attr.attributes)
    i === nothing ? nothing : popat!(attr.attributes, i)[2]
end

"The inlines a block holds, if it holds inlines."
inlines_of(b::Union{Plain, Para, Header}) = b.content
inlines_of(::Block) = nothing

function new_cell(item::Vector{Block})
    blocks = map(skip_data_pos, item)
    attr = Attr()
    # pandoc has no attributes on list items: an empty span opening the
    # first block holds the cell's
    first = isempty(blocks) ? nothing : inlines_of(blocks[1])
    if first !== nothing && !isempty(first) && first[1] isa Span && isempty(first[1].content)
        attr = popfirst!(first).attr
    end
    col_span = parse(Int, something(pop_attribute!(attr, "colspan"), "1"))
    row_span = parse(Int, something(pop_attribute!(attr, "rowspan"), "1"))
    alignment = get(ALIGNMENTS, something(pop_attribute!(attr, "align"), ""), AlignDefault)
    Cell(blocks, alignment, row_span, col_span; attr)
end

"The attributes of a row: a block holding one empty span."
function row_attr(block::Block)
    content = inlines_of(block)
    content === nothing && throw(ListTableError("expected list item to have row attrs"))
    length(content) == 1 || throw(ListTableError("expected row attrs to contain only one inline"))
    span = content[1]
    span isa Span || throw(ListTableError("expected row attrs to contain a span"))
    isempty(span.content) || throw(ListTableError("expected row attrs span to be empty"))
    span.attr
end

function colspecs(attr::Attr, columns::Integer)
    specs = [ColSpec() for _ in 1:columns]
    aligns = pop_attribute!(attr, "aligns")
    if aligns !== nothing
        given = filter(!isempty, split(aligns, ','))
        for a in given
            haskey(ALIGNMENTS, a) || throw(ListTableError("unknown column alignment $a"))
        end
        length(given) <= columns || throw(ListTableError("more alignments than columns"))
        for (spec, a) in zip(specs, given)
            spec.alignment = ALIGNMENTS[a]
        end
    end
    widths = pop_attribute!(attr, "widths")
    if widths !== nothing
        given = map(number, filter(!isempty, split(widths, ',')))
        (any(isnothing, given) || length(given) < columns) && throw(ListTableError("invalid widths: $widths"))
        total = sum(given; init = 0.0)
        for (spec, w) in zip(specs, given)
            spec.width = ColWidth(w / total)
        end
    end
    specs
end

function list_table(div::Div)
    kind = isempty(div.attr.classes) ? nothing : div.attr.classes[1]
    kind in ("list-table", "list-table-body") || return nothing
    content = map(skip_data_pos, div.content)
    caption = Block[]
    if !isempty(content) && content[1] isa Para
        caption = [Plain(popfirst!(content).content)]
    end
    isempty(content) && return nothing  # an empty div, or a caption alone, stays as it is
    list = content[1]
    list isa BulletList || throw(ListTableError("expected bullet list, found $(nameof(typeof(list)))"))

    bodies = [TableBody(Row[])]
    for item in list.content
        attr = Attr()
        if length(item) > 1
            attr = row_attr(skip_data_pos(item[1]))
            item = item[2:end]
        end
        length(item) == 1 || throw(ListTableError("expected item to contain only one block"))
        block = skip_data_pos(item[1])
        if block isa Table
            # its rows are a body of their own
            if !isempty(block.caption.content) || !isempty(something(block.caption.short, Inline[]))
                throw(ListTableError("table bodies can't have captions (they'd be ignored)"))
            end
            length(block.bodies) == 1 || throw(ListTableError("table bodies can't contain other table bodies"))
            isempty(bodies[end].body) || push!(bodies, TableBody(Row[]))
            bodies[end].attr = block.attr
            append!(bodies[end].body, block.bodies[1].body)
        elseif block isa BulletList
            push!(bodies[end].body, Row(map(new_cell, block.content); attr))
        else
            throw(ListTableError("expected bullet list, found $(nameof(typeof(block)))"))
        end
    end

    popfirst!(div.attr.classes)
    rows = bodies[1].body
    header_rows = something(number(pop_attribute!(div.attr, "header-rows")), kind == "list-table" ? 1 : 0)
    header_cols = something(number(pop_attribute!(div.attr, "header-cols")), 0)
    columns = isempty(rows) ? 0 : sum(cell.col_span for cell in rows[1].cells)
    specs = colspecs(div.attr, columns)
    head = [popfirst!(rows) for _ in 1:min(length(rows), max(0, floor(Int, header_rows)))]
    for body in bodies
        body.row_head_columns = trunc(Int, header_cols)
        header_cols = 0  # the first body's only
    end
    Table(Caption(caption; short = Inline[]), specs, TableHead(head), filter(b -> !isempty(b.body), bodies),
          TableFoot(); attr = div.attr)
end

run_filter(list_table)
