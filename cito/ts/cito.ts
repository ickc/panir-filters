#!/usr/bin/env node
// CiTO properties in citations: a pandoc filter.
//
// A citation `@property:key` (or an alias of the property:
// `@evidence:Li95`) is cited as `@key`, and the key is listed under the
// property (`citesAsEvidence`) in the metadata's `cito_cites`; any other
// under `citation`.
//
// A port of pandoc-ext/cito (Copyright © 2017–2022 Albert Krewinkel, Robert
// Winkler, © 2023-2024 Albert Krewinkel, Egon Willighagen; MIT, see
// ../LICENSE).

import { applyFilter, MetaMap } from "panir";
import { runFilter } from "panir/node";

const PROPERTIES_AND_ALIASES: { readonly [property: string]: readonly string[] } = {
  agreesWith: ["agreeWith", "agree_with", "agrees_with"],
  citation: [],
  cites: [],
  citesAsAuthority: ["asAuthority", "cites_as_authority", "as_authority", "authority"],
  citesAsDataSource: ["asDataSource", "dataSource", "cites_as_data_source", "as_data_source", "data_source"],
  citesAsEvidence: ["asEvidence", "cites_as_evidence", "as_evidence", "evidence"],
  citesAsMetadataDocument: [
    "asMetadataDocument", "metadataDocument", "cites_as_metadata_document", "as_metadata_document",
    "metadata_document", "metadata",
  ],
  citesAsPotentialSolution: ["cites_as_potential_solution", "potentialSolution", "potential_solution", "solution"],
  citesAsRecommendedReading: [
    "asRecommendedReading", "recommendedReading", "cites_as_recommended_reading", "as_recommended_reading",
    "recommended_reading",
  ],
  citesAsRelated: ["cites_as_related", "related"],
  citesAsSourceDocument: ["cites_as_source_document", "sourceDocument", "source_document"],
  citesForInformation: ["cites_for_information", "information"],
  compiles: [],
  confirms: [],
  containsAssertionFrom: [],
  corrects: [],
  credits: [],
  critiques: [],
  derides: [],
  describes: [],
  disagreesWith: ["disagrees_with", "disagree", "disagrees"],
  discusses: [],
  disputes: [],
  documents: [],
  extends: [],
  includesExcerptFrom: ["excerptFrom", "excerpt", "excerpt_from", "includes_excerpt_from"],
  includesQuotationFrom: ["quotationFrom", "includes_quotation_from", "quotation", "quotation_from"],
  linksTo: ["links_to", "link"],
  obtainsBackgroundFrom: ["backgroundFrom", "obtains_background_from", "background", "background_from"],
  providesDataFor: [],
  obtainsSupportFrom: [],
  qualifies: [],
  parodies: [],
  refutes: [],
  repliesTo: ["replies_to"],
  retracts: [],
  reviews: [],
  ridicules: [],
  speculatesOn: [],
  supports: [],
  updates: [],
  usesConclusionsFrom: ["uses_conclusions_from"],
  usesDataFrom: ["dataFrom", "uses_data_from", "data", "data_from"],
  usesMethodIn: ["methodIn", "uses_method_in", "method", "method_in"],
};

// every property is an alias of itself
const PROPERTIES_BY_ALIAS = new Map(
  Object.entries(PROPERTIES_AND_ALIASES).flatMap(([p, aliases]) => [p, ...aliases].map((a) => [a, p] as const)),
);

/** The property and the key; `citation` and the id if it names none. */
function splitCito(id: string): [string, string] {
  const colon = id.indexOf(":");
  const property = colon > 0 && colon < id.length - 1 ? PROPERTIES_BY_ALIAS.get(id.slice(0, colon)) : undefined;
  return property === undefined ? ["citation", id] : [property, id.slice(colon + 1)];
}

const REPEATS: { readonly [c: string]: string } = { "*": "*", "+": "+", "-": "*?", "?": "?" };

/** The id as the original's `gsub` takes it, a Lua pattern: items, each a
 * character (`.` any, `%x` the character x), perhaps repeated by the next
 * (`*`, `+`, `-` for the fewest, `?`); anything else is itself (NOTES.md). */
function luaPattern(text: string): RegExp {
  let out = "";
  for (let i = 0; i < text.length; i++) {
    const escaped = text[i] === "%" && i + 1 < text.length;
    if (escaped) i++;
    const c = text[i]!;
    out += c === "." && !escaped ? "." : c.replace(/[\\^$.*+?()[\]{}|/]/g, "\\$&");
    if (i + 1 < text.length && text[i + 1]! in REPEATS) out += REPEATS[text[++i]!];
  }
  return new RegExp(out, "gs");
}

await runFilter({
  Pandoc: (doc) => {
    const byProperty = new Map<string, string[]>();
    applyFilter(doc, {
      Cite: (cite) => {
        for (const citation of cite.citations) {
          const [property, key] = splitCito(citation.id);
          byProperty.set(property, [...(byProperty.get(property) ?? []), key]);
          const pattern = luaPattern(citation.id);
          // a string, as the original returns one: its words
          applyFilter(cite.content, { Str: (s) => s.text.replace(pattern, () => key) });
          citation.id = key;
        }
      },
    });
    doc.meta.cito_cites = MetaMap(Object.fromEntries(byProperty));
  },
});
