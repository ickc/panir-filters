#!/usr/bin/env julia
# Affiliations and correspondence as blocks before the body: a pandoc
# filter, run after scholarly-metadata (whose `author` and `institute` it
# reads).
#
# The body starts with a note on equal contributors (if any), the
# affiliations, numbered, and the corresponding authors (`correspondence`
# and an `email`), linked; `author` becomes one line of names with their
# marks (in LaTeX, a list, one each), and `institute` goes.
#
# A port of pandoc/lua-filters' author-info-blocks (Copyright © 2017–2021
# Albert Krewinkel; ISC; the repository's MIT licence, see ../LICENSE).

using Panir

corresponding(author) = truthy(get(author, "correspondence", nothing)) && truthy(get(author, "email", nothing))

# `author` and `institute`, as scholarly-metadata leaves them
maps(v) = [x isa MetaMap ? x.content : error("author-info-blocks: expected a map, got $(typeof(x))") for x in items(v)]

"A name or an email: inlines (the original fails on anything else)."
inlines(v::MetaInlines) = v.content
inlines(v) = error("author-info-blocks: expected inlines, got $(typeof(v))")

function intercalate(lists, sep)
    out = Inline[]
    for (i, xs) in enumerate(lists)
        i > 1 && append!(out, sep())  # a new separator each time: a node is in one place
        append!(out, xs)
    end
    out
end

function info_blocks(doc::Pandoc, ctx)
    format = something(ctx.format, "")
    function mark(name)
        latex = format == "latex"
        name == :corresponding_author && return Inline[latex ? RawInline("latex", "*") : Str("✉")]
        Inline[latex ? RawInline("latex", "\$\\dagger{}\$") : Str("*")]
    end
    "The author's name, with its marks as a superscript."
    function author_line(author)
        marks = truthy(get(author, "equal_contributor", nothing)) ? [mark(:equal_contributor)] : Vector{Inline}[]
        append!(marks, [Inline[Str(stringify(i))] for i in items(author["institute"])])
        corresponding(author) && push!(marks, mark(:corresponding_author))
        [inlines(author["name"]); Superscript(intercalate(marks, () -> [Str(",")]))]
    end

    authors = maps(get(doc.meta, "author", nothing))
    institutes = maps(pop!(doc.meta, "institute", nothing))

    body = Block[]
    if any(a -> truthy(get(a, "equal_contributor", nothing)), authors)
        push!(body, Para(Superscript(mark(:equal_contributor)), Space(), Str("These authors contributed equally to this work.")))
    end
    lines = [[Superscript(Str(string(i))); Space(); inlines(inst["name"])] for (i, inst) in enumerate(institutes)]
    push!(body, Para(intercalate(lines, () -> [LineBreak()])))
    # the name is in the author line too: a copy here
    links = [[Link([deepcopy(inlines(a["name"])); Space(); Str("<"); inlines(a["email"]); Str(">")];
                   url = "mailto:" * stringify(a["email"]))]
             for a in authors if corresponding(a)]
    if !isempty(links)
        push!(body, Para([Superscript(mark(:corresponding_author)); Space(); Str("Correspondence:"); Space();
                          intercalate(links, () -> [Str(","), Space()])]))
    end
    prepend!(doc.blocks, body)

    names = map(author_line, authors)
    if occursin("latex", format)  # (the marks above ask for "latex" exactly)
        doc.meta["author"] = MetaList(map(MetaInlines, names))
        return nothing
    end
    isempty(names) && error("author-info-blocks: no authors")  # (the original fails too)
    line = intercalate(names[1:end-1], () -> [Str(","), Space()])
    length(names) > 1 && append!(line, [Str(","), Space(), Str("and"), Space()])
    doc.meta["author"] = MetaInlines([line; names[end]])
    nothing
end

run_filter(info_blocks)
