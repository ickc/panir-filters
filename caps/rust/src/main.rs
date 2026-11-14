//! Upper-case all regular text: a pandoc filter. Code, link URLs and the
//! like are not text, and stay as they are.
//!
//! A port of pandocfilters' examples/caps.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Ctx, Filter, Inline};

struct Caps;

impl Filter for Caps {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        if let Inline::Str(s) = x {
            *s = s.to_uppercase().into();
        }
        None
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Caps, format));
}
