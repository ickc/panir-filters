//! Numbered paragraphs, and citations of them: a pandoc filter.
//!
//! Each paragraph of the body (not one opening with an image) gets a number,
//! in the margin (LaTeX) or in a span before it, per chapter with
//! `resetAtChapter`. A paragraph opening with `{#name}` can be cited as
//! `[@name]`: the citation becomes a link to its number. `enclosing`,
//! `chapterSep`, `refName` and `addPageNr` shape the numbers and the links;
//! `header-includes` gets the CSS or the LaTeX packages.
//!
//! A port of cysouw/count-para (Copyright © 2021, 2023 Michael Cysouw; ISC;
//! the repository's CC0-1.0, see ../LICENSE).

use std::collections::BTreeMap;

use panir::{
    Attr, Block, Ctx, Filter, Inline, Link, Meta, MetaValue, Pandoc, RawBlock, RawInline, Span,
    Target, stringify, truthy,
};

// (as the original's, its spaces at the ends of lines too)
const CSS: &str = " \n<!-- CSS added by lua-filter 'count-para' -->
<style>
.paragraph-number { \n  float: left;
  margin-left: -5em;
  width: 4.5em;
  text-align: right;
  color: grey;
  font-size: x-small;
  padding-top: 5px;
}
</style>
    ";

const LATEX: [&str; 4] = [
    r"\usepackage{xcolor}",
    r"\usepackage{marginnote}",
    r"\reversemarginpar",
    r"\newcommand{\paragraphnumber}[1]{\marginnote{\color{lightgray}\tiny{#1}}[0pt]}",
];

fn raw_inline(text: String) -> Inline {
    RawInline {
        format: "tex".into(),
        text: text.into(),
    }
    .into()
}

fn meta_blocks(format: &str, text: &str) -> MetaValue {
    Block::from(RawBlock {
        format: format.into(),
        text: text.into(),
    })
    .into()
}

/// The text's `i`th byte, as Lua's `sub` takes it and pandoc reads it back:
/// an ASCII character, else U+FFFD (or nothing past the end).
fn byte(text: &str, i: usize) -> String {
    text.as_bytes()
        .get(i..i + 1)
        .map(|b| String::from_utf8_lossy(b).into_owned())
        .unwrap_or_default()
}

/// What Lua's `s:match("{#(.*)}")` finds: from the first `{#` to the last
/// `}` after it.
fn given_name(text: &str) -> Option<&str> {
    let start = text.find("{#")? + 2;
    let end = text[start..].rfind('}')?;
    Some(&text[start..start + end])
}

/// The first filter: `header-includes`.
struct Formatting;

impl Filter for Formatting {
    fn meta(&mut self, meta: &mut Meta, ctx: &mut Ctx) -> Option<()> {
        let format = ctx.format().unwrap_or_default();
        let mut added = vec![];
        if format.contains("html") {
            added.push(meta_blocks("html", CSS));
        }
        if format.contains("latex") {
            added.extend(LATEX.map(|tex| meta_blocks("tex", tex)));
        }
        // The original appends to what is there as to a Lua table: a list
        // grows, a map's entries are lost (the table becomes a list of the
        // new ones), anything else fails; nothing there is an empty list,
        // even with nothing to add.
        match meta.get_mut("header-includes") {
            Some(MetaValue::MetaList(xs)) => xs.extend(added),
            None => {
                meta.insert("header-includes".into(), added.into());
            }
            Some(_) if added.is_empty() => {}
            Some(MetaValue::MetaMap(_)) => {
                meta.insert("header-includes".into(), added.into());
            }
            Some(other) => {
                return ctx.fail(format!(
                    "count-para: header-includes is {}, not a list",
                    other.tag()
                ));
            }
        }
        Some(())
    }
}

/// The options.
struct Settings {
    reset_at_chapter: bool,
    enclosing: String,
    chapter_sep: String,
    ref_name: String,
    add_page_nr: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            reset_at_chapter: false,
            enclosing: "[]".into(),
            chapter_sep: ".".into(),
            ref_name: "paragraph ".into(),
            add_page_nr: true,
        }
    }
}

/// The second filter: the options, from the metadata.
#[derive(Default)]
struct ReadSettings(Settings);

impl Filter for ReadSettings {
    fn meta(&mut self, meta: &mut Meta, ctx: &mut Ctx) -> Option<()> {
        let s = &mut self.0;
        // (Lua's truth: set and not false)
        if let Some(v) = meta.get("resetAtChapter") {
            s.reset_at_chapter = truthy(Some(v));
        }
        if let Some(v) = meta.get("enclosing") {
            s.enclosing = stringify(v);
        }
        if let Some(v) = meta.get("chapterSep") {
            s.chapter_sep = stringify(v);
        }
        if let Some(v) = meta.get("refName") {
            s.ref_name = stringify(v);
            if ctx.format().unwrap_or_default().contains("latex") && s.ref_name == "#" {
                s.ref_name = "\\#".into();
            }
        }
        if let Some(v) = meta.get("addPageNr") {
            s.add_page_nr = truthy(Some(v));
        }
        Some(())
    }
}

/// The third filter: the numbers.
struct Numbering<'a> {
    settings: &'a Settings,
    numbers: BTreeMap<String, String>, // by the names given
}

impl Filter for Numbering<'_> {
    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) -> Option<()> {
        let s = self.settings;
        let latex = ctx.format().unwrap_or_default().contains("latex");
        let (mut count, mut chapter) = (0, 0);
        for block in &mut doc.blocks {
            // (the body's own paragraphs only)
            let content = match block {
                Block::Header(h)
                    if h.level == 1 && h.attr.classes.first().is_none_or(|c| c != "unnumbered") =>
                {
                    if s.reset_at_chapter {
                        chapter += 1;
                        count = 0;
                    }
                    continue;
                }
                Block::Para(content) => content,
                _ => continue,
            };
            // (an empty paragraph fails, as in the original: it has no first inline)
            let Some(first) = content.first() else {
                return ctx.fail("count-para: an empty paragraph");
            };
            if let Inline::Image(_) = first {
                continue;
            }
            count += 1;
            let number = if s.reset_at_chapter {
                format!("{chapter}{}{count}", s.chapter_sep)
            } else {
                count.to_string()
            };
            let e = &s.enclosing;
            let shown = if e.len() == 1 {
                format!("{e}{number}{e}")
            } else {
                format!("{}{number}{}", byte(e, 0), byte(e, 1))
            };

            // a name given at the start: anywhere in the first inline's text
            let name = given_name(&stringify(first)).map(String::from);
            if let Some(name) = &name {
                self.numbers.insert(name.clone(), number.clone());
                content.remove(0);
                match content.first() {
                    Some(Inline::Space) => drop(content.remove(0)),
                    Some(_) => {}
                    None => return ctx.fail("count-para: a paragraph of a name alone"), // (as the original)
                }
            }

            if latex {
                // after the first word, to keep the number on its page
                let mut tex = format!("\\paragraphnumber{{{shown}}}");
                if let Some(name) = &name {
                    tex = format!("\\hypertarget{{{name}}}{{\n{tex}\\label{{{name}}}}}");
                }
                content.insert(1, raw_inline(tex));
            } else {
                let span = Span {
                    attr: Attr::new(&number, &["paragraph-number"], &[]),
                    content: panir::inlines(&shown),
                };
                content.splice(0..0, [span.into(), Inline::Space]);
            }
        }
        Some(())
    }
}

/// The fourth filter: the citations of a name, the metadata's too.
struct References<'a> {
    settings: &'a Settings,
    numbers: &'a BTreeMap<String, String>,
}

impl Filter for References<'_> {
    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Cite(cite) = x else { return None };
        let Some(citation) = cite.citations.first() else {
            return ctx.fail("count-para: a citation of nothing"); // (as the original)
        };
        let name = citation.id.as_str();
        let number = self.numbers.get(name)?;
        let s = self.settings;
        if ctx.format().unwrap_or_default().contains("latex") {
            let mut tex = format!("{}\\hyperlink{{{name}}}{{{number}}}", s.ref_name);
            if s.add_page_nr {
                tex += &format!(" on page~\\pageref{{{name}}}");
            }
            return Some(vec![raw_inline(tex)]);
        }
        let link = Link {
            attr: Attr::default(),
            content: panir::inlines(&format!("{}{number}", s.ref_name)),
            target: Target::new(&format!("#{number}"), ""),
        };
        Some(vec![link.into()])
    }
}

fn main() {
    panir::filter(|doc, format| {
        panir::apply(doc, &mut Formatting, format)?;
        let mut settings = ReadSettings::default();
        panir::apply(doc, &mut settings, format)?;
        let mut numbering = Numbering {
            settings: &settings.0,
            numbers: BTreeMap::new(),
        };
        panir::apply(doc, &mut numbering, format)?;
        let mut references = References {
            settings: &settings.0,
            numbers: &numbering.numbers,
        };
        panir::apply(doc, &mut references, format)
    });
}
