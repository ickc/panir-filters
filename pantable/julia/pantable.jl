#!/usr/bin/env julia
# Code blocks of class `table`, CSV with a YAML header of options, made
# tables: a pandoc filter.
#
# The block holds CSV, after a YAML header of options (between `---` and
# `---` or `...`, starting the block): `caption`, `short-caption`,
# `alignment` (a letter a column among L, R, C, D), `alignment-cells` (a
# word of letters a row), `width` (a number or fraction a column, or D),
# `table-width` (then the columns without a width get one from the length
# of their text), `ms` (rows of the head, of each body's head and rows, of
# the foot), `ns-head` (row head columns of each body), `markdown` (cells,
# captions and attributes read as markdown), `fancy-table` (with markdown:
# the first column marks the head, bodies and foot, with attributes; a
# cell's first line its span and attributes), `format` (csv only). The
# document's metadata under `pantable` gives defaults, a header's keys
# replacing them. `csv-kwargs` (Python's csv.reader's arguments) is not
# supported: a warning, then the CSV read as it is.
#
# The file a block's `include` attribute names is panclude's to include
# (`pandoc -L panclude.lua`, or libpandoc's `panclude` step, run before this
# filter): a block still carrying the attribute is left as it is.
#
# The header is read by pandoc, as a metadata block (`-f markdown-smart`;
# panir's `yaml_filter`), the markdown of the cells too: LibPandoc.jl's
# `read_many`, in this process (pandocjl, `LibPandoc.convert`, or LibPandoc
# as a library), in one call; else the `pandoc` command, a process a text.
# The CSV is DelimitedFiles'.
#
# A port of ickc/pantable's `pantable` filter, 0.15.0 with the include
# attribute (© 2016-2026 Kolen Cheung; BSD-3-Clause, see ../LICENSE).

using Panir
using DelimitedFiles: readdlm

Base.identify_package("LibPandoc") === nothing || import LibPandoc

warn(message) = println(stderr, "pantable: ", message)

struct EmptyTable <: Exception end

"Texts read by pandoc, a document each, as `format` says: LibPandoc.jl's
`read_many`, the conversion's way, in one call; else the `pandoc` command, a
process a text. Tabs are expanded, as the original's `pandoc` command does
(a read with a conversion keeps them)."
function read_texts(texts, ctx, format)
    @isdefined(LibPandoc) && return LibPandoc.read_many(texts, ctx.conversion; from = format, preserve_tabs = false)
    [Panir.parse(read(pipeline(`pandoc -f $format -t json`; stdin = IOBuffer(t)), String)) for t in texts]
end

# The options, from the metadata pandoc read: a key as written or with `_`
# for `-` (the hyphenated one if both); a value of another shape than the
# option's is its default, with a warning.

"An option's value as `shape` reads it (`Panir.text`, `Panir.boolean`,
`Panir.list`, or `raw`); `nothing` if missing or of another shape."
function option(options, key, shape)
    k = haskey(options, key) ? key : replace(key, '-' => '_')
    haskey(options, k) || return nothing
    value = shape(options, k)
    value === nothing && warn("option $k with value $(repr(to_data(options[k]))) has an invalid type: the default")
    value
end
raw(options, key) = options[key]

"A number as Lua reads one, or two with `/` between (`1/3`); `nothing` for
anything else. A zero denominator fails, as the original."
function fraction(text)
    text isa AbstractString || return nothing
    top, bottom = occursin('/', text) ? split(text, '/'; limit = 2) : (text, nothing)
    a, b = lua_number(top), bottom === nothing ? 1 : lua_number(bottom)
    (a === nothing || b === nothing) && return nothing
    iszero(b) && error("division by zero: $text")
    Float64(a) / Float64(b)
end

"A whole number as Lua reads one (`3`, `3.0`)."
function whole(text)
    x = text isa AbstractString ? lua_number(text) : nothing
    x !== nothing && isfinite(x) && isinteger(x) ? Int(x) : nothing
end

"Whole numbers (`ms`, `ns-head`), else the default."
function integers(values, key)
    values === nothing && return nothing
    out = map(whole, values)
    any(isnothing, out) || return Int[x for x in out]
    warn("option $key with value $(repr(values)) has an invalid type: the default")
    nothing
end

"Words of text: split at spaces, tabs and line breaks."
words(text) = split(text, r"[ \t\n\r]+"; keepempty = false)

Base.@kwdef mutable struct Settings
    caption::Union{Nothing, MetaValue}
    short_caption::Union{Nothing, MetaValue}
    alignment::String
    alignment_cells::Vector{SubString{String}}
    width::Union{Nothing, Vector{Union{Nothing, Float64}}}
    table_width::Union{Nothing, Float64}
    ms::Union{Nothing, Vector{Int}}
    ns_head::Union{Nothing, Vector{Int}}
    markdown::Bool
    fancy_table::Bool
    format::String
end

function Settings(options::Options)
    haskey(options, "include") &&
        warn("the include key is not read: name the file with the include attribute, and run panclude first")
    option(options, "csv-kwargs", raw) === nothing ||
        warn("csv-kwargs (Python's csv.reader's) is not supported in this port: read as CSV")
    width = option(options, "width", Panir.list)
    Settings(;
        caption = option(options, "caption", raw),
        short_caption = option(options, "short-caption", raw),
        alignment = something(option(options, "alignment", Panir.text), ""),
        alignment_cells = words(something(option(options, "alignment-cells", Panir.text), "")),
        width = width === nothing ? nothing : Union{Nothing, Float64}[fraction(w) for w in width],
        table_width = fraction(option(options, "table-width", Panir.text)),
        ms = integers(option(options, "ms", Panir.list), "ms"),
        ns_head = integers(option(options, "ns-head", Panir.list), "ns-head"),
        markdown = something(option(options, "markdown", Panir.boolean), false),
        fancy_table = something(option(options, "fancy-table", Panir.boolean), false),
        format = something(option(options, "format", Panir.text), "csv"),
    )
end

"What is wrong beyond the type, for a table of m rows and n columns: the
widths, `nothing` for the default."
function normalize!(o::Settings, m, n)
    total = 0.0
    widths = Union{Nothing, Float64}[nothing for _ in 1:n]
    for (i, w) in enumerate(something(o.width, [])[1:min(end, n)])
        if w !== nothing && w >= 0
            widths[i] = w
            total += w
        end
    end
    if o.table_width !== nothing && o.table_width < total
        warn("table-width smaller than the sum of the widths, $total: dropped")
        o.table_width = nothing
    end
    ms = o.ms
    if ms !== nothing && (length(ms) < 4 || isodd(length(ms)) || any(<(0), ms) || sum(ms) != m)
        warn("ms $ms: too short, of odd length, negative or not adding up to the rows, $m: dropped")
        o.ms = nothing
    end
    bodies = o.ms === nothing ? 1 : length(o.ms) ÷ 2 - 1
    ns = o.ns_head
    if ns !== nothing && (length(ns) != bodies || any(>(n), ns))
        warn("ns-head $ns: not one a body, $bodies, or more than the columns, $n: dropped")
        o.ns_head = nothing
    end
    widths
end

const ALIGN = (AlignDefault, AlignLeft, AlignRight, AlignCenter)

"A row of alignments, by the original's arithmetic on each letter's code (L,
R, C, D, and anything else somewhere); all default if one isn't ASCII."
function aligns(text, n)
    letters = uppercase(strip(text))
    if !isascii(letters)
        warn("non-ASCII character in $letters: the default")
        return fill(AlignDefault, n)
    end
    out = [ALIGN[mod(mod(mod(Int(c) - 3, 11), 6), 4) + 1] for c in first(letters, n)]
    [out; fill(AlignDefault, n - length(out))]
end

# (the characters themselves: r"..." passes `\x{2028}` on as `{2028}`)
const LINE_BREAK = Regex("\r\n|[\n\r\x0b\f\x1c-\x1e\u0085\u2028\u2029]")

"Python's str.splitlines."
function lines(s::AbstractString)
    out = split(s, LINE_BREAK)
    !isempty(out) && isempty(out[end]) && pop!(out)
    out
end

# The data

"The rows, as long as the longest."
function load_csv(data)
    all(isspace, data) && throw(EmptyTable())
    cells = readdlm(IOBuffer(data), ',', String)  # (DelimitedFiles' defaults)
    all(cell -> isempty(strip(cell)), cells) && throw(EmptyTable())
    [cells[i, :] for i in axes(cells, 1)]
end

"A table's texts, cells spanning others: each covered cell holds its span's
text and anchor, as the original's array does (a later span overwrites an
earlier one's)."
struct Grid
    texts::Matrix{Union{Nothing, String}}
    spans::Dict{Tuple{Int, Int}, NTuple{4, Int}}  # (rows, columns, anchor)
end
Grid(texts) = Grid(texts, Dict{Tuple{Int, Int}, NTuple{4, Int}}())

function put!(g::Grid, text, rows, cols, i, j)
    for a in i:i + rows - 1, b in j:j + cols - 1
        g.texts[a, b] = text
        (rows, cols) != (1, 1) && (g.spans[(a, b)] = (rows, cols, i, j))
    end
end
shape(g::Grid, i, j) = get(g.spans, (i, j), (1, 1, i, j))[1:2]
function is_at(g::Grid, i, j)
    r, c, a, b = get(g.spans, (i, j), (1, 1, i, j))
    (r, c) == (1, 1) || (a, b) == (i, j)
end

const CELL_LINE = r"^(\([0-9, ]+\))?(\{[^{}]*\})?$"
const FANCY = r"^(\{[^{}]*\})?? ?(---|===|___)? ?(\{[^{}]*\})?$"

"The cells of a table of markdown: a cell's first line may be its span,
`(rows, columns)`, and attributes, `{...}`; with `fancy`, the first column
marks the ends of the head (===), bodies' heads (---), bodies (___) and the
foot (===), with attributes. Attributes as markdown, `[]{...}`, or \"\"."
function markdown_cells(rows, fancy::Bool)
    m, n = length(rows), length(rows[1]) - fancy
    grid = Grid(Matrix{Union{Nothing, String}}(nothing, m, n))
    cell_attrs = fill("", m, n)
    for i in 1:m, j in 1:n
        grid.texts[i, j] === nothing || continue  # (in a span)
        string_ = rows[i][j + fancy]
        ls = lines(string_)
        attr, size, content = "", (1, 1), string_
        found = isempty(ls) ? nothing : match(CELL_LINE, ls[1])
        if found !== nothing
            span, braces = something(found[1], ""), something(found[2], "")
            parts = isempty(span) ? ["1", "1"] : strip.(split(span[2:end-1], ','))
            if !all(p -> occursin(r"^\d+$", p), parts)
                warn("invalid cell shape $span: ignored")
            elseif length(parts) != 2 || any(p -> parse(Int, p) <= 0, parts)
                warn("invalid cell shape ($(join(parts, ", "))): ignored")
            elseif parse(Int, parts[1]) + i - 1 > m || parse(Int, parts[2]) + j - 1 > n
                warn("a cell overflowing the table: its attributes ignored: $string_")
            else
                attr = isempty(braces) ? "" : "[]$braces"
                size, content = (parse(Int, parts[1]), parse(Int, parts[2])), join(ls[2:end], '\n')
            end
        end
        cell_attrs[i, j] = attr
        put!(grid, content, size..., i, j)
    end
    row_attrs = fill("", m)
    ms, block_attrs = nothing, nothing
    if fancy
        markers = Tuple{Int, String, String}[]  # (row, marker, the block's attributes)
        for (i, row) in enumerate(rows)
            isempty(strip(row[1])) && continue
            found = match(FANCY, row[1])
            if found === nothing
                warn("can't read the fancy-table cell $(row[1]): ignored")
                continue
            end
            found[2] === nothing || push!(markers, (i, found[2], something(found[1], "")))
            found[3] === nothing || (row_attrs[i] = "[]$(found[3])")
        end
        isempty(markers) || ((ms, block_attrs) = fancy_blocks(markers, rows))
    end
    (; grid, ms, block_attrs, row_attrs, cell_attrs)
end

"`ms` and the blocks' attributes from the markers."
function fancy_blocks(markers, rows)
    sizes = [i - (k > 1 ? markers[k - 1][1] : 0) for (k, (i, _, _)) in enumerate(markers)]
    head = markers[1][2] == "==="
    foot = length(markers) > 1 && markers[end][2] == "==="
    bodies = Dict{Symbol, Tuple{Int, String}}[]
    for k in 1 + head:length(markers) - foot
        i, marker, attr = markers[k]
        if marker == "___"
            if !isempty(bodies) && !haskey(bodies[end], :body)
                bodies[end][:body] = (sizes[k], attr)
            else
                push!(bodies, Dict(:body => (sizes[k], attr)))
            end
        elseif marker == "---"
            push!(bodies, Dict(:head => (sizes[k], attr)))
        else
            warn("a fancy-table row neither head nor foot: ignored: $(rows[i][1])")
        end
    end
    wrap(a) = isempty(a) ? "" : "[]$a"
    ms = [head ? sizes[1] : 0]
    attrs = [head ? wrap(markers[1][3]) : ""]
    for body in bodies
        attr = ""
        for part in (:head, :body)
            size, a = get(body, part, (0, ""))
            push!(ms, size)
            isempty(a) || (attr = a)  # (the body's rows' attributes win)
        end
        push!(attrs, wrap(attr))
    end
    push!(ms, foot ? sizes[end] : 0)
    push!(attrs, foot ? wrap(markers[end][3]) : "")
    ms, attrs
end

"The longest line's characters (code points, as Python counts), and 3 (as
pandoc's widths)."
text_width(s) = maximum(length, lines(s); init = 0) + 3

"The columns without a width get one, by the length of their text, so that
the widths add up to `table_width`; a cell spanning columns gives what its
first column lacks to the next. The first m rows only."
function auto_width(grid::Grid, m, widths, table_width)
    n = size(grid.texts, 2)
    found = [Any[] for _ in 1:n]
    for i in 1:m, j in 1:n
        is_at(grid, i, j) || continue
        w, cols = text_width(something(grid.texts[i, j], "")), shape(grid, i, j)[2]
        push!(found[j], cols > 1 ? (w, cols) : w)
    end
    lengths = Int[]
    for j in 1:n
        ints = filter(w -> w isa Int, found[j])
        isempty(ints) && error("max() arg is an empty sequence: a column of spanning cells only")
        longest = maximum(ints)
        push!(lengths, longest)
        for w in found[j]
            w isa Tuple && w[1] > longest && push!(found[j + 1], w[2] > 2 ? (w[1] - longest, w[2] - 1) : w[1] - longest)
        end
    end
    unset = [j for j in 1:n if widths[j] === nothing]
    total = sum(lengths[unset]; init = 0)
    out = copy(widths)
    if total > 0
        scale = (table_width - sum(something.(widths, 0.0))) / total
        out[unset] .= lengths[unset] .* scale
    else
        out[unset] .= 0.0
    end
    out
end

# The table

function text_or_none(key, value)
    (value === nothing || value isa Union{MetaString, MetaInlines, MetaBlocks}) && return value
    warn("option $key with value $(repr(to_data(value))) has an invalid type: the default")
    nothing
end

"The caption: with markdown, as pandoc read the header (the original reads
its text as markdown); else its text, one `Str`, as the original's (a plain
table's caption is an empty paragraph). Not text: none, with a warning; a
short caption of blocks too."
function caption(long, short, markdown)
    long, short = text_or_none("caption", long), text_or_none("short-caption", short)
    if !markdown
        text(v) = v === nothing ? "" : to_data(v)
        return Caption(Para(Str(text(long))); short = Inline[Str(text(short))])
    end
    if short isa MetaBlocks
        warn("a short-caption of blocks: none")
        short = nothing
    end
    short_inlines = short === nothing ? Inline[] : inlines_of(short)
    inlines = long === nothing || long isa MetaBlocks ? Inline[] : inlines_of(long)
    blocks = long isa MetaBlocks ? blocks_of(long) : isempty(inlines) ? Block[] : Block[Para(inlines...)]
    Caption(blocks; short = isempty(short_inlines) ? nothing : short_inlines)
end

"The attributes of `[]{...}` read: its span's; else none."
function attr_of(blocks)
    if !isempty(blocks)
        first_ = hasproperty(blocks[1], :content) ? first(blocks[1].content) : nothing
        first_ isa Span && return first_.attr
        warn("can't read the attributes: none")
    end
    Attr()
end

"A cell of one paragraph is plain text."
plain(blocks) = length(blocks) == 1 && blocks[1] isa Para ? Block[Plain(blocks[1].content...)] : blocks

function table(options, data, cb, ctx)
    if haskey(cb.attr, "include")
        warn("a code block with an include attribute: left as it is (run panclude first)")
        return nothing
    end
    settings = Settings(options)
    rows = try
        settings.format == "csv" || error("Unknown format: $(settings.format)")
        load_csv(strip(==('\n'), data))  # (blank lines around the data dropped)
    catch e
        if e isa EmptyTable
            warn("table is empty: deleted")
            return Block[]
        end
        rethrow()
    end
    md = settings.markdown ? markdown_cells(rows, settings.fancy_table) : nothing
    grid = md === nothing ? Grid(Matrix{Union{Nothing, String}}(permutedims(reduce(hcat, rows)))) : md.grid
    rows_in, n = size(grid.texts)
    widths = normalize!(settings, rows_in, n)
    ms = something(md === nothing ? nothing : md.ms, settings.ms, Any[1, 0, rows_in - 1, 0])
    # the table has the rows of its blocks: with fancy-table, the rows after
    # the last marker are dropped (as the original, which sizes it by ms)
    m = Int(sum(ms))
    ns_head = something(settings.ns_head, zeros(Int, length(ms) ÷ 2 - 1))
    settings.table_width === nothing || (widths = auto_width(grid, m, widths, settings.table_width))
    specs = [ColSpec(; alignment = a, width = w === nothing ? ColWidthDefault() : ColWidth(w))
             for (a, w) in zip(aligns(settings.alignment, n), widths)]
    cells_ = settings.alignment_cells
    cell_aligns = [aligns(i <= length(cells_) ? cells_[i] : "", n) for i in 1:m]

    anchors = [(i, j) for i in 1:m for j in 1:n if is_at(grid, i, j)]
    nblocks = length(ms) ÷ 2 + 1
    if md !== nothing
        # every text of the table in one call: cells and their attributes,
        # rows' and blocks' attributes
        texts = String[[grid.texts[a...] for a in anchors]; [md.cell_attrs[a...] for a in anchors]; md.row_attrs[1:m];
                       something(md.block_attrs, fill("", nblocks))]
        todo = filter(!isempty, texts)
        done = Iterators.Stateful(isempty(todo) ? Pandoc[] : read_texts(todo, ctx, "markdown"))
        got = [isempty(t) ? Block[] : popfirst!(done).blocks for t in texts]
        k = length(anchors)
        contents = map(plain, got[1:k])
        attrs = map(attr_of, got[k + 1:2k])
        row_attrs = map(attr_of, got[2k + 1:2k + m])
        block_attrs = map(attr_of, got[2k + m + 1:end])
    else
        contents = [Block[Plain(Str(grid.texts[a...]))] for a in anchors]
        attrs = [Attr() for _ in anchors]
        row_attrs = [Attr() for _ in 1:m]
        block_attrs = [Attr() for _ in 1:nblocks]
    end
    at = Dict(a => k for (k, a) in enumerate(anchors))
    row(i) = Row([Cell(contents[at[(i, j)]], cell_aligns[i][j], shape(grid, i, j)...; attr = attrs[at[(i, j)]])
                  for j in 1:n if haskey(at, (i, j))]; attr = row_attrs[i])
    # the rows of each block
    ends = cumsum(ms[1:end-1])
    parts = [[row(i) for i in a + 1:b] for (a, b) in zip([0; ends], [ends; m])]
    head = TableHead(parts[1]; attr = block_attrs[1])
    bodies = [TableBody(parts[1 + 2k], parts[2k], ns; attr = block_attrs[1 + k]) for (k, ns) in enumerate(ns_head)]
    foot = TableFoot(parts[end]; attr = block_attrs[end])
    attr = Attr(cb.attr.identifier, filter(!=("table"), cb.attr.classes), cb.attr.attributes)
    Table(caption(settings.caption, settings.short_caption, settings.markdown), specs, head, bodies, foot; attr)
end

pantable(doc::Pandoc, ctx) =
    yaml_filter((options, data, cb) -> table(options, data, cb, ctx), doc, "table", "pantable",
                texts -> read_texts(texts, ctx, "markdown-smart"))

run_filter(pantable)
