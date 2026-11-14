//! Interpolate metadata fields into a document: a pandoc filter.
//!
//! A word that is all `%{field}` becomes that field's value, if it is
//! inlines (in a span) or a string. What it puts in is walked too, so a
//! field may name another.
//!
//! A port of pandocfilters' examples/metavars.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Attr, Ctx, Filter, Inline, MetaValue, Order, Span};

struct MetaVars;

impl Filter for MetaVars {
    // pandocfilters' order: what is put in is walked
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Str(s) = x else { return None };
        let field = s.strip_prefix("%{")?.strip_suffix('}')?;
        // a field that isn't there is an error, as in the original
        let Some(value) = ctx.meta().and_then(|meta| meta.get(field)) else {
            return ctx.fail(format!("no metadata field {field}"));
        };
        match value {
            // the original's attributes() reads "class" as an attribute, not a class
            MetaValue::MetaInlines(content) => Some(vec![
                Span {
                    attr: Attr::new("", &[], &[("class", "interpolated"), ("field", field)]),
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
    panir::filter(|doc, format| panir::apply(doc, &mut MetaVars, format));
}
