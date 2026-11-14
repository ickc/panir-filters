//! Affiliations and correspondence as blocks before the body: a pandoc
//! filter, run after scholarly-metadata (whose `author` and `institute` it
//! reads).
//!
//! The body starts with a note on equal contributors (if any), the
//! affiliations, numbered, and the corresponding authors (`correspondence`
//! and an `email`), linked; `author` becomes one line of names with their
//! marks (in LaTeX, a list, one each), and `institute` goes.
//!
//! A port of pandoc/lua-filters' author-info-blocks (Copyright © 2017–2021
//! Albert Krewinkel; ISC; the repository's MIT licence, see ../LICENSE).

use std::collections::BTreeMap;

use panir::{
    Attr, Block, Ctx, Filter, Inline, Link, MetaValue, Pandoc, RawInline, Target, Text, stringify,
};

type Map = BTreeMap<Text, MetaValue>;

#[derive(Clone, Copy)]
enum Mark {
    EqualContributor,
    CorrespondingAuthor,
}

/// Whether Lua counts the field: there, and not false.
fn given(author: &Map, key: &str) -> bool {
    !matches!(author.get(key), None | Some(MetaValue::MetaBool(false)))
}

fn corresponding(author: &Map) -> bool {
    given(author, "correspondence") && given(author, "email")
}

fn items(value: Option<&MetaValue>) -> Result<&[MetaValue], String> {
    match value {
        Some(MetaValue::MetaList(items)) => Ok(items),
        other => Err(format!(
            "author-info-blocks: expected a list, got {:?}",
            other.map(MetaValue::tag)
        )),
    }
}

/// `author` or `institute`, as scholarly-metadata leaves them.
fn maps(value: Option<MetaValue>) -> Result<Vec<Map>, String> {
    let Some(MetaValue::MetaList(items)) = value else {
        return Err("author-info-blocks: expected a list".into());
    };
    items
        .into_iter()
        .map(|x| match x {
            MetaValue::MetaMap(map) => Ok(map),
            other => Err(format!(
                "author-info-blocks: expected a map, got {}",
                other.tag()
            )),
        })
        .collect()
}

/// A name or an email: inlines (the original fails on anything else).
fn inlines<'a>(map: &'a Map, key: &str) -> Result<&'a Vec<Inline>, String> {
    match map.get(key) {
        Some(MetaValue::MetaInlines(xs)) => Ok(xs),
        other => Err(format!(
            "author-info-blocks: {key}: expected inlines, got {:?}",
            other.map(MetaValue::tag)
        )),
    }
}

fn intercalate(lists: Vec<Vec<Inline>>, sep: impl Fn() -> Vec<Inline>) -> Vec<Inline> {
    let mut out = vec![];
    for (i, xs) in lists.into_iter().enumerate() {
        if i > 0 {
            out.extend(sep());
        }
        out.extend(xs);
    }
    out
}

struct AuthorInfoBlocks;

impl AuthorInfoBlocks {
    fn info_blocks(doc: &mut Pandoc, format: &str) -> Result<(), String> {
        let mark = |m: Mark| -> Inline {
            match (m, format == "latex") {
                (Mark::CorrespondingAuthor, true) => RawInline {
                    format: "latex".into(),
                    text: "*".into(),
                }
                .into(),
                (Mark::CorrespondingAuthor, false) => "✉".into(),
                (Mark::EqualContributor, true) => RawInline {
                    format: "latex".into(),
                    text: "$\\dagger{}$".into(),
                }
                .into(),
                (Mark::EqualContributor, false) => "*".into(),
            }
        };
        // the name, with its marks as a superscript
        let author_line = |author: &Map| -> Result<Vec<Inline>, String> {
            let mut marks = vec![];
            if given(author, "equal_contributor") {
                marks.push(vec![mark(Mark::EqualContributor)]);
            }
            for i in items(author.get("institute"))? {
                marks.push(vec![Inline::Str(stringify(i).into())]);
            }
            if corresponding(author) {
                marks.push(vec![mark(Mark::CorrespondingAuthor)]);
            }
            let mut line = inlines(author, "name")?.clone();
            line.push(Inline::Superscript(intercalate(marks, || vec![",".into()])));
            Ok(line)
        };

        let authors = maps(doc.meta.remove("author"))?;
        let institutes = match doc.meta.remove("institute") {
            None => vec![],
            some => maps(some)?,
        };

        let mut body = vec![];
        if authors.iter().any(|a| given(a, "equal_contributor")) {
            body.push(Block::Para(vec![
                Inline::Superscript(vec![mark(Mark::EqualContributor)]),
                Inline::Space,
                "These authors contributed equally to this work.".into(),
            ]));
        }
        let mut lines = vec![];
        for (i, inst) in institutes.iter().enumerate() {
            let mut line = vec![
                Inline::Superscript(vec![Inline::Str((i + 1).to_string().into())]),
                Inline::Space,
            ];
            line.extend(inlines(inst, "name")?.iter().cloned());
            lines.push(line);
        }
        body.push(Block::Para(intercalate(lines, || vec![Inline::LineBreak])));
        let mut links = vec![];
        for author in authors.iter().filter(|a| corresponding(a)) {
            let email = inlines(author, "email")?;
            let mut content = inlines(author, "name")?.clone();
            content.extend([Inline::Space, "<".into()]);
            content.extend(email.iter().cloned());
            content.push(">".into());
            let target = Target {
                url: format!("mailto:{}", stringify(email)).into(),
                title: "".into(),
            };
            links.push(vec![
                Link {
                    attr: Attr::default(),
                    content,
                    target,
                }
                .into(),
            ]);
        }
        if !links.is_empty() {
            let mut para = vec![
                Inline::Superscript(vec![mark(Mark::CorrespondingAuthor)]),
                Inline::Space,
                "Correspondence:".into(),
                Inline::Space,
            ];
            para.extend(intercalate(links, || vec![",".into(), Inline::Space]));
            body.push(Block::Para(para));
        }
        body.append(&mut doc.blocks);
        doc.blocks = body;

        let mut names = authors
            .iter()
            .map(author_line)
            .collect::<Result<Vec<_>, _>>()?;
        let author = if format.contains("latex") {
            // (the marks above ask for "latex" exactly)
            MetaValue::MetaList(names.into_iter().map(MetaValue::MetaInlines).collect())
        } else {
            let last = names.pop().ok_or("author-info-blocks: no authors")?; // (the original fails too)
            let several = !names.is_empty();
            let mut line = intercalate(names, || vec![",".into(), Inline::Space]);
            if several {
                line.extend([",".into(), Inline::Space, "and".into(), Inline::Space]);
            }
            line.extend(last);
            MetaValue::MetaInlines(line)
        };
        doc.meta.insert("author".into(), author);
        Ok(())
    }
}

impl Filter for AuthorInfoBlocks {
    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) {
        if let Err(e) = Self::info_blocks(doc, ctx.format().unwrap_or_default()) {
            ctx.fail::<()>(e);
        }
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut AuthorInfoBlocks, format));
}
