//! LaTeX page breaks to the output format's: a pandoc filter.
//!
//! A raw TeX block holding only `\newpage` or `\pagebreak` becomes the
//! format's page break (a raw block, or a paragraph of a form feed in
//! formats without one); with `pagebreak.break-on.form-feed`, so does a
//! paragraph of a form feed alone, and with `break-on.plaintext-command`, a
//! paragraph of the command as text. `html-class` and `odt-style` (or
//! `$PANDOC_PAGEBREAK_HTML_CLASS`, `$PANDOC_PAGEBREAK_ODT_STYLE`) style the
//! HTML and ODT ones; the configuration is removed.
//!
//! A port of pandoc-ext/pagebreak (Copyright © 2017-2025 Benct Philip
//! Jonsson, Albert Krewinkel; ISC; the repository's MIT licence, see
//! ../LICENSE).

use panir::{
    Block, Ctx, Filter, Inline, Meta, MetaValue, Pandoc, RawBlock, lookup, stringify, truthy,
};

const FORM_FEED: &str = "\u{c}";

/// `\newpage` or `\pagebreak`, perhaps with `{`, `}` or both after.
fn is_newpage(text: &str) -> bool {
    let rest = text
        .strip_prefix("\\newpage")
        .or_else(|| text.strip_prefix("\\pagebreak"));
    matches!(rest, Some("" | "{" | "}" | "{}"))
}

/// A text option: the metadata's, else the environment's. (The top-level
/// field: the original reads `html-class`, not `pagebreak.html-class` as
/// its README says.)
fn option(meta: &Meta, key: &str, variable: &str) -> Option<String> {
    if truthy(meta.get(key)) {
        Some(stringify(&meta[key]))
    } else {
        std::env::var(variable).ok()
    }
}

fn raw(format: &str, text: &str) -> Block {
    RawBlock {
        format: format.into(),
        text: text.into(),
    }
    .into()
}

/// A page break in the format.
fn pagebreak(format: &str, meta: &Meta) -> Block {
    if format.contains("asciidoc") {
        raw("asciidoc", "<<<\n\n")
    } else if format == "context" {
        raw("context", "\\page")
    } else if format == "docx" {
        raw("openxml", "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>")
    } else if format.contains("epub") {
        raw("html", "<p style=\"page-break-after: always;\"> </p>")
    } else if format.contains("html") {
        match option(meta, "html-class", "PANDOC_PAGEBREAK_HTML_CLASS") {
            Some(class) if !class.is_empty() => {
                raw("html", &format!("<div class=\"{class}\"></div>"))
            }
            _ => raw("html", "<div style=\"page-break-after: always;\"></div>"),
        }
    } else if format.contains("latex") {
        raw("tex", "\\newpage{}")
    } else if format.contains("ms") {
        raw("ms", ".bp")
    } else if format.contains("odt") {
        let style = option(meta, "odt-style", "PANDOC_PAGEBREAK_ODT_STYLE")
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| "Pagebreak".into());
        raw(
            "opendocument",
            &format!("<text:p text:style-name=\"{style}\"/>"),
        )
    } else if format.contains("typst") {
        raw("typst", "#pagebreak()\n\n")
    } else {
        Block::Para(vec![FORM_FEED.into()]) // a form feed, in any other format
    }
}

/// The page break in place of each marker.
struct Breaks {
    pagebreak: Block,
    raw_tex: bool, // (in LaTeX, raw TeX stays as it is)
    form_feed: bool,
    plaintext: bool,
}

impl Filter for Breaks {
    fn block(&mut self, x: &mut Block, _: &mut Ctx) -> Option<Vec<Block>> {
        let marker = match x {
            Block::RawBlock(r) => self.raw_tex && r.format.contains("tex") && is_newpage(&r.text),
            Block::Para(content) => match content.as_slice() {
                [Inline::Str(text)] => {
                    (self.form_feed && text == FORM_FEED) || (self.plaintext && is_newpage(text))
                }
                _ => false,
            },
            _ => false,
        };
        marker.then(|| vec![self.pagebreak.clone()])
    }
}

struct Pagebreak;

impl Filter for Pagebreak {
    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) -> Option<()> {
        let format = ctx.format().unwrap_or_default();
        let config = |path: &[&str]| lookup(&doc.meta, path);
        let is_true = |v: Option<&MetaValue>| matches!(v, Some(MetaValue::MetaBool(true)));
        if is_true(config(&["pagebreak"])) || is_true(config(&["pagebreak", "break-on"])) {
            return ctx.fail("pagebreak: a configuration of true"); // (Lua indexes a boolean)
        }
        let mut breaks = Breaks {
            pagebreak: pagebreak(format, &doc.meta),
            raw_tex: !format.ends_with("tex"),
            form_feed: truthy(config(&["pagebreak", "break-on", "form-feed"])),
            plaintext: truthy(config(&["pagebreak", "break-on", "plaintext-command"])),
        };
        doc.meta.remove("pagebreak");
        ctx.apply(doc, &mut breaks) // the metadata's blocks too
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut Pagebreak, format));
}
