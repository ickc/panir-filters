//! Definition lists to bullet lists, each term in bold before its
//! definitions: a pandoc filter, for Markdown without definition lists.
//!
//! A port of pandocfilters' examples/deflists.py (Copyright (c) 2013, John
//! MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Block, Ctx, Filter, Inline};

struct Deflists;

impl Filter for Deflists {
    fn block(&mut self, x: &mut Block, _: &mut Ctx) -> Option<Vec<Block>> {
        let Block::DefinitionList(items) = x else {
            return None;
        };
        let items = std::mem::take(items)
            .into_iter()
            .map(|(term, defs)| {
                std::iter::once(Block::Para(vec![Inline::Strong(term)]))
                    .chain(defs.into_iter().flatten())
                    .collect()
            })
            .collect();
        Some(vec![Block::BulletList(items)])
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Deflists, format));
}
