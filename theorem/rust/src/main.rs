//! Divs of class `theorem` to LaTeX theorem environments, or to numbered
//! theorems in HTML: a pandoc filter.
//!
//! A port of pandocfilters' examples/theorem.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Block, Ctx, Filter, Order};

#[derive(Default)]
struct Theorems {
    count: u32,
}

impl Filter for Theorems {
    // numbered in document order (the metadata first)
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let Block::Div(div) = x else { return None };
        if !div.attr.has_class("theorem") {
            return None;
        }
        match ctx.format() {
            Some("latex") => {
                let label = match &*div.attr.identifier {
                    "" => String::new(),
                    id => format!("\\label{{{id}}}"),
                };
                // what is spliced in is not filtered itself: a theorem
                // directly in this one stays a div, as in the original
                let mut blocks = vec![Block::raw("latex", format!("\\begin{{theorem}}{label}"))];
                blocks.append(&mut div.content);
                blocks.push(Block::raw("latex", "\\end{theorem}"));
                Some(blocks)
            }
            Some("html" | "html5") => {
                self.count += 1;
                let head = [
                    Block::raw("html", format!("<dt>Theorem {}</dt>", self.count)),
                    Block::raw("html", "<dd>"),
                ];
                div.content.splice(0..0, head);
                div.content.push(Block::raw("html", "</dd>\n</dl>"));
                None
            }
            _ => None,
        }
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Theorems::default(), format));
}
