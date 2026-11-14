//! Emphasis shown in capitals: a pandoc filter. An emphasis is replaced by
//! its content, upper-cased (its text: code, link URLs and the like stay
//! as they are).
//!
//! A port of pandocfilters' examples/deemph.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Ctx, Filter, Inline, Order};

struct Caps;

impl Filter for Caps {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        if let Inline::Str(s) = x {
            *s = s.to_uppercase().into();
        }
        None
    }
}

struct Deemph;

impl Filter for Deemph {
    // pandocfilters' order: what replaces an emphasis is not filtered
    // itself, only what is in it, so an emphasis directly in another stays one
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Emph(content) = x else {
            return None;
        };
        let mut content = std::mem::take(content);
        match panir::apply_with(&mut content, &mut Caps, ctx.conversion()) {
            Ok(()) => Some(content),
            Err(e) => ctx.fail(e),
        }
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Deemph, format));
}
