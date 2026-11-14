#!/usr/bin/env julia
# CiTO properties in citations: a pandoc filter.
#
# A citation `@property:key` (or an alias of the property:
# `@evidence:Li95`) is cited as `@key`, and the key is listed under the
# property (`citesAsEvidence`) in the metadata's `cito_cites`; any other
# under `citation`.
#
# A port of pandoc-ext/cito (Copyright © 2017–2022 Albert Krewinkel, Robert
# Winkler, © 2023-2024 Albert Krewinkel, Egon Willighagen; MIT, see
# ../LICENSE).

using Panir

const PROPERTIES_AND_ALIASES = Dict{String, Vector{String}}(
    "agreesWith" => ["agreeWith", "agree_with", "agrees_with"],
    "citation" => [],
    "cites" => [],
    "citesAsAuthority" => ["asAuthority", "cites_as_authority", "as_authority", "authority"],
    "citesAsDataSource" => ["asDataSource", "dataSource", "cites_as_data_source", "as_data_source", "data_source"],
    "citesAsEvidence" => ["asEvidence", "cites_as_evidence", "as_evidence", "evidence"],
    "citesAsMetadataDocument" => ["asMetadataDocument", "metadataDocument", "cites_as_metadata_document", "as_metadata_document", "metadata_document", "metadata"],
    "citesAsPotentialSolution" => ["cites_as_potential_solution", "potentialSolution", "potential_solution", "solution"],
    "citesAsRecommendedReading" => ["asRecommendedReading", "recommendedReading", "cites_as_recommended_reading", "as_recommended_reading", "recommended_reading"],
    "citesAsRelated" => ["cites_as_related", "related"],
    "citesAsSourceDocument" => ["cites_as_source_document", "sourceDocument", "source_document"],
    "citesForInformation" => ["cites_for_information", "information"],
    "compiles" => [],
    "confirms" => [],
    "containsAssertionFrom" => [],
    "corrects" => [],
    "credits" => [],
    "critiques" => [],
    "derides" => [],
    "describes" => [],
    "disagreesWith" => ["disagrees_with", "disagree", "disagrees"],
    "discusses" => [],
    "disputes" => [],
    "documents" => [],
    "extends" => [],
    "includesExcerptFrom" => ["excerptFrom", "excerpt", "excerpt_from", "includes_excerpt_from"],
    "includesQuotationFrom" => ["quotationFrom", "includes_quotation_from", "quotation", "quotation_from"],
    "linksTo" => ["links_to", "link"],
    "obtainsBackgroundFrom" => ["backgroundFrom", "obtains_background_from", "background", "background_from"],
    "providesDataFor" => [],
    "obtainsSupportFrom" => [],
    "qualifies" => [],
    "parodies" => [],
    "refutes" => [],
    "repliesTo" => ["replies_to"],
    "retracts" => [],
    "reviews" => [],
    "ridicules" => [],
    "speculatesOn" => [],
    "supports" => [],
    "updates" => [],
    "usesConclusionsFrom" => ["uses_conclusions_from"],
    "usesDataFrom" => ["dataFrom", "uses_data_from", "data", "data_from"],
    "usesMethodIn" => ["methodIn", "uses_method_in", "method", "method_in"],
)

# every property is an alias of itself
const PROPERTIES_BY_ALIAS = Dict(alias => p for (p, aliases) in PROPERTIES_AND_ALIASES for alias in [p; aliases])

"The property and the key; `citation` and the id if it names none."
function split_cito(id)
    parts = split(id, ':'; limit = 2)
    if length(parts) == 2 && all(!isempty, parts) && haskey(PROPERTIES_BY_ALIAS, parts[1])
        return PROPERTIES_BY_ALIAS[parts[1]], String(parts[2])
    end
    "citation", id
end

const REPEATS = Dict('*' => "*", '+' => "+", '-' => "*?", '?' => "?")

"""The id as the original's `gsub` takes it, a Lua pattern: items, each a
character (`.` any, `%x` the character x), perhaps repeated by the next (`*`,
`+`, `-` for the fewest, `?`); anything else is itself (NOTES.md)."""
function lua_pattern(text)
    out = IOBuffer()
    chars = collect(text)
    i = 1
    while i <= length(chars)
        escaped = chars[i] == '%' && i < length(chars)
        i += escaped
        c = chars[i]
        print(out, c == '.' && !escaped ? "." : isascii(c) && !isletter(c) && !isdigit(c) ? "\\$c" : c)
        i += 1
        if i <= length(chars) && haskey(REPEATS, chars[i])
            print(out, REPEATS[chars[i]])
            i += 1
        end
    end
    Regex(String(take!(out)), "s")
end

function cito(doc::Pandoc)
    by_property = Dict{String, Vector{String}}()
    function extract(cite::Cite)
        for citation in cite.citations
            prop, key = split_cito(citation.id)
            push!(get!(Vector{String}, by_property, prop), key)
            pattern = lua_pattern(citation.id)
            # a string, as the original's: its words
            walk!((s::Str) -> replace(s.text, pattern => key), cite.content)
            citation.id = key
        end
    end
    walk!(extract, doc)
    doc.meta["cito_cites"] = by_property
    nothing
end

run_filter(cito)
