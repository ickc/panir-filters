#!/usr/bin/env julia
# Numbered paragraphs, and citations of them: a pandoc filter.
#
# Each paragraph of the body (not one opening with an image) gets a number,
# in the margin (LaTeX) or in a span before it, per chapter with
# `resetAtChapter`. A paragraph opening with `{#name}` can be cited as
# `[@name]`: the citation becomes a link to its number. `enclosing`,
# `chapterSep`, `refName` and `addPageNr` shape the numbers and the links;
# `header-includes` gets the CSS or the LaTeX packages.
#
# A port of cysouw/count-para (Copyright © 2021, 2023 Michael Cysouw; ISC;
# the repository's CC0-1.0, see ../LICENSE).

using Panir

# (as the original's, its spaces at the ends of lines too)
const CSS = " \n<!-- CSS added by lua-filter 'count-para' -->\n<style>\n.paragraph-number { \n" *
    "  float: left;\n  margin-left: -5em;\n  width: 4.5em;\n  text-align: right;\n  color: grey;\n" *
    "  font-size: x-small;\n  padding-top: 5px;\n}\n</style>\n    "

const LATEX = [
    raw"\usepackage{xcolor}",
    raw"\usepackage{marginnote}",
    raw"\reversemarginpar",
    raw"\newcommand{\paragraphnumber}[1]{\marginnote{\color{lightgray}\tiny{#1}}[0pt]}",
]

# the options
Base.@kwdef mutable struct Settings
    reset_at_chapter::Bool = false
    enclosing::String = "[]"
    chapter_sep::String = "."
    ref_name::String = "paragraph "
    add_page_nr::Bool = true
end
const settings = Settings()
const numbers = Dict{String, String}()  # by the names given

"The text's `i`th byte, as Lua's `sub` takes it and pandoc reads it back: an ASCII character, else U+FFFD (or nothing past the end)."
function byte(text, i)
    i > ncodeunits(text) && return ""
    b = codeunit(text, i)
    b < 0x80 ? string(Char(b)) : "\ufffd"
end

function formatting(meta::Panir.Meta, ctx)
    format = something(ctx.format, "")
    added = MetaValue[]
    occursin("html", format) && push!(added, MetaBlocks(RawBlock("html", CSS)))
    occursin("latex", format) && append!(added, [MetaBlocks(RawBlock("tex", tex)) for tex in LATEX])
    # The original appends to what is there as to a Lua table: a list grows, a
    # map's entries are lost (the table becomes a list of the new ones),
    # anything else fails; nothing there is an empty list, even with nothing
    # to add.
    existing = get(meta, "header-includes", nothing)
    if existing isa MetaList
        append!(existing.content, added)
    elseif existing === nothing || (existing isa MetaMap && !isempty(added))
        meta["header-includes"] = added
    elseif !isempty(added)
        error("count-para: header-includes is $(nameof(typeof(existing))), not a list")
    end
    nothing
end

function read_settings(meta::Panir.Meta, ctx)
    value(key) = get(meta, key, nothing)
    value("resetAtChapter") === nothing || (settings.reset_at_chapter = truthy(value("resetAtChapter")))  # (Lua's truth)
    value("enclosing") === nothing || (settings.enclosing = stringify(value("enclosing")))
    value("chapterSep") === nothing || (settings.chapter_sep = stringify(value("chapterSep")))
    if value("refName") !== nothing
        settings.ref_name = stringify(value("refName"))
        occursin("latex", something(ctx.format, "")) && settings.ref_name == "#" && (settings.ref_name = "\\#")
    end
    value("addPageNr") === nothing || (settings.add_page_nr = truthy(value("addPageNr")))
    nothing
end

function numbering(doc::Pandoc, ctx)
    latex = occursin("latex", something(ctx.format, ""))
    count = chapter = 0
    for block in doc.blocks  # (the body's own paragraphs only)
        if block isa Header && block.level == 1 && get(block.attr.classes, 1, nothing) != "unnumbered" && settings.reset_at_chapter
            chapter += 1
            count = 0
        end
        # (an empty paragraph fails, as in the original: it has no first inline)
        (block isa Para && !(block.content[1] isa Image)) || continue
        count += 1
        number = settings.reset_at_chapter ? "$chapter$(settings.chapter_sep)$count" : string(count)
        e = settings.enclosing
        shown = ncodeunits(e) == 1 ? "$e$number$e" : byte(e, 1) * number * byte(e, 2)

        # a name given at the start: anywhere in the first inline's text
        found = match(r"\{#(.*)\}"s, stringify(block.content[1]))
        name = found === nothing ? nothing : String(found[1])
        if name !== nothing
            numbers[name] = number
            popfirst!(block.content)
            block.content[1] isa Space && popfirst!(block.content)  # (nothing left fails, as in the original)
        end

        if latex  # after the first word, to keep the number on its page
            tex = "\\paragraphnumber{$shown}"
            name === nothing || (tex = "\\hypertarget{$name}{\n$tex\\label{$name}}")
            insert!(block.content, 2, RawInline("tex", tex))
        else
            prepend!(block.content, [Span(shown; identifier = number, classes = ["paragraph-number"]), Space()])
        end
    end
    nothing
end

function references(cite::Cite, ctx)
    name = cite.citations[1].id
    number = get(numbers, name, nothing)
    number === nothing && return nothing
    if occursin("latex", something(ctx.format, ""))
        page = settings.add_page_nr ? " on page~\\pageref{$name}" : ""
        return RawInline("tex", "$(settings.ref_name)\\hyperlink{$name}{$number}$page")
    end
    Link(settings.ref_name * number; url = "#$number")
end

# header-includes, the options, the numbers, then the citations (the
# metadata's too)
run_filter([formatting, read_settings, numbering, references])
