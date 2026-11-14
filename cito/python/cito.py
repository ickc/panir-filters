#!/usr/bin/env python3
"""CiTO properties in citations: a pandoc filter.

A citation `@property:key` (or an alias of the property:
`@evidence:Li95`) is cited as `@key`, and the key is listed under the
property (`citesAsEvidence`) in the metadata's `cito_cites`; any other
under `citation`.

A port of pandoc-ext/cito (Copyright © 2017–2022 Albert Krewinkel, Robert
Winkler, © 2023-2024 Albert Krewinkel, Egon Willighagen; MIT, see
../LICENSE).
"""

import re

from panir import Cite, Filter, Pandoc, Str

PROPERTIES_AND_ALIASES = {
    "agreesWith": ["agreeWith", "agree_with", "agrees_with"],
    "citation": [],
    "cites": [],
    "citesAsAuthority": ["asAuthority", "cites_as_authority", "as_authority", "authority"],
    "citesAsDataSource": [
        "asDataSource",
        "dataSource",
        "cites_as_data_source",
        "as_data_source",
        "data_source",
    ],
    "citesAsEvidence": ["asEvidence", "cites_as_evidence", "as_evidence", "evidence"],
    "citesAsMetadataDocument": [
        "asMetadataDocument",
        "metadataDocument",
        "cites_as_metadata_document",
        "as_metadata_document",
        "metadata_document",
        "metadata",
    ],
    "citesAsPotentialSolution": [
        "cites_as_potential_solution",
        "potentialSolution",
        "potential_solution",
        "solution",
    ],
    "citesAsRecommendedReading": [
        "asRecommendedReading",
        "recommendedReading",
        "cites_as_recommended_reading",
        "as_recommended_reading",
        "recommended_reading",
    ],
    "citesAsRelated": ["cites_as_related", "related"],
    "citesAsSourceDocument": ["cites_as_source_document", "sourceDocument", "source_document"],
    "citesForInformation": ["cites_for_information", "information"],
    "compiles": [],
    "confirms": [],
    "containsAssertionFrom": [],
    "corrects": [],
    "credits": [],
    "critiques": [],
    "derides": [],
    "describes": [],
    "disagreesWith": ["disagrees_with", "disagree", "disagrees"],
    "discusses": [],
    "disputes": [],
    "documents": [],
    "extends": [],
    "includesExcerptFrom": ["excerptFrom", "excerpt", "excerpt_from", "includes_excerpt_from"],
    "includesQuotationFrom": [
        "quotationFrom",
        "includes_quotation_from",
        "quotation",
        "quotation_from",
    ],
    "linksTo": ["links_to", "link"],
    "obtainsBackgroundFrom": [
        "backgroundFrom",
        "obtains_background_from",
        "background",
        "background_from",
    ],
    "providesDataFor": [],
    "obtainsSupportFrom": [],
    "qualifies": [],
    "parodies": [],
    "refutes": [],
    "repliesTo": ["replies_to"],
    "retracts": [],
    "reviews": [],
    "ridicules": [],
    "speculatesOn": [],
    "supports": [],
    "updates": [],
    "usesConclusionsFrom": ["uses_conclusions_from"],
    "usesDataFrom": ["dataFrom", "uses_data_from", "data", "data_from"],
    "usesMethodIn": ["methodIn", "uses_method_in", "method", "method_in"],
}

# every property is an alias of itself
PROPERTIES_BY_ALIAS = {
    alias: p for p, aliases in PROPERTIES_AND_ALIASES.items() for alias in (p, *aliases)
}


def split_cito(citation_id: str) -> tuple[str, str]:
    """The property and the key; `citation` and the id if it names none."""
    alias, _, key = citation_id.partition(":")
    if key and alias in PROPERTIES_BY_ALIAS:
        return PROPERTIES_BY_ALIAS[alias], key
    return "citation", citation_id


REPEATS = {"*": "*", "+": "+", "-": "*?", "?": "?"}


def lua_pattern(text: str) -> re.Pattern[str]:
    """The id as the original's `gsub` takes it, a Lua pattern: items, each
    a character (`.` any, `%x` the character x), perhaps repeated by the
    next (`*`, `+`, `-` for the fewest, `?`); anything else is itself
    (NOTES.md)."""
    out, i = [], 0
    while i < len(text):
        escaped = text[i] == "%" and i + 1 < len(text)
        i += escaped
        item = "." if text[i] == "." and not escaped else re.escape(text[i])
        i += 1
        if i < len(text) and text[i] in REPEATS:
            item += REPEATS[text[i]]
            i += 1
        out.append(item)
    return re.compile("".join(out), re.DOTALL)


def rewrite(citation_id: str, key: str) -> Filter:
    """The key where a text has the id (as a pattern)."""
    pattern = lua_pattern(citation_id)
    g = Filter()

    @g.on(Str)
    def replace(s: Str) -> str:
        return pattern.sub(lambda _: key, s.text)  # a string, as the original's: its words

    return g


f = Filter()


@f.on(Pandoc)
def cito(doc: Pandoc) -> None:
    by_property: dict[str, list[str]] = {}
    cites = Filter()

    @cites.on(Cite)
    def extract(cite: Cite) -> None:
        for citation in cite.citations:
            prop, key = split_cito(citation.id)
            by_property.setdefault(prop, []).append(key)
            cite.content.walk(rewrite(citation.id, key))
            citation.id = key

    doc.walk(cites)
    doc.meta["cito_cites"] = by_property


if __name__ == "__main__":
    f.main()
