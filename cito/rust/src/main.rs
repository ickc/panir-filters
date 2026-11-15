//! CiTO properties in citations: a pandoc filter.
//!
//! A citation `@property:key` (or an alias of the property:
//! `@evidence:Li95`) is cited as `@key`, and the key is listed under the
//! property (`citesAsEvidence`) in the metadata's `cito_cites`; any other
//! under `citation`.
//!
//! A port of pandoc-ext/cito (Copyright © 2017–2022 Albert Krewinkel, Robert
//! Winkler, © 2023-2024 Albert Krewinkel, Egon Willighagen; MIT, see
//! ../LICENSE).

use std::collections::BTreeMap;

use panir::{Ctx, Filter, Inline, Meta, MetaValue};
use regex::{NoExpand, Regex};

const PROPERTIES_AND_ALIASES: &[(&str, &[&str])] = &[
    ("agreesWith", &["agreeWith", "agree_with", "agrees_with"]),
    ("citation", &[]),
    ("cites", &[]),
    (
        "citesAsAuthority",
        &[
            "asAuthority",
            "cites_as_authority",
            "as_authority",
            "authority",
        ],
    ),
    (
        "citesAsDataSource",
        &[
            "asDataSource",
            "dataSource",
            "cites_as_data_source",
            "as_data_source",
            "data_source",
        ],
    ),
    (
        "citesAsEvidence",
        &["asEvidence", "cites_as_evidence", "as_evidence", "evidence"],
    ),
    (
        "citesAsMetadataDocument",
        &[
            "asMetadataDocument",
            "metadataDocument",
            "cites_as_metadata_document",
            "as_metadata_document",
            "metadata_document",
            "metadata",
        ],
    ),
    (
        "citesAsPotentialSolution",
        &[
            "cites_as_potential_solution",
            "potentialSolution",
            "potential_solution",
            "solution",
        ],
    ),
    (
        "citesAsRecommendedReading",
        &[
            "asRecommendedReading",
            "recommendedReading",
            "cites_as_recommended_reading",
            "as_recommended_reading",
            "recommended_reading",
        ],
    ),
    ("citesAsRelated", &["cites_as_related", "related"]),
    (
        "citesAsSourceDocument",
        &[
            "cites_as_source_document",
            "sourceDocument",
            "source_document",
        ],
    ),
    (
        "citesForInformation",
        &["cites_for_information", "information"],
    ),
    ("compiles", &[]),
    ("confirms", &[]),
    ("containsAssertionFrom", &[]),
    ("corrects", &[]),
    ("credits", &[]),
    ("critiques", &[]),
    ("derides", &[]),
    ("describes", &[]),
    (
        "disagreesWith",
        &["disagrees_with", "disagree", "disagrees"],
    ),
    ("discusses", &[]),
    ("disputes", &[]),
    ("documents", &[]),
    ("extends", &[]),
    (
        "includesExcerptFrom",
        &[
            "excerptFrom",
            "excerpt",
            "excerpt_from",
            "includes_excerpt_from",
        ],
    ),
    (
        "includesQuotationFrom",
        &[
            "quotationFrom",
            "includes_quotation_from",
            "quotation",
            "quotation_from",
        ],
    ),
    ("linksTo", &["links_to", "link"]),
    (
        "obtainsBackgroundFrom",
        &[
            "backgroundFrom",
            "obtains_background_from",
            "background",
            "background_from",
        ],
    ),
    ("providesDataFor", &[]),
    ("obtainsSupportFrom", &[]),
    ("qualifies", &[]),
    ("parodies", &[]),
    ("refutes", &[]),
    ("repliesTo", &["replies_to"]),
    ("retracts", &[]),
    ("reviews", &[]),
    ("ridicules", &[]),
    ("speculatesOn", &[]),
    ("supports", &[]),
    ("updates", &[]),
    ("usesConclusionsFrom", &["uses_conclusions_from"]),
    (
        "usesDataFrom",
        &["dataFrom", "uses_data_from", "data", "data_from"],
    ),
    (
        "usesMethodIn",
        &["methodIn", "uses_method_in", "method", "method_in"],
    ),
];

/// The property and the key; `citation` and the id if it names none (every
/// property is an alias of itself).
fn split_cito(id: &str) -> (&'static str, &str) {
    if let Some((alias, key)) = id.split_once(':')
        && !alias.is_empty()
        && !key.is_empty()
        && let Some((property, _)) = PROPERTIES_AND_ALIASES
            .iter()
            .find(|(p, aliases)| *p == alias || aliases.contains(&alias))
    {
        return (property, key);
    }
    ("citation", id)
}

/// The id as the original's `gsub` takes it, a Lua pattern: items, each a
/// character (`.` any, `%x` the character x), perhaps repeated by the next
/// (`*`, `+`, `-` for the fewest, `?`); anything else is itself (NOTES.md).
fn lua_pattern(text: &str) -> Regex {
    let mut out = String::from("(?s)");
    let mut chars = text.chars().peekable();
    while let Some(mut c) = chars.next() {
        let escaped = c == '%' && chars.peek().is_some();
        if escaped {
            c = chars.next().expect("peeked");
        }
        if c == '.' && !escaped {
            out.push('.');
        } else {
            out.push_str(&regex::escape(c.encode_utf8(&mut [0; 4])));
        }
        let repeat = match chars.peek() {
            Some('*') => "*",
            Some('+') => "+",
            Some('-') => "*?",
            Some('?') => "?",
            _ => continue,
        };
        out.push_str(repeat);
        chars.next();
    }
    Regex::new(&out).expect("characters, escaped")
}

/// The key where a text has the id (as a pattern).
struct Rewrite {
    pattern: Regex,
    key: String,
}

impl Filter for Rewrite {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Str(s) = x else { return None };
        // a string, as the original's: its words
        Some(panir::inlines(
            &self.pattern.replace_all(s, NoExpand(&self.key)),
        ))
    }
}

#[derive(Default)]
struct Cito {
    by_property: BTreeMap<&'static str, Vec<String>>,
}

impl Filter for Cito {
    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Cite(cite) = x else { return None };
        for citation in &mut cite.citations {
            let (property, key) = split_cito(&citation.id);
            let key = key.to_string();
            self.by_property
                .entry(property)
                .or_default()
                .push(key.clone());
            ctx.apply(
                &mut cite.content,
                &mut Rewrite {
                    pattern: lua_pattern(&citation.id),
                    key: key.clone(),
                },
            )?;
            citation.id = key.into();
        }
        None
    }

    // after every inline, the metadata's first
    fn meta(&mut self, meta: &mut Meta, _: &mut Ctx) -> Option<()> {
        let cites = std::mem::take(&mut self.by_property)
            .into_iter()
            .map(|(p, keys)| (p, keys.into_iter().collect::<MetaValue>()))
            .collect();
        meta.insert("cito_cites".into(), cites);
        Some(())
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Cito::default(), format));
}
