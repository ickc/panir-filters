//! Notes to Markdown's inline notes (`^[...]`): a pandoc filter, for
//! Markdown output. A note's paragraphs are joined by a space; a note
//! holding any other block (a list, code, a quote...) stays as it is.
//!
//! A port of pandocfilters' examples/inlinenotes.py (Copyright (c) 2013,
//! John MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Block, Ctx, Filter, Inline, RawInline};

struct InlineNotes;

impl Filter for InlineNotes {
    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Note(blocks) = x else {
            return None;
        };
        let inlinable =
            |b: &Block| matches!(b, Block::Para(_) | Block::Plain(_) | Block::HorizontalRule);
        if ctx.format() != Some("markdown") || !blocks.iter().all(inlinable) {
            return None;
        }
        let mut inlines = Vec::new();
        for block in std::mem::take(blocks) {
            match block {
                Block::Para(xs) => {
                    if !inlines.is_empty() {
                        inlines.push(Inline::Space);
                    }
                    inlines.extend(xs);
                }
                Block::Plain(xs) => inlines.extend(xs), // as the original: no space before it
                _ => {}                                 // a rule: dropped, as in the original
            }
        }
        let open = RawInline {
            format: "html".into(),
            text: "^[".into(),
        };
        let mut note = vec![open.into()];
        note.append(&mut inlines);
        note.push(Inline::Str("]".into()));
        Some(note)
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut InlineNotes, format));
}
