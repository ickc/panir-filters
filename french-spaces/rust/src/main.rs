//! French punctuation's spaces for LaTeX: a pandoc filter. In LaTeX and
//! beamer output, a space after `«`, `“`, `‹` or before `;`, `?`, `!`,
//! `»`, `”`, `›` becomes a thin space (`\thinspace{}`), and one before `:`
//! an unbreakable space (`~`).
//!
//! A port of chdemko/pandoc-latex-french-spaces (Copyright (c) 2018-2024,
//! Christophe Demko; BSD-3-Clause, see ../LICENSE).

use panir::{Ctx, Filter, Inline, RawInline};

const THIN: &str = "\\thinspace{}";

fn text(x: Option<&Inline>) -> &str {
    match x {
        Some(Inline::Str(s)) => s,
        _ => "",
    }
}

struct FrenchSpaces;

impl Filter for FrenchSpaces {
    // a space's neighbours: a function for each list of inlines, as a
    // function's context has no siblings in Rust
    fn inlines(&mut self, xs: &mut Vec<Inline>, ctx: &mut Ctx) {
        if !matches!(ctx.format(), Some("latex" | "beamer")) {
            return;
        }
        for i in 0..xs.len() {
            if !matches!(xs[i], Inline::Space) {
                continue;
            }
            let prev = text(i.checked_sub(1).and_then(|j| xs.get(j)));
            let next = text(xs.get(i + 1));
            let space = if prev.ends_with(['«', '“', '‹']) {
                THIN
            } else if next.starts_with(':') {
                "~"
            } else if next.starts_with([';', '?', '!', '»', '”', '›']) {
                THIN
            } else {
                continue;
            };
            xs[i] = RawInline {
                format: "tex".into(),
                text: space.into(),
            }
            .into();
        }
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut FrenchSpaces, format));
}
