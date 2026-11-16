#!/usr/bin/env julia
# Several bibliographies, one a topic: a pandoc filter.
#
# The metadata's `bibliography` maps topics to files; a div `#refs-TOPIC`
# (or `refs_TOPIC`, `refsTOPIC`) gets the bibliography of that topic's file,
# of every work the document cites. The document's citations are resolved
# with all the files together.
#
# citeproc is pandoc's (`pandoc.utils.citeproc` in the original): run here
# with `LibPandoc.convert` of the document's JSON with `citeproc`, in this
# process: the pandoc the filter runs in under pandocjl or
# `LibPandoc.convert`, or LibPandoc as a library under plain pandoc.
#
# A port of pandoc-ext/multibib (Copyright © 2018-2024 Albert Krewinkel;
# MIT, see ../LICENSE).

using Panir
using Logging: NullLogger, with_logger
import LibPandoc

# What citeproc reads of the metadata, copied for a topic's bibliography.
const FIELDS = ["bibliography", "references", "csl", "citation-style", "link-citations", "citation-abbreviations",
    "lang", "suppress-bibliography", "reference-section-title", "notes-after-punctuation", "nocite"]

"pandoc's citeproc on a document, its warnings silenced."
citeproc(doc) = with_logger(NullLogger()) do
    Panir.parse(LibPandoc.convert(Panir.serialize(doc); from = "json", to = "json", citeproc = true))
end

const cites = Cite[]  # every citation of the document
const doc_meta = Ref(Panir.Meta())  # the document's metadata, as the first pass saw it
const refs = Ref{Union{Div, Nothing}}(nothing)  # the bibliography's div, as citeproc made it

# a new document (a host may run these filters on many: Lua loads the
# original anew for each)
reset(::Pandoc) = (empty!(cites); nothing)

collect_cite(c::Cite) = (push!(cites, deepcopy(c)); nothing)
collect_cite(m::Panir.Meta) = (doc_meta[] = deepcopy(m); nothing)

"The citations resolved with every bibliography at once."
function resolve_doc_citations(doc::Pandoc)
    given = get(doc.meta, "bibliography", nothing)
    # (Lua's pandoc.utils.type: a map is a "table"; its values in any order)
    files = given isa MetaMap ? MetaValue[MetaString(stringify(v)) for v in values(given.content)] : MetaValue[]
    doc.meta["bibliography"] = MetaList(files)
    push!(doc.blocks, Div(Block[]; identifier = "refs"))
    out = citeproc(doc)
    refs[] = pop!(out.blocks)
    # (the original restores `orig_bib`, which it never set: the field goes)
    delete!(out.meta, "bibliography")
    out
end

function create_topic_bibliography(div::Div, ctx)
    m = match(r"^refs[-_]?([-_A-Za-z0-9]*)$", div.attr.identifier)
    bibliography = get(doc_meta[], "bibliography", nothing)
    # (a map's entry; Lua indexes a list or text by a name to nothing)
    bibfile = m !== nothing && bibliography isa MetaMap ? get(bibliography.content, m[1], nothing) : nothing
    bibfile === nothing && return nothing
    meta = Panir.Meta(k => deepcopy(doc_meta[][k]) for k in FIELDS if haskey(doc_meta[], k))
    meta["bibliography"] = deepcopy(bibfile)
    res = citeproc(Pandoc(Block[Para(deepcopy(cites)), Div(Block[]; identifier = "refs")]; meta))
    div.content = res.blocks[2].content
    div.attr.classes = unique(refs[].attr.classes)
    div.attr.attributes = deepcopy(refs[].attr.attributes)
    if ctx.format in ("odt", "docx")
        # pandoc gives Word's bibliography this style
        style = get(div.attr, "custom-style", "")
        div.attr["custom-style"] = isempty(style) ? "Bibliography" : style
    end
    div
end

run_filter([reset, collect_cite, resolve_doc_citations, create_topic_bibliography])
