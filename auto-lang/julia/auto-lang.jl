#!/usr/bin/env julia
# Spans with `lang` (and `dir`) around runs of a mapped Unicode script: a
# pandoc filter.
#
# The metadata maps scripts to languages (`auto-lang: {Hebrew: he, Han:
# zh-Hant}`); each run of text in a mapped script becomes a span with the
# language, and `dir=rtl` for a right-to-left script. Spaces, digits and
# punctuation join a run that goes on after them; a character that
# Script_Extensions ties to the run's script (`。` to Han) joins a run it
# touches; a combining mark goes with the character before it. A span that
# has a `lang` already is left alone, and what is in it.
#
# A port of ickc/font's bin/auto-lang.lua (Copyright (c) 2026, Kolen Cheung;
# BSD-3-Clause, see ../LICENSE), with its table of Unicode 15.0.0's scripts
# (`script-ranges.jl`, made by ../tables.py from the original's).

using Panir

include("script-ranges.jl")

const languages = Dict{String, String}()  # a mapped script's language

within(ranges, cp) = (i = searchsortedlast(ranges, (cp, typemax(UInt32))); i > 0 && cp <= ranges[i][2])

const STRONG_FIRSTS = first.(STRONG)

"The script of a code point's own Script property, if a script."
function own_script(cp)
    i = searchsortedlast(STRONG_FIRSTS, cp)
    i > 0 && cp <= STRONG[i][2] ? STRONG[i][3] : nothing
end

# What a code point is to a run: of a script (a `String`, its own, mapped or
# not), tied by Script_Extensions to some mapped scripts (a `Set`),
# `:inherited` (a mark, going with what precedes it), or `nothing`: neutral
# (spaces, digits, punctuation).
const memo = Dict{Char, Union{String, Set{String}, Symbol, Nothing}}()

# Its own script first if mapped; then the ties, even of a code point of an
# unmapped script (the Arabic-Indic digits continue a Thaana run).
classify(ch::Char) = get!(memo, ch) do
    cp = UInt32(ch)
    own = own_script(cp)
    own !== nothing && haskey(languages, own) && return own
    tied = Set(s for s in keys(languages) if haskey(EXT, s) && within(EXT[s], cp))
    !isempty(tied) ? tied : own !== nothing ? own : within(INHERITED, cp) ? :inherited : nothing
end

mapped(c) = c isa String && haskey(languages, c)

mutable struct Piece
    text::String
    script::Union{String, Nothing}  # a run of a mapped script
    tied::Union{Set{String}, Nothing}
    foreign::Bool  # another script's own text
end
Piece(text; script = nothing, tied = nothing, foreign = false) = Piece(text, script, tied, foreign)

tied_to(script, ties) = ties !== nothing && script in ties
"How many of the ties, from the start, are to `script`."
leading_tied(script, ties) = something(findfirst(t -> !tied_to(script, t), ties), length(ties) + 1) - 1
"Where the ties to `script` at the end start."
trailing_tied(script, ties) = something(findlast(t -> !tied_to(script, t), ties), 0) + 1

"""A text as maximal runs of one mapped script, and what is between them:
neutral text joins a run only if the run goes on after it; text tied to a
run's script joins the run it touches. Tied text stays in pieces of one
character (with its marks): it may belong to a run in a neighbouring
inline, which only `group` sees."""
function split_runs(text)
    pieces = Piece[]
    run = nothing
    pending = Piece[]  # each character since the run last took one
    function close()
        run === nothing && return
        n = leading_tied(run.script, [p.tied for p in pending])
        run.text *= join(p.text for p in pending[1:n])
        deleteat!(pending, 1:n)
        push!(pieces, run)
        run = nothing
    end
    for ch in text
        c = classify(ch)
        if mapped(c)
            if run !== nothing && run.script == c
                run.text *= join(p.text for p in pending) * ch
                empty!(pending)
            else
                close()
                from = trailing_tied(c, [p.tied for p in pending])
                carried = join(p.text for p in pending[from:end])
                deleteat!(pending, from:length(pending))
                append!(pieces, pending)
                empty!(pending)
                run = Piece(carried * ch; script = c)
            end
        elseif c isa String
            # another script's own text ends the run, apart from the neutral
            # text around it
            close()
            append!(pieces, pending)
            empty!(pending)
            push!(pieces, Piece(string(ch); foreign = true))
        elseif c === :inherited
            # with its base: the last pending character, or the run
            if !isempty(pending)
                pending[end].text *= ch
            elseif run !== nothing
                run.text *= ch
            else
                push!(pending, Piece(string(ch)))
            end
        else
            push!(pending, Piece(string(ch); tied = c))
        end
    end
    close()
    append!(pieces, pending)

    merged = Piece[]
    for p in pieces
        prev = isempty(merged) ? nothing : merged[end]
        if prev !== nothing && prev.script === p.script === prev.tied === p.tied === nothing && prev.foreign == p.foreign
            prev.text *= p.text
        else
            push!(merged, p)
        end
    end
    merged
end

function span(script, inlines)
    attributes = ["lang" => languages[script]]
    script in RTL && push!(attributes, "dir" => "rtl")
    Span(inlines; attributes)
end

# Containers that carry nothing of their own: what is in them decides the run
# they belong to (a span with a `lang` aside). A note's content is a flow of
# its own; code, math and raw inlines are not prose.
const Container = Union{Emph, Link, Quoted, SmallCaps, Span, Strikeout, Strong, Subscript, Superscript, Underline}
transparent(x) = x isa Container && !(x isa Span && haskey(x.attr, "lang"))

"""Which run a list of inlines belongs to, from the text in it: its one
mapped script and no ties, or none and, if all it holds is tied text, the
scripts all of it is tied to; `nothing` if it is opaque: it holds another
script, or what isn't prose, or a span with a language."""
function list_script(inlines)
    found = nothing
    ties = nothing
    untied = false
    function scan(items)
        for x in items
            if x isa Str
                for ch in x.text
                    c = classify(ch)
                    if c isa String
                        (!mapped(c) || (found !== nothing && found != c)) && return false
                        found = c
                    elseif c isa Set
                        ties = ties === nothing ? copy(c) : intersect!(ties, c)
                    elseif c === nothing
                        untied = true
                    end  # (a mark goes with its base: it neither ties nor unties)
                end
            elseif x isa Union{Space, SoftBreak, LineBreak}
                untied = true
            elseif transparent(x)
                scan(x.content) || return false
            else
                return false
            end
        end
        true
    end
    scan(inlines) || return nothing
    found !== nothing && return (found, nothing)
    (nothing, untied || ties === nothing || isempty(ties) ? nothing : ties)
end

"""The list with its runs in spans, a run going on across spaces and the
containers of nothing but its script; `nothing` if there is none."""
function group(inlines)
    # A run with the whole list to itself takes the neutral text around it.
    whole = list_script(inlines)
    whole !== nothing && whole[1] !== nothing && return Inline[span(whole[1], inlines)]

    out = Inline[]
    run = Inline[]
    script = nothing
    pending = Tuple{Inline, Union{Set{String}, Nothing}}[]  # since the run last took one
    changed = false
    function close()
        if script !== nothing
            n = leading_tied(script, last.(pending))
            append!(run, first.(pending[1:n]))
            deleteat!(pending, 1:n)
            push!(out, span(script, run))
            run = Inline[]
            script, changed = nothing, true
        end
        append!(out, first.(pending))
        empty!(pending)
    end
    function add(x, of, tied)
        if of === nothing
            push!(pending, (x, tied))
        elseif of == script
            append!(run, first.(pending))
            empty!(pending)
            push!(run, x)
        else
            # the first run, or another script's: the tied text at the end of
            # what is pending is the new run's
            from = trailing_tied(of, last.(pending))
            carried = first.(pending[from:end])
            deleteat!(pending, from:length(pending))
            close()
            script = of
            append!(run, carried)
            push!(run, x)
        end
    end
    for x in inlines
        if x isa Union{Space, SoftBreak}
            add(x, nothing, nothing)
        elseif x isa Str
            for piece in split_runs(x.text)
                if piece.foreign
                    close()
                    push!(out, Str(piece.text))
                else
                    add(Str(piece.text), piece.script, piece.tied)
                end
            end
        elseif transparent(x) && (found = list_script(x.content)) !== nothing
            add(x, found...)
        else
            close()
            push!(out, x)
        end
    end
    close()
    changed ? out : nothing
end

"`auto-lang`, the scripts' languages. In a filter of its own, as a filter's metadata function runs after its inlines'."
function configure(meta::Panir.Meta)
    map = get(meta, "auto-lang", nothing)
    scripts = if map === nothing || map == MetaBool(false)
        Pair{String, Any}[]
    elseif map isa MetaMap
        [k => stringify(v) for (k, v) in map.content]
    elseif map isa Union{MetaList, MetaInlines, MetaBlocks}
        [string(i) => nothing for i in eachindex(map.content)]  # (Lua's `pairs` over a list: its indices)
    else
        error("auto-lang: the map of scripts to languages is $(nameof(typeof(map)))")
    end
    for (script, language) in scripts
        if script in NAMES && language !== nothing
            languages[script] = language
        else
            println(stderr, "[WARNING] auto-lang: no Unicode script named $script (Unicode $VERSION)")
        end
    end
    empty!(memo)
    nothing
end

# A language written by hand wins: the span and what is in it stay.
tag(span::Span, ctx) = (haskey(span.attr, "lang") && skip_children!(ctx); nothing)
tag(inlines::Vector{Inline}) = isempty(languages) ? nothing : group(inlines)

run_filter([configure, tag => :topdown])
