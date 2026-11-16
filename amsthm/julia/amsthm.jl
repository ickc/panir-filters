#!/usr/bin/env julia
# amsthm's theorem and proof environments for any output format: a pandoc
# filter.
#
# The metadata declares the environments (`amsthm: {plain: [Theorem:
# [Lemma]], definition: [Definition], ...}`, with options); a div of one's
# class is one. In LaTeX (and beamer) they are amsthm's own: `\newtheorem`s
# in `header-includes`, `\begin{...}...\end{...}` around each div, and
# citations of their identifiers `\ref`s. In any other format the filter
# numbers them as amsthm would (counters shared, within sections), sets
# their headings and bodies in their style's fonts, ends proofs with a
# symbol, and makes citations of them links.
#
# A div's `info` is its note, `Theorem 1 (note)`, which the original reads
# as markdown with pandoc (`pandoc.read`); this port reads it with
# LibPandoc.jl where its environment has it (pandocjl's, or one it is added
# to). Without it, a note of plain text (words of letters and digits, a
# hyphen inside a word, and `,;:?!` after one, single spaces between) is
# read as pandoc reads it; any other is kept as its text, with a warning.
#
# A port of ickc/pandoc-amsthm's amsthm.lua (Copyright (c) 2016-2026, Kolen
# Cheung; BSD-3-Clause, see ../LICENSE).

using Panir
# pandoc's reader, for the notes
Base.identify_package("LibPandoc") === nothing || import LibPandoc

# LaTeX's sectioning units, by depth: parent_counter's values.
const PARENT_COUNTERS = Dict("part" => 0, "chapter" => 1, "section" => 2, "subsection" => 3,
    "subsubsection" => 4, "paragraph" => 5, "subparagraph" => 6)
# Document classes whose level-1 headings pandoc's LaTeX writer makes chapters.
const CHAPTER_CLASSES = Set(["memoir", "book", "report", "scrreprt", "scrreport", "scrbook", "extreport", "extbook", "tufte-book"])
const FONTS = Dict("bold" => Strong, "italic" => Emph, "smallcaps" => SmallCaps)
const FONT_LATEX = Dict("bold" => "\\bfseries", "italic" => "\\itshape", "smallcaps" => "\\scshape")
# Keys of the metadata that are options, not styles.
const OPTION_KEYS = Set(["styles", "name_to_text", "parent_counter", "counter_depth", "counter_ignore_headings", "css", "swapnumbers", "qed_symbol"])
const LATEX_LIKE = Set(["latex", "beamer"])
# Formats whose header-includes are HTML.
const HTML_LIKE = Set(["html", "html4", "html5", "chunkedhtml", "epub", "epub2", "epub3", "revealjs", "slidy", "slideous", "dzslides", "s5"])
const CSS = "<style>\n:where(.amsthm-qed) { float: right; }\n</style>"
const Blank = Union{Space, SoftBreak, LineBreak}

warn(message) = println(stderr, "[amsthm] warning: ", message)

"A number as Lua's `tonumber` reads text, decimal or hexadecimal, blanks around it; `nothing` if not one."
function lua_number(text)
    t = strip(text, [' ', '\t', '\n', '\v', '\f', '\r'])
    occursin(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$", t) && return parse(Float64, t)
    m = match(r"^([+-]?)0[xX]([0-9a-fA-F]+)$", t)
    m === nothing ? nothing : (m[1] == "-" ? -1 : 1) * parse(Int, m[2]; base = 16)
end

# A note of plain text: words of letters and digits (anything not ASCII but a
# space), a hyphen inside one, and one of `,;:?!` after one, single spaces
# between. pandoc's markdown reader gives such a text as it is, a Str a word.
letter(c) = isascii(c) ? (isletter(c) || isdigit(c)) : !(c in ('\u85', '\ua0', ' ', ' ', ' ', ' ', ' ', '　', '﻿') || ' ' <= c <= ' ')
is_plain(note) = all(split(note, ' ')) do word
    word = endswith(word, r"[,;:?!]") ? chop(word) : word
    all(part -> !isempty(part) && all(letter, part), split(word, '-'))
end

"A div's note, as the original reads it with pandoc's markdown reader where it is plain text; otherwise its text, with a warning."
function note_inlines(info)
    if @isdefined LibPandoc
        out = Inline[]
        for block in LibPandoc.read(info, "markdown").blocks
            # (the original puts the block among inlines, and pandoc fails)
            block isa Union{Para, Plain} || error("the note $(repr(info)) is not inlines: a $(typeof(block))")
            append!(out, block.content)
        end
        return out
    end
    is_plain(info) || warn("the note $(repr(info)) has markup, which needs pandoc's markdown reader: kept as text")
    inlines(info)
end

const Unsafe = Union{Math, Cite, Image, RawInline}

"Whether a note holds math, a citation, an image or raw TeX, where a `]` would end LaTeX's optional argument: the original braces it then."
function needs_brace(note)
    found = false
    unsafe(::Unsafe) = (found = true; nothing)
    for x in note
        x isa Unsafe && return true
        walk!(unsafe, x)
    end
    found
end

# ---------------------------------------------------------------------------
# Fonts: an element of one around each Str, and what that does to nesting

"""In LaTeX, the font of the text around a reference applies to what `\\ref`
and an in-text citation print; `\\eqref` is upright, which undoes a shape
(italic, small caps) but not bold."""
function applies_to(::Type{T}, x::Cite) where {T}
    mode = length(x.citations) > 1 ? NormalCitation : length(x.citations) == 1 ? x.citations[1].mode : nothing
    mode == AuthorInText || (T === Strong && mode == NormalCitation)
end
applies_to(::Type{T}, x::RawInline) where {T} =
    x.format == "tex" && (occursin(r"^\\ref\{.*\}$"s, x.text) || (T === Strong && occursin(r"^\\eqref\{.*\}$"s, x.text)))

"Joins elements of the font next to each other, or with one Space between."
function merge_consecutive!(::Type{T}, content) where {T}
    out = Inline[]
    run = nothing
    for (i, x) in enumerate(content)
        if x isa T
            run = run === nothing ? copy(x.content) : append!(run, x.content)
        elseif run !== nothing && x isa Space && i < length(content) && content[i + 1] isa T
            push!(run, Space())
        else
            run === nothing || push!(out, T(run))
            run = nothing
            push!(out, x)
        end
    end
    run === nothing || push!(out, T(run))
    length(out) == length(content) || (empty!(content); append!(content, out))
    nothing
end

"""Each Str in the font; an element of the font in it undone for emphasis
(`\\emph` toggles), flattened for the others (bold in bold is bold); the
references the font applies to in it; then neighbouring elements of the
font in a block joined."""
struct Wrap{T <: Inline} end
(::Wrap{T})(x::Str) where {T} = T(x)
(::Wrap{T})(x::Union{Cite, RawInline}) where {T} = applies_to(T, x) ? T(x) : nothing
function (::Wrap{T})(x::T) where {T}
    if T === Emph
        return Inline[c for child in x.content for c in (child isa T ? child.content : child isa Blank ? [child] : [T(child)])]
    end
    x.content = Inline[c for child in x.content for c in (child isa T ? child.content : [child])]
    nothing
end
(::Wrap{T})(x::Union{Para, Plain, Header}) where {T} = merge_consecutive!(T, x.content)

# ---------------------------------------------------------------------------
# The environments

Base.@kwdef struct Style
    headfont::Vector{String}
    bodyfont::Vector{String}
    headpunct::String = "."
    headspace::String = " "  # " ", "newline", or a length (LaTeX)
    above::String = ""  # LaTeX only, as below and indent
    below::String = ""
    indent::String = ""
end

const BUILTIN_STYLES = Dict(
    "plain" => Style(; headfont = ["bold"], bodyfont = ["italic"]),
    "definition" => Style(; headfont = ["bold"], bodyfont = String[]),
    "remark" => Style(; headfont = ["italic"], bodyfont = String[]),
)

mutable struct Theorem
    style::String
    env_name::String
    text::String
    parent_counter::Union{String, Nothing}
    shared_counter::Union{String, Nothing}
    numbered::Bool
    proof::Bool
end

function Theorem(style, name, text; parent_counter = nothing, shared_counter = nothing, proof = false)
    numbered = !endswith(name, "*")
    env_name = numbered ? name : chop(name)
    if parent_counter !== nothing && !haskey(PARENT_COUNTERS, parent_counter)
        warn("unsupported parent_counter $parent_counter, ignoring")
        parent_counter = nothing
    end
    numbered && parent_counter !== nothing && (shared_counter = nothing)
    Theorem(style, env_name, isempty(text) ? env_name : text, parent_counter, shared_counter, numbered, proof)
end

function latex(t::Theorem)
    env, text = t.env_name, t.text
    !t.numbered && return "\\newtheorem*{$env}{$text}"
    t.shared_counter !== nothing && return "\\newtheorem{$env}[$(t.shared_counter)]{$text}"
    "\\newtheorem{$env}{$text}" * (t.parent_counter === nothing ? "" : "[$(t.parent_counter)]")
end
class_name(t::Theorem) = replace(t.env_name, ' ' => '_')
counter_name(t::Theorem) = something(t.shared_counter, t.env_name)

"The heading levels in a counter's number, `first` to `depth`; a heading at one of them restarts it."
Base.@kwdef mutable struct Counter
    first::Int = 1
    depth::Float64 = 0
end

"""The writer's options the original reads (`PANDOC_WRITER_OPTIONS`): the
conversion's options (defaults-file keys) where a host running the filter in
process knows them, else its `writer_options`, which libpandoc gives every
filter it runs (Lua's values under the same keys, but `top-level-chapter`
for `chapter`); plain pandoc gives a JSON filter neither."""
struct Writer
    number_sections::Bool
    top_level_division::String
    number_offset::Vector{Int}
    secnumdepth::Union{String, Nothing}  # (a variable, -V)
end

function Writer(conversion::Conversion)
    o = something(conversion.options, conversion.writer_options, Dict{String, Any}())
    depth = get(something(get(o, "variables", nothing), Dict{String, Any}()), "secnumdepth", nothing)
    division = chopprefix(string(get(o, "top-level-division", "default")), "top-level-")
    Writer(get(o, "number-sections", false) === true, division,
        Int.(something(get(o, "number-offset", nothing), Int[])), depth === nothing ? nothing : string(depth))
end

Base.@kwdef mutable struct Options
    css::Bool
    styles::Dict{String, Style}
    user_styles::Vector{String}
    swapnumbers::Bool
    qed_symbol::String
    counters::Dict{String, Counter}
    numbered_depth::Float64
    part_level::Union{Int, Nothing}
    proof_name::Union{String, Nothing}
    theorems::Vector{Theorem}  # in order
    by_class::Dict{String, Theorem}
    counter_depth::Float64
    ignored_headings::Set{String}
    header_counters::Dict{Int, Int}
    theorem_counters::Dict{String, Int} = Dict()
    identifiers::Dict{String, String} = Dict()  # (LaTeX: "")
    names::Dict{String, String} = Dict()
    unnumbered::Set{String} = Set()
    warned::Set{String} = Set()
end

# A map's entries, as Lua's `pairs`: a list's are its indices, which no name
# matches; text or a boolean fails, as in Lua.
lua_pairs(v::MetaMap, _) = pairs(v.content)
lua_pairs(::Union{MetaList, MetaInlines, MetaBlocks}, _) = Pair{String, MetaValue}[]
lua_pairs(v, what) = error("amsthm: $what is $(nameof(typeof(v))), not a map")

function read_font(value, place)
    font = String[]
    for v in items(value)
        name = stringify(v)
        if haskey(FONTS, name)
            push!(font, name)
        elseif name != "normal"
            warn("unknown font $name in $place; use bold, italic, smallcaps or normal")
        end
    end
    font
end

"""amsthm's styles, and those under `styles` (`\\newtheoremstyle`'s
arguments; what is left out as for a theorem), with the names of the user's,
sorted."""
function read_styles(value)
    styles = copy(BUILTIN_STYLES)
    value isa MetaMap || return styles, String[]
    kept = String[]
    for name in sort(collect(keys(value.content)))
        spec = value.content[name]
        if name in OPTION_KEYS
            warn("a style cannot be called $name, which is an option of its own; ignoring it")
        elseif name == "proof"
            warn("a style cannot be called proof, which lists the proof environments; ignoring it")
        elseif !(spec isa MetaMap)
            warn("style $name is not a map of its settings; ignoring it")
        else
            s = spec.content
            setting(key, default) = haskey(s, key) ? stringify(s[key]) : default
            place = "style $name"
            styles[name] = Style(haskey(s, "headfont") ? read_font(s["headfont"], place) : ["bold"],
                read_font(get(s, "bodyfont", nothing), place), setting("headpunct", "."), setting("headspace", " "),
                setting("above", ""), setting("below", ""), setting("indent", ""))
            push!(kept, name)
        end
    end
    styles, kept
end

"The LaTeX unit that level-1 headings are, as pandoc decides it."
function top_level_division(meta, writer)
    haskey(PARENT_COUNTERS, writer.top_level_division) && return writer.top_level_division
    class = get(meta, "documentclass", nothing)
    truthy(class) && stringify(class) in CHAPTER_CLASSES ? "chapter" : "section"
end

"How many heading levels step their LaTeX counters: none without -N, else as deep as secnumdepth."
function numbered_depth(meta, writer, top)
    writer.number_sections || return 0.0
    value = get(meta, "secnumdepth", nothing)
    # (false: as if unset, but not the variable)
    text = value === nothing ? writer.secnumdepth : truthy(value) ? stringify(value) : nothing
    depth = text === nothing ? nothing : lua_number(text)
    something(depth, 5) - PARENT_COUNTERS[top] + 2
end

function read_options(meta, writer)
    config = get(meta, "amsthm", nothing)
    # (an empty `amsthm:` is text, not a map: it defines nothing)
    opt = config isa MetaMap ? config.content : Dict{String, MetaValue}()

    name_to_text = Dict{String, String}(k => stringify(v) for (k, v) in lua_pairs(get(opt, "name_to_text", MetaMap()), "name_to_text"))
    # one unit for every environment, or a map of them by name
    pc = get(opt, "parent_counter", nothing)
    parent_counters = pc isa MetaMap ? Dict{String, String}(k => stringify(v) for (k, v) in pc.content) : Dict{String, String}()
    parent_counter = pc === nothing || pc isa MetaMap ? nothing : stringify(pc)
    parent_of(name) = get(parent_counters, name, parent_counter)

    theorems = Theorem[]
    styles, user_styles = read_styles(get(opt, "styles", nothing))
    for style in ["plain", "definition", "remark", filter(s -> !haskey(BUILTIN_STYLES, s), user_styles)...]
        for entry in items(get(opt, style, nothing))
            if entry isa MetaMap
                # a name, and the names sharing its counter (YAML's keys come
                # sorted, and so are they here)
                for key in sort(collect(keys(entry.content)))
                    push!(theorems, Theorem(style, key, get(name_to_text, key, ""); parent_counter = parent_of(key)))
                    value = entry.content[key]
                    for v in (value isa MetaList ? value.content : [value])
                        name = stringify(v)
                        push!(theorems, Theorem(style, name, get(name_to_text, name, ""); shared_counter = key))
                    end
                end
            else
                name = stringify(entry)
                push!(theorems, Theorem(style, name, get(name_to_text, name, ""); parent_counter = parent_of(name)))
            end
        end
    end
    # (`name_to_text: {proof: ""}` is an empty name, which a theorem's text
    # takes as none: the proof's is then its environment's, "proof")
    push!(theorems, Theorem("proof", "proof*", get(name_to_text, "proof", "Proof"); proof = true))
    by_class = Dict(class_name(t) => t for t in theorems)

    top = top_level_division(meta, writer)
    # with parts at level 1: a part steps no other counter, and is in a
    # theorem's number only when theorems are numbered within parts
    part_level = top == "part" ? 1 : nothing
    depth = get(opt, "counter_depth", nothing)
    explicit = truthy(depth) ? lua_number(stringify(depth)) : nothing
    counters = Dict{String, Counter}()
    counter_depth = something(explicit, 0.0)
    for t in theorems
        (t.numbered && t.shared_counter === nothing) || continue
        c = Counter(; depth = something(explicit, 0.0))
        if explicit === nothing && t.parent_counter !== nothing
            # within the heading level that LaTeX would number within
            c.depth = max(0, PARENT_COUNTERS[t.parent_counter] - PARENT_COUNTERS[top] + 1)
            part_level !== nothing && t.parent_counter != "part" && (c.first = 2)
        end
        counters[counter_name(t)] = c
        counter_depth = max(counter_depth, c.depth)
    end
    for name in keys(parent_counters)
        t = get(by_class, replace(name, ' ' => '_'), nothing)
        if t !== nothing && t.shared_counter !== nothing
            warn("$name shares the counter of $(t.shared_counter), so its parent_counter is $(t.shared_counter)'s; ignoring the one given")
        end
    end

    ignore = get(opt, "counter_ignore_headings", nothing)
    ignored = Set{String}(stringify.(truthy(ignore) ? items(ignore) : MetaValue[]))
    # where pandoc starts numbering sections (--number-offset)
    offset = writer.number_offset
    header_counters = Dict{Int, Int}(i => (i <= length(offset) ? offset[i] : 0) for i in 1:floor(Int, counter_depth))

    # the end-of-proof symbol as TeX math: the text of its math, raw TeX and words
    qed_symbol = "\\Box"
    if haskey(opt, "qed_symbol")
        parts = String[]
        walk!(inlines_of(opt["qed_symbol"])) do x
            x isa Union{Math, RawInline, Str} && push!(parts, x.text)
            nothing
        end
        isempty(parts) || (qed_symbol = join(parts, " "))
    end

    Options(; css = !haskey(opt, "css") || stringify(opt["css"]) != "false", styles, user_styles,
        swapnumbers = haskey(opt, "swapnumbers") && stringify(opt["swapnumbers"]) == "true", qed_symbol, counters,
        numbered_depth = numbered_depth(meta, writer, top), part_level, proof_name = get(name_to_text, "proof", nothing),
        theorems, by_class, counter_depth, ignored_headings = ignored, header_counters)
end

function latex(o::Options)
    lines = String[]
    o.qed_symbol == "\\Box" || push!(lines, "\\renewcommand{\\qedsymbol}{\\ensuremath{$(o.qed_symbol)}}")
    # \newtheoremstyle{name}{above}{below}{body font}{indent}{heading font}
    #   {punctuation}{space after the heading}{heading layout}
    # (\normalfont for none: an empty argument is the font around it)
    fonts(font) = isempty(font) ? "\\normalfont" : join(FONT_LATEX[f] for f in font)
    for name in o.user_styles
        s = o.styles[name]
        space = s.headspace == "newline" ? "\\newline" : s.headspace
        push!(lines, "\\newtheoremstyle{$name}{$(s.above)}{$(s.below)}{$(fonts(s.bodyfont))}{$(s.indent)}{$(fonts(s.headfont))}{$(s.headpunct)}{$space}{}")
    end
    o.swapnumbers && push!(lines, "\\swapnumbers")  # before \newtheorem, which takes the order from it
    # at the start of the document, after babel names the proof
    o.proof_name === nothing || push!(lines, "\\AtBeginDocument{\\renewcommand{\\proofname}{$(o.proof_name)}}")
    style = ""
    for t in o.theorems
        t.proof && continue
        t.style == style || push!(lines, "\\theoremstyle{$(style = t.style)}")
        push!(lines, latex(t))
    end
    join(lines, "\n")
end

"A block in header-includes, whatever shape it has."
function add_header_include!(meta, block; at_start = false)
    existing = items(get(meta, "header-includes", nothing))
    added = MetaBlocks(block)
    meta["header-includes"] = MetaList(at_start ? [added; existing] : [existing; added])
end

# ---------------------------------------------------------------------------
# The two passes

"The environment a div is, by its classes; none for one naming two."
function find_theorem(o, classes; warn_two = false)
    found = nothing
    others = String[]
    for c in classes
        t = get(o.by_class, c, nothing)
        (t === nothing || t === found) && continue
        if found === nothing
            found = t
        elseif !(c in others)
            push!(others, c)
        end
    end
    if !isempty(others)
        warn_two && warn("multiple environments found: $(class_name(found)), $(join(others, ", ")); skipping")
        return nothing
    end
    found
end

"""The heading, as amsthm's `\\thmhead` writes it: the name and the number in
the style's heading font, the number upright, the note in parentheses in the
note font, and the punctuation; then the space after the heading."""
function header(o, theorem, identifier, info)
    if theorem.proof
        # "Proof.", in italics; or the note, in italics but its references,
        # for the whole heading
        (info === nothing || isempty(info)) && return Inline[Emph(inlines(theorem.text * ".")), Space()]
        # each Str in italics, emphasis in it upright, references as the font
        # applies to them; then neighbouring italics joined (in the paragraph
        # itself: a walk is of what is in it)
        para = walk!(Wrap{Emph}(), Para(note_inlines(info)))
        merge_consecutive!(Emph, para.content)
        return Inline[para.content; Emph(Str(".")); Space()]
    end
    s = o.styles[theorem.style]
    head = s.headfont
    number = nothing
    if theorem.numbered
        name = counter_name(theorem)
        o.theorem_counters[name] = get(o.theorem_counters, name, 0) + 1
        c = get(o.counters, name, Counter())
        parts = map(c.first:floor(Int, c.depth)) do level
            n = o.header_counters[level]
            # \thepart is a Roman numeral, and empty before the first part
            level == o.part_level ? (n > 0 ? to_roman_numeral(n) : "") : string(n)
        end
        number = join([parts; string(o.theorem_counters[name])], ".")
        if !isempty(identifier)
            o.identifiers[identifier] = number
            o.names[identifier] = theorem.text
        end
    end

    runs = Pair{Vector{Inline}, Vector{String}}[inlines(theorem.text) => head]
    if number !== nothing && o.swapnumbers
        # \swappedhead: the number first, in the heading font, then a tie
        pushfirst!(runs, Inline[Str(number), Str(" ")] => head)
    elseif number !== nothing
        push!(runs, Inline[Space(), Str(number)] => filter(==("bold"), head))  # upright
    end
    if info !== nothing && !isempty(info)
        push!(runs, Inline[Space(); Str("("); note_inlines(info); Str(")")] => String[])
    end
    push!(runs, Inline[Str(s.headpunct)] => head)
    Inline[join_runs(runs); s.headspace == "newline" ? LineBreak() : Space()]
end

in_font(content, font) = foldr((f, inner) -> Inline[FONTS[f](inner)], font; init = content)

"""Runs of inlines, each with its font, each stretch of one font in its
elements once ("Theorem 1." one Strong, not three)."""
function join_runs(runs)
    out = Inline[]
    content, font = Inline[], nothing
    for (run, run_font) in runs
        if run_font != font
            font === nothing || append!(out, in_font(content, font))
            content, font = Inline[], run_font
        end
        append!(content, run)
    end
    font === nothing || append!(out, in_font(content, font))
    out
end

"`name`, a no-break space, then `number`: \"Theorem 1\"."
named(name, number) = Inline[inlines(name); Str(" "); number]

"The div's content in its style's body font; environments in it are held out, as each has a style of its own."
function style_body!(div, o, bodyfont)
    held = Div[]
    function hold(d::Div, ctx)
        find_theorem(o, d.attr.classes) === nothing && return nothing
        push!(held, d)
        skip_children!(ctx)
        Div(Block[]; attributes = ["amsthm-held" => string(length(held))])
    end
    walk!(hold, div.content; traverse = :topdown)
    for font in bodyfont
        walk!(Wrap{FONTS[font]}(), div.content)
    end
    isempty(held) && return
    walk!(div.content) do d
        d isa Div || return nothing
        i = lua_number(something(get(d.attr, "amsthm-held", nothing), ""))
        # (any div with the attribute, the user's too: a copy each, as each
        # return is in Lua)
        i !== nothing && isinteger(i) && 1 <= i <= length(held) ? deepcopy(held[Int(i)]) : nothing
    end
end

"amsthm's \\mathqed, \\quad\\qedsymbol (a space between where the symbol starts with a letter)."
qed_math(o) = "\\quad" * (occursin(r"^[A-Za-z]", o.qed_symbol) ? " " : "") * o.qed_symbol
qed_span(o) = Span(Math(InlineMath, qed_math(o)); classes = ["amsthm-qed"])

is_qedhere(x) = x isa RawInline && x.format in ("tex", "latex") && occursin(r"^[ \t\n\v\f\r]*\\qedhere[ \t\n\v\f\r]*$", x.text)

"""amsthm's \\qedhere: the symbol where it is, \\mathqed in math, \\qed (after
the spaces before it) in text; environments in this one keep theirs.
Whether there was one."""
function place_qedhere!(div, o)
    placed = false
    function qedhere(d::Div, ctx)
        find_theorem(o, d.attr.classes) === nothing || skip_children!(ctx)
        nothing
    end
    function qedhere(m::Math)
        text = replace(m.text, r"\\qedhere(?![A-Za-z])" => qed_math(o))
        text == m.text || (placed = true; m.text = text)
        nothing
    end
    function qedhere(xs::Vector{Inline})
        any(is_qedhere, xs) || return nothing
        out = Inline[]
        for x in xs
            if is_qedhere(x)
                while !isempty(out) && out[end] isa Blank  # (\qed starts with \unskip)
                    pop!(out)
                end
                push!(out, qed_span(o))
                placed = true
            else
                push!(out, x)
            end
        end
        out
    end
    walk!(qedhere, div.content; traverse = :topdown)
    placed
end

with_inlines(block) = block isa Union{Para, Plain, Header} ? block.content : nothing

"Not LaTeX: the heading, the body's font, the end of a proof."
function environment!(div, o)
    theorem = find_theorem(o, div.attr.classes; warn_two = true)
    theorem === nothing && return nothing
    identifier = div.attr.identifier
    !theorem.numbered && !isempty(identifier) && push!(o.unnumbered, identifier)
    title = header(o, theorem, identifier, get(div.attr, "info", nothing))
    space = pop!(title)
    head = Inline[Span(title; classes = ["amsthm-title"]), space]

    bodyfont = haskey(o.styles, theorem.style) ? o.styles[theorem.style].bodyfont : String[]
    isempty(bodyfont) || style_body!(div, o, bodyfont)
    first = isempty(div.content) ? nothing : with_inlines(div.content[1])
    first === nothing ? pushfirst!(div.content, Para(head)) : prepend!(first, head)

    if theorem.proof && !place_qedhere!(div, o)
        # amsthm's \qed, \nobreak\hfill\quad\openbox: as math, \quad\Box (the
        # \hfill is the CSS's)
        last = isempty(div.content) ? nothing : with_inlines(div.content[end])
        last === nothing ? push!(div.content, Para(qed_span(o))) : push!(last, qed_span(o))
    end
    push!(div.attr.classes, "amsthm")
    "amsthm-$(theorem.style)" in div.attr.classes || push!(div.attr.classes, "amsthm-$(theorem.style)")
    div
end

"The environment a citation's id names, and whether to name it: `@Euler` is `@euler`, the environment's name before the number."
function reference_target(id, known)
    id in known && return (id, false)
    lower = replace(id, r"^[A-Z]" => lowercase)
    lower != id && lower in known ? (lower, true) : nothing
end

"The ids of `[@a; @b]`, each an environment's, none with a prefix or suffix (which the list would drop); else `nothing`, for citeproc."
function multi_ref(cite, known)
    length(cite.citations) < 2 && return nothing
    all(c -> c.mode == NormalCitation && haskey(known, c.id) && isempty(c.prefix) && isempty(c.suffix), cite.citations) || return nothing
    [c.id for c in cite.citations]
end

ref_list(ids, one) = Inline[x for (i, id) in enumerate(ids) for x in [(i > 1 ? Inline[Str(","), Space()] : Inline[]); one(id)]]

const RAW_REF = r"^\\([A-Za-z]+)\{(.*)\}$"s

"Once per id, a reference to an unnumbered environment; and a citation of several that is left to citeproc."
function warn_unnumbered(x, o)
    ids = if x isa Cite
        [c.id for c in x.citations]
    elseif x.format == "tex" && (m = match(RAW_REF, x.text)) !== nothing && m[1] in ("ref", "eqref")
        [String(m[2])]
    else
        String[]
    end
    for id in ids
        target = reference_target(id, o.unnumbered)
        if target !== nothing && !(target[1] in o.warned)
            push!(o.warned, target[1])
            warn("reference to the unnumbered environment $(target[1]); LaTeX prints the last number before it, other output leaves it unresolved")
        end
    end
    (x isa Cite && length(x.citations) > 1) || return
    ours = others = named_ = affixed = false
    for c in x.citations
        affixed |= !isempty(c.prefix) || !isempty(c.suffix)
        if haskey(o.identifiers, c.id) || c.id in o.unnumbered
            ours = true
        elseif reference_target(c.id, keys(o.identifiers)) !== nothing
            named_ = true
        else
            others = true
        end
    end
    if named_
        warn("a reference to several environments cannot name them, and is left to citeproc: $(stringify(x))")
    elseif ours && others
        warn("a citation mixes environments with other keys, and is left to citeproc: $(stringify(x))")
    elseif ours && affixed
        warn("a reference to several environments cannot carry a prefix or suffix, and is left to citeproc: $(stringify(x))")
    end
end

parenthesised(out, paren) = paren ? Inline[Str("("); out; Str(")")] : out

"The number, a link to the environment, perhaps led by its name, in parentheses with `paren`."
function ref_link(id, number, paren, name = nothing)
    link = Link(Str(number); url = "#$id")
    parenthesised(name === nothing ? Inline[link] : named(name, link), paren)
end

"Not LaTeX: `[@id]`, `@id`, `\\ref{id}`, `\\eqref{id}` to numbers."
function resolve(x::Cite, o)
    known = o.identifiers
    ids = multi_ref(x, known)
    ids === nothing || return ref_list(ids, i -> ref_link(i, known[i], true))
    length(x.citations) == 1 || return nothing
    c = x.citations[1]
    target = reference_target(c.id, keys(known))
    target === nothing && return nothing
    id, is_named = target
    name = is_named ? get(o.names, id, nothing) : nothing
    c.mode in (NormalCitation, AuthorInText) ? ref_link(id, known[id], c.mode == NormalCitation, name) : nothing
end
function resolve(x::RawInline, o)
    m = x.format == "tex" ? match(RAW_REF, x.text) : nothing
    (m !== nothing && m[1] in ("ref", "eqref") && haskey(o.identifiers, m[2])) || return nothing
    ref_link(m[2], o.identifiers[m[2]], m[1] == "eqref")
end

"LaTeX: a citation of an environment to `\\ref` or `\\eqref`, perhaps led by its name (`Theorem~\\ref{euler}`)."
function cite_to_ref(cite, o)
    known = o.identifiers
    ids = multi_ref(cite, known)
    ids === nothing || return ref_list(ids, i -> Inline[RawInline("latex", "\\eqref{$i}")])
    length(cite.citations) == 1 || return nothing
    c = cite.citations[1]
    c.mode in (NormalCitation, AuthorInText) || return nothing
    paren = c.mode == NormalCitation
    target = reference_target(c.id, keys(known))
    target === nothing && return nothing
    id, is_named = target
    name = is_named ? get(o.names, id, nothing) : nothing
    name === nothing && return RawInline("latex", "\\$(paren ? "eqref" : "ref"){$id}")
    parenthesised(named(name, RawInline("latex", "\\ref{$id}")), paren)
end

"LaTeX: the div's own blocks between `\\begin{...}` and `\\end{...}`, sharing a line with the first and last paragraphs, in a bare div."
function latex_environment(div, o)
    theorem = find_theorem(o, div.attr.classes)
    theorem === nothing && return nothing
    opening = Inline[RawInline("latex", "\\begin{$(theorem.env_name)}")]
    info = get(div.attr, "info", nothing)
    if info !== nothing && !isempty(info)
        note = note_inlines(info)
        brace = needs_brace(note)
        append!(opening, [RawInline("latex", brace ? "[{" : "["); note; RawInline("latex", brace ? "}]" : "]")])
    end
    isempty(div.attr.identifier) || push!(opening, RawInline("latex", "\\label{$(div.attr.identifier)}"))
    closing = RawInline("latex", "\\end{$(theorem.env_name)}")
    content = div.content
    if !isempty(content) && content[1] isa Union{Para, Plain}
        prepend!(content[1].content, [opening; RawInline("latex", "\n")])
    else
        pushfirst!(content, Plain(opening))
    end
    if content[end] isa Union{Para, Plain}
        append!(content[end].content, [RawInline("latex", "\n"), closing])
    else
        push!(content, Plain(closing))
    end
    Div(content)
end

const state = (options = Ref{Options}(), latex = Ref(false))

function first_pass(doc::Pandoc, ctx)
    format = something(ctx.format, "")
    state.latex[] = format in LATEX_LIKE
    o = state.options[] = read_options(doc.meta, Writer(ctx.conversion))
    if any(c.depth >= c.first && o.numbered_depth < c.first for c in values(o.counters))
        warn("theorems are numbered within sections that are not numbered, so their numbers start with 0 as in LaTeX; pass -N (--number-sections)")
    end
    if state.latex[]
        # amsthm first, for the user's own \newtheoremstyle; the environments
        # last, to use those styles
        add_header_include!(doc.meta, RawBlock("latex", "\\usepackage{amsthm}"); at_start = true)
        add_header_include!(doc.meta, RawBlock("latex", latex(o)))
    elseif format in HTML_LIKE && o.css
        add_header_include!(doc.meta, RawBlock("html", CSS))
    end
    nothing
end

# \section* and the like step no counter, nor do headings LaTeX doesn't number
function first_pass(h::Header)
    o = state.options[]
    (state.latex[] || h.level > o.counter_depth || h.level > o.numbered_depth || "unnumbered" in h.attr.classes) && return
    stringify(h) in o.ignored_headings && return
    o.header_counters[h.level] = get(o.header_counters, h.level, 0) + 1
    if h.level != o.part_level
        for level in h.level+1:floor(Int, o.counter_depth)
            o.header_counters[level] = 0
        end
    end
    # restart each counter numbered within this level or above it
    for (name, c) in o.counters
        c.first <= h.level <= c.depth && delete!(o.theorem_counters, name)
    end
end

function first_pass(d::Div)
    o = state.options[]
    state.latex[] || return environment!(d, o)
    # the ids, for the references before their environments
    theorem = find_theorem(o, d.attr.classes; warn_two = true)
    if theorem !== nothing && !isempty(d.attr.identifier)
        o.identifiers[d.attr.identifier] = ""
        o.names[d.attr.identifier] = theorem.text
        theorem.numbered || push!(o.unnumbered, d.attr.identifier)
    end
    nothing
end

second_pass(d::Div) = state.latex[] ? latex_environment(d, state.options[]) : nothing
function second_pass(x::Union{Cite, RawInline})
    warn_unnumbered(x, state.options[])
    if state.latex[]
        x isa Cite ? cite_to_ref(x, state.options[]) : nothing
    else
        resolve(x, state.options[])
    end
end

run_filter([first_pass => :topdown, second_pass => :topdown])
