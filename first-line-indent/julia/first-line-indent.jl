#!/usr/bin/env julia
# First-line indents, removed after quotes, lists and the like: a pandoc
# filter.
#
# A paragraph right after a block quote, a list, a code block or a rule
# (`first-line-indent.remove-after`, `dont-remove-after`) gets `\noindent` in
# LaTeX, or a marker div before it in HTML, unless it starts with `\indent`
# or `\noindent` itself. The metadata's `indent` is set true, and
# `header-includes` gets the CSS (HTML) or the indent's `size` (LaTeX).
#
# A port of pandoc/lua-filters' first-line-indent (Copyright (c) 2021 Julien
# Dutant; MIT, see ../LICENSE).

using Panir

const CSS = """
  <style>
    p {
      text-indent: SIZE;
      margin: 0;
    }
    header + p {
      text-indent: ;
    }
    :is(h1, h2, h3, h4, h5, h6) + p {
      text-indent: 0;
    }
    :is(div.no-first-line-indent-after) + p {
      text-indent: 0;
    }
    :is(div.first-line-indent-after) + p {
      text-indent: SIZE;
    }
  </style>
"""

const DIVS = Dict(
    :indent => "<div class=\"first-line-indent-after\"></div>",
    :noindent => "<div class=\"no-first-line-indent-after\"></div>",
)

# read from the metadata by the first filter, used by the second
Base.@kwdef mutable struct Options
    set_metadata_variable::Bool = true
    set_header_includes::Bool = true
    auto_remove::Bool = true
    remove_after::Vector{String} = ["BlockQuote", "BulletList", "CodeBlock", "DefinitionList", "HorizontalRule", "OrderedList"]
    size::String = "1em"
end
const options = Options()

"`\\indent ` or `\\noindent `, as the markdown reader makes them."
command(kind) = RawInline("tex", "\\$kind ")

# Block types: text is one, a list's items each one; else none.
names(v::MetaInlines) = [stringify(v)]
names(v::MetaList) = map(stringify, v.content)
names(_) = nothing

# `text` as Lua's `gsub` reads a replacement, for a pattern without captures:
# `%0` and `%1` are the match, `%%` a percent sign, any other `%` an error.
# (So a size of `5%` fails, as in the original.)
function lua_replacement(text, whole)
    replace(text, r"%.?"s => function (escape)
        escape in ("%0", "%1") && return whole
        escape == "%%" && return "%"
        error("first-line-indent: invalid use of '%' in replacement string")
    end)
end

# A metadata value where a block goes, as Lua converts it: text a `Plain` of
# it, a map (a table) an empty `Plain`; blocks, a list or a boolean fail.
as_block(v::MetaInlines) = Plain(v.content)
as_block(v::MetaString) = Plain(v.text)
as_block(::MetaMap) = Plain()
as_block(v) = error("first-line-indent: $(nameof(typeof(v))) in header-includes, where a block goes")

function configure(meta::Panir.Meta, ctx)
    user = get(meta, "first-line-indent", nothing)
    user == MetaBool(true) && error("first-line-indent: a configuration of true")  # (Lua indexes a boolean)
    if truthy(user)
        is_false(key) = lookup(user, key) == MetaBool(false)
        is_false("set-metadata-variable") && (options.set_metadata_variable = false)
        is_false("set-header-includes") && (options.set_header_includes = false)
        is_false("auto-remove") && (options.auto_remove = false)
        append!(options.remove_after, something(names(lookup(user, "remove-after")), String[]))
        blacklist = names(lookup(user, "dont-remove-after"))
        blacklist === nothing || filter!(x -> !(x in blacklist), options.remove_after)
        size = lookup(user, "size")
        size === nothing || (options.size = stringify(size))
    end

    # The original changes the metadata, then returns it only if it set
    # `indent`: otherwise (Lua's `return nil`) nothing it changed counts.
    (!options.set_metadata_variable || get(meta, "indent", nothing) == MetaBool(false)) && return nothing
    meta["indent"] = true
    format = something(ctx.format, "")
    options.set_header_includes || return nothing
    header = if occursin("htm", format)  # (Lua's pattern `html*`: `htm`, then any `l`s)
        RawBlock("html", replace(CSS, "SIZE" => lua_replacement(options.size, "SIZE")))
    elseif occursin("latex", format) && options.size != "1em"
        RawBlock("tex", "\\setlength{\\parindent}{$(options.size)}")
    else
        return nothing
    end
    # before what is there: a list's items, or one value
    existing = get(meta, "header-includes", nothing)
    meta["header-includes"] = MetaBlocks(Block[header; as_block.(truthy(existing) ? items(existing) : [])])
    nothing
end

"The paragraph with its first line indented or not."
function markup(kind, para, format)
    if occursin("latex", format)
        head = isempty(para.content) ? nothing : para.content[1]
        head in (command(:indent), command(:noindent)) ? (para.content[1] = command(kind)) : pushfirst!(para.content, command(kind))
        return [para]
    end
    occursin("htm", format) && return [RawBlock("html", DIVS[kind]), para]
    [para]
end

function body(doc::Pandoc, ctx)
    format = something(ctx.format, "")
    blocks = Block[]
    after = false  # whether the block before removes an indent
    for block in doc.blocks
        if !(block isa Para)
            after = string(nameof(typeof(block))) in options.remove_after
            push!(blocks, block)
            continue
        end
        head = isempty(block.content) ? nothing : block.content[1]
        explicit = findfirst(kind -> head == command(kind), (:indent, :noindent))
        if explicit !== nothing
            append!(blocks, markup((:indent, :noindent)[explicit], block, format))
        elseif options.auto_remove && after
            append!(blocks, markup(:noindent, block, format))
        else
            push!(blocks, block)
        end
        after = false
    end
    doc.blocks = blocks
    nothing
end

run_filter([configure, body])  # the options and the metadata, then the body
