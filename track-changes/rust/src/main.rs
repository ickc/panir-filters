//! Tracked changes (Word's, as pandoc reads them with `--track-changes=all`):
//! accepted, rejected, or shown: a pandoc filter.
//!
//! The metadata's `trackChanges` (`accept`, the default, `reject` or `all`)
//! decides. Accepting keeps insertions and drops deletions and comments;
//! rejecting the reverse; `all` shows them in HTML (`ins`, `del`, `mark`) and
//! LaTeX (the `changes` package, in `header-includes`, with a colour per
//! author), and in Word and ODT keeps them, a comment's `¶` a line break.
//!
//! A port of pandoc/lua-filters' track-changes (Copyright (c) 2017-2021
//! pandoc Lua filters contributors; MIT, see ../LICENSE).

use std::collections::BTreeMap;

use panir::{Block, Ctx, Filter, Inline, MetaValue, Pandoc, RawBlock, RawInline, Span, stringify};

const HEADER: &str = r"
\makeatletter
\PassOptionsToPackage{textsize=scriptsize}{todonotes}
\PassOptionsToPackage{markup=underlined,authormarkup=none,commentmarkup=todo}{changes}
\usepackage{changes}
\@ifpackagelater{changes}{2018/11/03}{%
}{%
  \usepackage{todonotes}
  \setremarkmarkup{\todo[color=Changes@Color#1!20]{\sffamily\textbf{#1:}~#2}}
}%
\makeatother
\definecolor{auth1}{HTML}{4477AA}
\definecolor{auth2}{HTML}{117733}
\definecolor{auth3}{HTML}{999933}
\definecolor{auth4}{HTML}{CC6677}
\definecolor{auth5}{HTML}{AA4499}
\definecolor{auth6}{HTML}{332288}
\setlength{\marginparwidth}{3cm}
\newcommand{\note}[2][]{\added[#1,remark={#2}]{}}
\newcommand\hlnotesingle{%
  \bgroup
  \expandafter\def\csname sout\space\endcsname{\bgroup \ULdepth =-.8ex \ULset}%
  \markoverwith{\textcolor{yellow}{\rule[-.5ex]{.1pt}{2.5ex}}}%
  \ULon}
\newcommand\hlnote[1]{\let\helpcmd\hlnotesingle\parhelp#1\par\relax\relax}
\long\def\parhelp#1\par#2\relax{%
  \helpcmd{#1}\ifx\relax#2\else\par\parhelp#2\relax\fi%
}

\makeatletter
\newcommand\ifmoving{%
  \ifx\protect\@unexpandable@protect
      \expandafter\@firstoftwo
  \else
      \expandafter\@secondoftwo
  \fi
}

\newcommand{\gobbletwo}[2][]{\@bsphack\@esphack}
\newcommand{\gobbleone}[1][]{\@bsphack\@esphack}

\let\oldadded\added
\let\olddeleted\deleted
\let\oldhlnote\hlnote
\let\oldnote\note
\renewcommand{\added}{\ifmoving{\gobbleone}{\oldadded}}
\renewcommand{\deleted}{\ifmoving{\gobbletwo}{\olddeleted}}
\renewcommand{\hlnote}{\ifmoving{}{\oldhlnote}}
\renewcommand{\note}{\ifmoving{\gobbletwo}{\oldnote}}
\makeatother
";

const TITLES: &[&str] = &["dr", "mr", "ms", "mrs", "prof", "mx", "sir"];

/// A name's initials (ASCII letters and digits: Lua's %w), titles left out;
/// a name of one word as it is.
fn initials(name: &str) -> String {
    if !name.contains(' ') {
        return name.into();
    }
    name.split(|c: char| !(c.is_ascii_alphanumeric() || c == '\''))
        .filter(|w| !w.is_empty() && !TITLES.contains(&w.to_ascii_lowercase().as_str()))
        .map(|w| w[..1].to_ascii_uppercase())
        .collect()
}

fn raw(format: &str, text: String) -> Inline {
    RawInline {
        format: format.into(),
        text: text.into(),
    }
    .into()
}

/// A comment's `¶` (a paragraph break in Word) as another inline.
struct Pilcrows(fn() -> Inline);

impl Filter for Pilcrows {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        matches!(x, Inline::Str(s) if s == "¶").then(|| vec![(self.0)()])
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Accept,
    Reject,
    Html,
    Tex,
    Word,
}

/// What a mode does to each tracked change.
struct Changes {
    mode: Mode,
    authors: BTreeMap<String, String>, // by initials, those of the changes shown
}

impl Changes {
    /// The author's initials, the author kept by them.
    fn author(&mut self, span: &Span) -> Result<String, &'static str> {
        // (the original fails without one too)
        let name = span
            .attr
            .get("author")
            .ok_or("track-changes: a change without an author")?;
        self.authors.insert(initials(name), name.into());
        Ok(initials(name))
    }

    fn tex(&mut self, span: &mut Span, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let command = match span.attr.classes.first().map(|c| c.as_str()) {
            Some("comment-start") => "\\note",
            Some("insertion") => "\\added",
            Some("deletion") => "\\deleted",
            _ if span.attr.has_class("comment-end") => return Some(vec![raw("latex", "}".into())]),
            _ => return None,
        };
        let id = match self.author(span) {
            Ok(id) => id,
            Err(e) => return ctx.fail(e),
        };
        let text = if span.attr.has_class("comment-start") {
            ctx.apply(&mut span.content, &mut Pilcrows(|| "\\newline".into()))?;
            format!("{}}}\\hlnote{{", stringify(&span.content))
        } else {
            format!("{}}}", stringify(&span.content))
        };
        Some(vec![raw("latex", format!("{command}[id={id}]{{{text}"))])
    }

    fn html(&mut self, span: &mut Span, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let tag = match span.attr.classes.first().map(|c| c.as_str()) {
            Some("comment-start") => "mark",
            Some("insertion") => "ins",
            Some("deletion") => "del",
            _ if span.attr.has_class("comment-end") => {
                return Some(vec![raw("html", "</mark>".into())]);
            }
            _ => return None,
        };
        if let Err(e) = self.author(span) {
            return ctx.fail(e);
        }
        let mut s = format!("<{tag}");
        for (key, value) in &span.attr.attributes {
            let key = if key == "date" {
                key.to_string()
            } else {
                format!("data-{key}")
            };
            s += &format!(" {key}=\"{value}\"");
        }
        if span.attr.has_class("comment-start") {
            s += &format!(" data-id=\"{}\"", span.attr.identifier); // (Lua's "" is true: always)
            ctx.apply(&mut span.content, &mut Pilcrows(|| "&#10;".into()))?;
            s += &format!(" title=\"{}\">", stringify(&span.content));
        } else {
            s += &format!(">{}</{tag}>", stringify(&span.content));
        }
        Some(vec![raw("html", s)])
    }

    /// Accepting or rejecting: comments go, an insertion becomes its
    /// content (accepted) or goes, a deletion the reverse; a span of both is
    /// an insertion.
    fn resolve(span: &mut Span, accept: bool) -> Option<Vec<Inline>> {
        let has = |c| span.attr.has_class(c);
        let keep = if has("comment-start") || has("comment-end") {
            false
        } else if has("insertion") {
            accept
        } else if has("deletion") {
            !accept
        } else {
            return None;
        };
        Some(if keep {
            std::mem::take(&mut span.content)
        } else {
            vec![]
        })
    }
}

impl Filter for Changes {
    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Span(span) = x else { return None };
        match self.mode {
            Mode::Accept => Self::resolve(span, true),
            Mode::Reject => Self::resolve(span, false),
            Mode::Html => self.html(span, ctx),
            Mode::Tex => self.tex(span, ctx),
            Mode::Word if span.attr.has_class("comment-start") => {
                ctx.apply(&mut span.content, &mut Pilcrows(|| Inline::LineBreak))?;
                None
            }
            Mode::Word => None,
        }
    }
}

/// `header-includes` with the `changes` package and a colour for each
/// author: the original wraps what is there in a list, a list too (its test
/// for a list, of `.t`, fails since pandoc 2.17).
fn header_includes(existing: Option<MetaValue>, authors: &BTreeMap<String, String>) -> MetaValue {
    let latex = |text: String| -> MetaValue {
        Block::from(RawBlock {
            format: "latex".into(),
            text: text.into(),
        })
        .into()
    };
    let mut out: Vec<MetaValue> = existing.into_iter().collect();
    out.push(latex(HEADER.into()));
    for (i, (inits, name)) in authors.iter().enumerate() {
        let color = i + 1;
        out.push(latex(format!(
            "\\definechangesauthor[name={{{name}}}, color=auth{color}]{{{inits}}}"
        )));
    }
    out.into()
}

struct TrackChanges;

impl Filter for TrackChanges {
    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) -> Option<()> {
        // (the original reads the reader's option first, as pandoc 2 named
        // it: since pandoc 3 never there, so the metadata decides)
        let mode = doc.meta.remove("trackChanges").map(|v| stringify(&v));
        let format = ctx.format().unwrap_or_default();
        let tex = matches!(format, "latex" | "tex" | "context");
        let mode = match mode.as_deref() {
            Some("all") if matches!(format, "html" | "html4" | "html5") => Mode::Html,
            Some("all") if tex => Mode::Tex,
            Some("all") if matches!(format, "docx" | "odt") => Mode::Word,
            Some("all") => return Some(()),
            Some("reject") => Mode::Reject,
            _ => Mode::Accept,
        };
        let mut changes = Changes {
            mode,
            authors: BTreeMap::new(),
        };
        ctx.apply(&mut doc.blocks, &mut changes)?; // (not the metadata)
        if mode == Mode::Tex {
            let existing = doc.meta.remove("header-includes");
            doc.meta.insert(
                "header-includes".into(),
                header_includes(existing, &changes.authors),
            );
        }
        Some(())
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut TrackChanges, format));
}
