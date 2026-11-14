//! Leave out everything between `<!-- BEGIN COMMENT -->` and `<!-- END
//! COMMENT -->`: a pandoc filter. Each marker is an HTML block of its own
//! (blank lines around it).
//!
//! A port of pandocfilters' examples/comments.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Block, Ctx, Filter, Inline, Order};

const BEGIN: &str = "<!-- BEGIN COMMENT -->";
const END: &str = "<!-- END COMMENT -->";

#[derive(Default)]
struct Comments {
    hiding: bool,
}

impl Filter for Comments {
    // in document order (the metadata first)
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn block(&mut self, x: &mut Block, _: &mut Ctx) -> Option<Vec<Block>> {
        if let Block::RawBlock(raw) = x
            && raw.format == "html"
        {
            if raw.text.contains(BEGIN) {
                self.hiding = true;
                return Some(vec![]);
            }
            if raw.text.contains(END) {
                self.hiding = false;
                return Some(vec![]);
            }
        }
        self.hiding.then(Vec::new)
    }

    // after a beginning in a note, the rest of its paragraph
    fn inline(&mut self, _: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        self.hiding.then(Vec::new)
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Comments::default(), format));
}
