//! Interpolate metadata fields into a document: a pandoc filter.
//!
//! A word that is all `%{field}` becomes that field's value, if it is
//! inlines (in a span) or a string. What it puts in is walked too, so a
//! field may name another.
//!
//! A port of pandocfilters' examples/metavars.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Attr, Ctx, Filter, Inline, Meta, MetaValue, Order, Span};

/// The metadata as it was before the walk (a function's context has no
/// document to read it from).
struct MetaVars {
    meta: Meta,
}

impl Filter for MetaVars {
    // pandocfilters' order: what is put in is walked
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Str(s) = x else { return None };
        let field = s.strip_prefix("%{")?.strip_suffix('}')?;
        // a field that isn't there is an error, as in the original
        let value = self
            .meta
            .get(field)
            .unwrap_or_else(|| panic!("no metadata field {field}"));
        match value {
            MetaValue::MetaInlines(content) => Some(vec![
                Span {
                    // the original's attributes() reads "class" as an attribute, not a class
                    attr: Attr {
                        attributes: vec![
                            ("class".into(), "interpolated".into()),
                            ("field".into(), field.into()),
                        ],
                        ..Attr::default()
                    },
                    content: content.clone(),
                }
                .into(),
            ]),
            MetaValue::MetaString(text) => Some(vec![Inline::Str(text.clone())]),
            _ => None,
        }
    }
}

fn main() {
    panir::filter(|doc, format| {
        let meta = doc.meta.clone();
        panir::apply(doc, &mut MetaVars { meta }, format);
    });
}
