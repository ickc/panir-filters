//! A section headed "Abstract" moves into the metadata: a pandoc filter.
//!
//! A section whose heading has the identifier `abstract`, at the highest
//! level met so far, in any list of blocks, runs to the next such heading or
//! a horizontal rule; it becomes the metadata's `abstract`, unless that is
//! set. `abstract-section.section-identifiers` lists other identifiers (each
//! its own field); the configuration is removed.
//!
//! A port of pandoc-ext/abstract-section (Copyright © 2017-2023 Albert
//! Krewinkel; MIT, see ../LICENSE).

use std::collections::{BTreeMap, BTreeSet};

use panir::{Block, Ctx, Filter, MetaValue, Pandoc, stringify};

/// `abstract-section.section-identifiers`, as the original reads it.
fn section_identifiers(config: Option<MetaValue>) -> Result<BTreeSet<String>, String> {
    let identifiers = match config {
        Some(MetaValue::MetaMap(mut config)) => config.remove("section-identifiers"),
        Some(MetaValue::MetaBool(true)) => {
            return Err("abstract-section: the configuration is true, not a map".into());
        }
        _ => None, // (a string, a list... has no field in Lua)
    };
    Ok(match identifiers {
        // (each inline of text too)
        Some(MetaValue::MetaList(xs)) if !xs.is_empty() => xs.iter().map(stringify).collect(),
        Some(MetaValue::MetaInlines(xs)) if !xs.is_empty() => xs.iter().map(stringify).collect(),
        Some(MetaValue::MetaBlocks(xs)) if !xs.is_empty() => xs.iter().map(stringify).collect(),
        Some(MetaValue::MetaString(s)) if !s.is_empty() => BTreeSet::new(), // (Lua takes its length, then finds no items)
        Some(MetaValue::MetaBool(true)) => {
            return Err("abstract-section: section-identifiers is true, not a list".into());
        }
        _ => BTreeSet::from(["abstract".to_string()]),
    })
}

/// Takes the sections out of each list of blocks.
struct Sections<'a> {
    identifiers: &'a BTreeSet<String>,
    collected: BTreeMap<String, Vec<Block>>,
    toplevel: i64, // the highest level of heading met so far
}

impl Filter for Sections<'_> {
    fn blocks(&mut self, xs: &mut Vec<Block>, _: &mut Ctx) {
        let mut body = vec![];
        let mut section = None; // the identifier of the section being collected
        for block in std::mem::take(xs) {
            match (&block, &section) {
                (Block::Header(h), _) if h.level <= self.toplevel => {
                    self.toplevel = h.level;
                    if self.identifiers.contains(h.attr.identifier.as_str()) {
                        let id = h.attr.identifier.to_string();
                        self.collected.insert(id.clone(), vec![]);
                        section = Some(id);
                    } else {
                        section = None;
                        body.push(block);
                    }
                }
                (_, None) => body.push(block),
                (Block::HorizontalRule, Some(_)) => section = None,
                (_, Some(id)) => self
                    .collected
                    .get_mut(id)
                    .expect("collecting it")
                    .push(block),
            }
        }
        *xs = body;
    }
}

struct AbstractSection;

impl Filter for AbstractSection {
    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) {
        let identifiers = match section_identifiers(doc.meta.remove("abstract-section")) {
            Ok(identifiers) => identifiers,
            Err(e) => {
                ctx.fail::<()>(e);
                return;
            }
        };
        let mut sections = Sections {
            identifiers: &identifiers,
            collected: BTreeMap::new(),
            toplevel: 6,
        };
        // the body's lists, inner ones first
        if ctx.apply(&mut doc.blocks, &mut sections).is_none() {
            return;
        }
        for (key, blocks) in sections.collected {
            let set = matches!(doc.meta.get(key.as_str()), Some(v) if !matches!(v, MetaValue::MetaBool(false)));
            if !blocks.is_empty() && !set {
                doc.meta.insert(key.into(), MetaValue::MetaBlocks(blocks));
            }
        }
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut AbstractSection, format));
}
