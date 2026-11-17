#!/usr/bin/env julia
# Code blocks of class `table`, CSV with YAML options, made tables: a pandoc
# filter.
#
# The block holds CSV, and YAML blocks (each between `---` and `---` or
# `...`) of options: `caption`, `short-caption`, `alignment` (a letter a
# column among L, R, C, D), `alignment-cells` (a line of letters a row),
# `width` (a number or fraction a column, or D), `table-width` (then the
# columns without a width get one from the length of their text), `ms`
# (rows of the head, of each body's head and rows, of the foot), `ns-head`
# (row head columns of each body), `markdown` (cells, captions and
# attributes read as markdown), `fancy-table` (with markdown: the first
# column marks the head, bodies and foot, with attributes; a cell's first
# line its span and attributes), `include` (the CSV from a file, relative to
# the working directory, `include-encoding` its encoding), `format` (csv
# only). `csv-kwargs` (Python's csv.reader's arguments) is not supported: a
# warning, then the CSV read as it is.
#
# The CSV is read by DelimitedFiles, the YAML by YAML.jl (a port of PyYAML,
# the original's), each option made the type the original wants as Python
# makes it (`str(1.5)`, `list("12")`, `bool("false")`); an included file's
# encoding by StringEncodings. The markdown of the cells is pandoc's:
# LibPandoc.jl's `read_many`, in this process (pandocjl, `LibPandoc.convert`,
# or LibPandoc as a library), all of a table's texts in one call. Without
# LibPandoc a table with markdown fails, saying so; the others don't need it.
#
# A port of ickc/pantable's `pantable` filter, 0.15.0 (© 2016-2026 Kolen
# Cheung; BSD-3-Clause, see ../LICENSE).

using Panir
using DelimitedFiles: readdlm
using OrderedCollections: OrderedDict
using StringEncodings: decode as decode_with
import Dates
import YAML

Base.identify_package("LibPandoc") === nothing || import LibPandoc

warn(message) = println(stderr, "pantable: ", message)

# Python's exceptions that the original catches, to fall back to a default
# (ValueError, TypeError): anything else fails, as there.
struct PyValueError <: Exception
    msg::String
end

struct EmptyTable <: Exception end

struct NotFound <: Exception
    msg::String
end

# The code block: YAML and data, as panflute's yaml_filter (strict_yaml)
# splits it: at lines of three or more dots or dashes; a chunk starting with
# --- opens YAML (even one of data), --- or ... closes it.

is_separator(line) = length(line) >= 3 && (all(==('.'), line) || all(==('-'), line))

"The options and the data; `nothing` for YAML that can't be scanned."
function split_yaml(text::AbstractString)
    chunks = [""]
    for line in split(text, '\n')
        if is_separator(line)
            push!(chunks, line, "")
        else
            chunks[end] *= line * "\n"
        end
    end
    options = OrderedDict{String, Any}()
    data = String[]
    raw = true
    for chunk in chunks
        chunk = strip(chunk, '\n')
        isempty(chunk) && continue
        if raw
            startswith(chunk, "---") ? (raw = false) : push!(data, chunk)
        elseif startswith(chunk, "---") || startswith(chunk, "...")
            raw = true
        else
            value = try
                YAML.load(chunk; dicttype = OrderedDict{Any, Any})
            catch e
                e isa YAML.ScannerError || rethrow()
                warn("malformed YAML block: left as it is")
                return nothing
            end
            # (Python's dict.update: a map, or a list of pairs)
            for (k, v) in py_pairs(value)
                options[py_str(k)] = v
            end
        end
    end
    options, join(data, '\n')
end

# A value as Python has it, made another type as Python makes it

"Python's str() of a value."
py_str(v::AbstractString) = String(v)
py_str(v) = py_repr(v)

py_repr(::Nothing) = "None"
py_repr(v::Bool) = v ? "True" : "False"
py_repr(v::Integer) = string(v)
py_repr(v::AbstractFloat) = py_float(Float64(v))
py_repr(v::Dates.Date) = string(v)
py_repr(v::Dates.DateTime) = replace(string(v), 'T' => ' ')
py_repr(v::AbstractVector) = "[" * join(map(py_repr, v), ", ") * "]"
py_repr(v::AbstractDict) = "{" * join(("$(py_repr(k)): $(py_repr(x))" for (k, x) in v), ", ") * "}"
function py_repr(v::AbstractString)
    q = occursin('\'', v) && !occursin('"', v) ? '"' : '\''
    escaped = map(collect(v)) do c
        c == '\\' ? "\\\\" : c == '\n' ? "\\n" : c == '\r' ? "\\r" : c == '\t' ? "\\t" : c == q ? "\\$c" :
        (c < ' ' || c == '\x7f') ? "\\x" * string(UInt32(c); base = 16, pad = 2) : string(c)
    end
    string(q, join(escaped), q)
end

"Python's repr of a float: the shortest digits, in exponent form below 1e-4
and from 1e16."
function py_float(x::Float64)
    isnan(x) && return "nan"
    isinf(x) && return x > 0 ? "inf" : "-inf"
    sign = signbit(x) ? "-" : ""
    iszero(x) && return sign * "0.0"
    s = string(abs(x))  # (the shortest digits: 1000.0, 1.0e16, 0.0001)
    m, e = occursin('e', s) ? split(s, 'e') : (s, "0")
    i, f = split(m, '.')
    digits, exp = i * f, parse(Int, e) + length(i) - 1
    k = findfirst(!=('0'), digits)
    digits, exp = rstrip(digits[k:end], '0'), exp - (k - 1)
    if -4 <= exp < 16
        exp < 0 && return sign * "0." * "0"^(-exp - 1) * digits
        whole = rpad(digits[1:min(end, exp + 1)], exp + 1, '0')
        fraction = length(digits) > exp + 1 ? digits[exp + 2:end] : "0"
        return sign * whole * "." * fraction
    end
    mantissa = length(digits) > 1 ? digits[1:1] * "." * digits[2:end] : digits
    sign * mantissa * "e" * (exp < 0 ? "-" : "+") * lpad(abs(exp), 2, '0')
end

"Python's bool()."
py_bool(::Nothing) = false
py_bool(v::Bool) = v
py_bool(v::Number) = !iszero(v)
py_bool(v::Union{AbstractString, AbstractVector, AbstractDict}) = !isempty(v)
py_bool(v) = true

"Python's list(): of text its characters, of a map its keys."
py_list(v::AbstractString) = Any[string(c) for c in v]
py_list(v::AbstractVector) = v
py_list(v::AbstractDict) = collect(Any, keys(v))
py_list(v) = throw(PyValueError("'$(py_repr(v))' object is not iterable"))

"Python's int() of a value (`ms`, `ns-head`)."
py_int(v::Integer) = Int(v)
function py_int(v::AbstractFloat)
    isnan(v) && throw(PyValueError("cannot convert float NaN to integer"))
    isinf(v) && error("cannot convert float infinity to integer")
    trunc(Int, v)
end
function py_int(v::AbstractString)
    occursin(r"^\s*[-+]?\d+(_\d+)*\s*$", v) || throw(PyValueError("invalid literal for int(): $(py_repr(v))"))
    parse(Int, replace(strip(v), '_' => ""))
end
py_int(v) = throw(PyValueError("int() of $(py_repr(v))"))

# Python's fractions.Fraction of text
const FRACTION = r"^\s*([-+]?)(?=\d|\.\d)(\d*|\d+(?:_\d+)*)(?:(?:\s*/\s*(\d+(?:_\d+)*))?|(?:\.(\d*|\d+(?:_\d+)*))?(?:[eE]([-+]?\d+(?:_\d+)*))?)\s*$"

"float(Fraction(v)): a number, or a fraction or decimal as text."
fraction(v::Integer) = Float64(v)
function fraction(v::AbstractFloat)
    isnan(v) && throw(PyValueError("cannot convert NaN to integer ratio"))
    isinf(v) && error("cannot convert Infinity to integer ratio")
    Float64(v)
end
function fraction(v::AbstractString)
    m = match(FRACTION, v)
    m === nothing && throw(PyValueError("Invalid literal for Fraction: $(py_repr(v))"))
    sign, num, denom = m.captures
    if denom !== nothing
        d = parse(Float64, replace(denom, '_' => ""))
        iszero(d) && error("Fraction($num, 0)")
        return parse(Float64, sign * replace(num, '_' => "")) / d
    end
    parse(Float64, replace(strip(v), '_' => ""))
end
fraction(v) = throw(PyValueError("argument should be a string or a number"))

"Python's dict(): a map, or a list of pairs."
py_pairs(v::AbstractDict) = collect(v)
function py_pairs(v::AbstractVector)
    map(v) do p
        pair = p isa AbstractString ? [string(c) for c in p] : p
        pair isa AbstractVector && length(pair) == 2 ||
            throw(PyValueError("dictionary update sequence element has not length 2"))
        pair[1] => pair[2]
    end
end
py_pairs(v) = throw(PyValueError("'$(py_repr(v))' object is not iterable"))

Base.@kwdef mutable struct Options
    short_caption::String = ""
    caption::String = ""
    alignment::String = ""
    alignment_cells::String = ""
    width::Union{Nothing, Vector{Any}} = nothing
    table_width::Union{Nothing, Float64} = nothing
    header::Bool = true  # (read, and never used: as the original)
    ms::Union{Nothing, Vector{Any}} = nothing
    ns_head::Union{Nothing, Vector{Any}} = nothing
    markdown::Bool = false
    fancy_table::Bool = false
    include::String = ""
    include_encoding::String = ""
    format::String = "csv"
    csv_kwargs::Bool = false  # (given: not supported here)
end

maybe(f) = v -> v === nothing ? nothing : f(v)

const CASTS = Dict{Symbol, Function}(
    :short_caption => py_str, :caption => py_str, :alignment => py_str, :alignment_cells => py_str,
    :include => py_str, :include_encoding => py_str, :format => py_str,
    :header => py_bool, :markdown => py_bool, :fancy_table => py_bool,
    :width => maybe(py_list), :ms => maybe(py_list), :ns_head => maybe(py_list),
    # (a float as it is, infinity too)
    :table_width => maybe(v -> v isa AbstractFloat ? Float64(v) : fraction(v)),
    :csv_kwargs => v -> true,
)

function Options(given::AbstractDict)
    options = Options()
    # keys with - or _ alike, the later winning
    named = OrderedDict{Symbol, Any}(Symbol(replace(k, '-' => '_')) => v for (k, v) in given)
    for (name, value) in named
        haskey(CASTS, name) || continue
        try
            setproperty!(options, name, CASTS[name](value))
        catch e
            e isa PyValueError || rethrow()
            warn("option $(replace(string(name), '_' => '-')) with value $(py_str(value)): $(e.msg): the default")
        end
    end
    for name in (:ms, :ns_head)
        value = getfield(options, name)
        value === nothing && continue
        try
            setproperty!(options, name, Any[py_int(x) for x in value])
        catch e
            e isa PyValueError || rethrow()
            warn("option $(replace(string(name), '_' => '-')): $(e.msg): the default")
            setproperty!(options, name, nothing)
        end
    end
    options
end

"What is wrong beyond the type, for a table of m rows and n columns: the
widths, `nothing` for the default."
function normalize!(o::Options, m, n)
    total = 0.0
    widths = Union{Nothing, Float64}[nothing for _ in 1:n]
    for (i, w) in enumerate(something(o.width, Any[])[1:min(end, n)])
        x = try
            fraction(w)
        catch e
            e isa PyValueError ? continue : rethrow()
        end
        if x >= 0
            widths[i] = x
            total += x
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

"A file's bytes as text in an encoding named as Python names it: UTF-8 and
Latin-1 by Julia, the others by iconv (StringEncodings)."
function decode(bytes::Vector{UInt8}, name::AbstractString)
    key = replace(lowercase(name), r"[^a-z0-9.]+" => "_")
    if key in ("utf_8", "utf8", "u8", "utf", "cp65001", "utf_8_sig", "utf8_sig")
        endswith(key, "sig") && length(bytes) >= 3 && bytes[1:3] == [0xef, 0xbb, 0xbf] && (bytes = bytes[4:end])
        text = String(bytes)
        isvalid(text) || error("'utf-8' codec can't decode the file")
        return text
    elseif key in ("latin_1", "latin1", "iso_8859_1", "iso8859_1", "8859", "cp819", "l1", "latin")
        return String(map(Char, bytes))
    end
    decode_with(bytes, replace(key, '_' => '-'))
end

"The rows, as long as the longest."
function load_csv(data, options::Options)
    text = data
    if !isempty(options.include)
        path = options.include
        isdir(path) && error("[Errno 21] Is a directory: '$path'")
        bytes = try
            read(path)
        catch e
            e isa SystemError && e.errnum == Base.Libc.ENOENT && throw(NotFound("include path $path not found."))
            rethrow()
        end
        text = decode(bytes, isempty(options.include_encoding) ? "utf-8" : options.include_encoding)
    end
    options.csv_kwargs && warn("csv-kwargs (Python's csv.reader's) is not supported in this port: read as CSV")
    all(isspace, text) && throw(EmptyTable())
    cells = readdlm(IOBuffer(text), ',', String)  # (DelimitedFiles' defaults)
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

"Each text's blocks, read by pandoc as markdown, the conversion's way, in
one call; tabs expanded, as the original's `pandoc` command does (a read
with a conversion keeps them)."
function read_markdown(texts, ctx)
    @isdefined(LibPandoc) || error("pantable: markdown cells need pandoc's reader, LibPandoc.jl")
    [doc.blocks for doc in LibPandoc.read_many(texts, ctx.conversion; from = "markdown", preserve_tabs = false)]
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

function pantable(cb::CodeBlock, ctx)
    "table" in cb.attr.classes || return nothing
    parsed = split_yaml(cb.text)
    parsed === nothing && return nothing
    options = Options(parsed[1])
    rows = try
        options.format == "csv" || error("Unknown format: $(options.format)")
        load_csv(parsed[2], options)
    catch e
        if e isa NotFound
            warn("$(e.msg) The code block is left as it is.")
            return nothing
        elseif e isa EmptyTable
            warn("table is empty: deleted")
            return Block[]
        end
        rethrow()
    end
    md = options.markdown ? markdown_cells(rows, options.fancy_table) : nothing
    grid = md === nothing ? Grid(Matrix{Union{Nothing, String}}(permutedims(reduce(hcat, rows)))) : md.grid
    rows_in, n = size(grid.texts)
    widths = normalize!(options, rows_in, n)
    ms = something(md === nothing ? nothing : md.ms, options.ms, Any[1, 0, rows_in - 1, 0])
    # the table has the rows of its blocks: with fancy-table, the rows after
    # the last marker are dropped (as the original, which sizes it by ms)
    m = Int(sum(ms))
    ns_head = something(options.ns_head, zeros(Int, length(ms) ÷ 2 - 1))
    options.table_width === nothing || (widths = auto_width(grid, m, widths, options.table_width))
    specs = [ColSpec(; alignment = a, width = w === nothing ? ColWidthDefault() : ColWidth(w))
             for (a, w) in zip(aligns(options.alignment, n), widths)]
    align_lines = lines(strip(options.alignment_cells))
    cell_aligns = [aligns(i <= length(align_lines) ? align_lines[i] : "", n) for i in 1:m]

    anchors = [(i, j) for i in 1:m for j in 1:n if is_at(grid, i, j)]
    nblocks = length(ms) ÷ 2 + 1
    if md !== nothing
        # every text of the table in one call: captions, cells and their
        # attributes, rows' and blocks' attributes
        texts = String[options.caption; options.short_caption; [grid.texts[a...] for a in anchors];
                       [md.cell_attrs[a...] for a in anchors]; md.row_attrs[1:m]; something(md.block_attrs, fill("", nblocks))]
        todo = filter(!isempty, texts)
        done = Iterators.Stateful(isempty(todo) ? Vector{Block}[] : read_markdown(todo, ctx))
        got = [isempty(t) ? Block[] : popfirst!(done) for t in texts]
        k = length(anchors)
        caption = Caption(got[1]; short = isempty(got[2]) ? nothing : got[2][1].content)
        contents = map(plain, got[3:2 + k])
        attrs = map(attr_of, got[3 + k:2 + 2k])
        row_attrs = map(attr_of, got[3 + 2k:2 + 2k + m])
        block_attrs = map(attr_of, got[3 + 2k + m:end])
    else
        caption = Caption(Para(Str(options.caption)); short = Inline[Str(options.short_caption)])
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
    Table(caption, specs, head, bodies, foot; attr)
end

run_filter(pantable)
