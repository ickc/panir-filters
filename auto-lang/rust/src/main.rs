//! Spans with `lang` (and `dir`) around runs of a mapped Unicode script: a
//! pandoc filter.
//!
//! The metadata maps scripts to languages (`auto-lang: {Hebrew: he, Han:
//! zh-Hant}`); each run of text in a mapped script becomes a span with the
//! language, and `dir=rtl` for a right-to-left script. Spaces, digits and
//! punctuation join a run that goes on after them; a character that
//! Script_Extensions ties to the run's script (`。` to Han) joins a run it
//! touches; a combining mark goes with the character before it. A span that
//! has a `lang` already is left alone, and what is in it.
//!
//! A port of ickc/font's bin/auto-lang.lua (Copyright (c) 2026, Kolen
//! Cheung; BSD-3-Clause, see ../LICENSE), with its table of Unicode 15.0.0's
//! scripts (`script_ranges.rs`, made by ../tables.py from the original's).

mod script_ranges;

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::rc::Rc;

use panir::{Attr, Ctx, Filter, Inline, Meta, MetaValue, Order, Span, stringify};
use script_ranges::{EXT, INHERITED, NAMES, RTL, STRONG, VERSION};

/// The mapped scripts a code point is tied to (by Script_Extensions).
type Ties = Rc<BTreeSet<&'static str>>;

/// A mapped script's language, by the table's name of the script.
type Languages = BTreeMap<&'static str, String>;

fn within(ranges: &[(u32, u32)], cp: u32) -> bool {
    let i = ranges.partition_point(|&(first, _)| first <= cp);
    i > 0 && cp <= ranges[i - 1].1
}

/// The script of a code point's own Script property, if a script.
fn own_script(cp: u32) -> Option<&'static str> {
    let i = STRONG.partition_point(|&(first, _, _)| first <= cp);
    (i > 0 && cp <= STRONG[i - 1].1).then(|| STRONG[i - 1].2)
}

/// What a code point is to a run.
#[derive(Clone)]
enum Kind {
    /// Of a script, its own, mapped or not.
    Script(&'static str),
    /// Tied by Script_Extensions to some mapped scripts.
    Tied(Ties),
    /// Script=Inherited: a mark, going with what precedes it.
    Inherited,
    /// Spaces, digits, punctuation.
    Neutral,
}

#[derive(Clone)]
struct Piece {
    text: String,
    /// A run of a mapped script.
    script: Option<&'static str>,
    tied: Option<Ties>,
    /// Another script's own text.
    foreign: bool,
}

impl Piece {
    fn new(text: String) -> Self {
        Piece {
            text,
            script: None,
            tied: None,
            foreign: false,
        }
    }
}

fn tied_to(script: &str, ties: &Option<Ties>) -> bool {
    ties.as_ref().is_some_and(|t| t.contains(script))
}

/// What is in a list of inlines, for the run it belongs to.
enum Content {
    /// Its one mapped script.
    Script(&'static str),
    /// Nothing of a script; if all it holds is tied text, the scripts all of
    /// it is tied to.
    Neutral(Option<Ties>),
    /// Another script, what isn't prose, or a span with a language.
    Opaque,
}

/// Containers that carry nothing of their own: what is in them decides the
/// run they belong to (a span with a `lang` aside). A note's content is a
/// flow of its own; code, math and raw inlines are not prose.
fn transparent(x: &Inline) -> Option<&Vec<Inline>> {
    match x {
        Inline::Emph(xs)
        | Inline::Underline(xs)
        | Inline::Strong(xs)
        | Inline::Strikeout(xs)
        | Inline::Superscript(xs)
        | Inline::Subscript(xs)
        | Inline::SmallCaps(xs) => Some(xs),
        Inline::Quoted(q) => Some(&q.content),
        Inline::Link(link) => Some(&link.content),
        Inline::Span(span) if span.attr.get("lang").is_none() => Some(&span.content),
        _ => None,
    }
}

struct Tag {
    languages: Languages,
    memo: HashMap<char, Kind>,
}

impl Tag {
    /// Its own script first if mapped; then the ties, even of a code point
    /// of an unmapped script (the Arabic-Indic digits continue a Thaana run).
    fn classify(&mut self, ch: char) -> Kind {
        if let Some(kind) = self.memo.get(&ch) {
            return kind.clone();
        }
        let cp = u32::from(ch);
        let own = own_script(cp);
        let tied: BTreeSet<&'static str> = self
            .languages
            .keys()
            .copied()
            .filter(|s| {
                EXT.binary_search_by_key(s, |&(name, _)| name)
                    .is_ok_and(|i| within(EXT[i].1, cp))
            })
            .collect();
        let kind = match own {
            Some(s) if self.languages.contains_key(s) => Kind::Script(s),
            _ if !tied.is_empty() => Kind::Tied(Rc::new(tied)),
            Some(s) => Kind::Script(s),
            None if within(INHERITED, cp) => Kind::Inherited,
            None => Kind::Neutral,
        };
        self.memo.insert(ch, kind.clone());
        kind
    }

    fn mapped(&self, script: &str) -> bool {
        self.languages.contains_key(script)
    }

    /// A text as maximal runs of one mapped script, and what is between
    /// them: neutral text joins a run only if the run goes on after it; text
    /// tied to a run's script joins the run it touches. Tied text stays in
    /// pieces of one character (with its marks): it may belong to a run in a
    /// neighbouring inline, which only `group` sees.
    fn split(&mut self, text: &str) -> Vec<Piece> {
        let mut pieces: Vec<Piece> = vec![];
        let mut run: Option<Piece> = None;
        let mut pending: Vec<Piece> = vec![]; // each character since the run last took one

        let close = |run: &mut Option<Piece>, pending: &mut Vec<Piece>, pieces: &mut Vec<Piece>| {
            if let Some(mut r) = run.take() {
                let script = r.script.unwrap_or_default();
                let n = pending
                    .iter()
                    .take_while(|p| tied_to(script, &p.tied))
                    .count();
                r.text.extend(pending.drain(..n).map(|p| p.text));
                pieces.push(r);
            }
        };

        for ch in text.chars() {
            match self.classify(ch) {
                Kind::Script(script) if self.mapped(script) => match &mut run {
                    Some(r) if r.script == Some(script) => {
                        r.text.extend(pending.drain(..).map(|p| p.text));
                        r.text.push(ch);
                    }
                    _ => {
                        close(&mut run, &mut pending, &mut pieces);
                        let first = pending.len()
                            - pending
                                .iter()
                                .rev()
                                .take_while(|p| tied_to(script, &p.tied))
                                .count();
                        let mut text: String = pending.drain(first..).map(|p| p.text).collect();
                        text.push(ch);
                        pieces.append(&mut pending);
                        run = Some(Piece {
                            script: Some(script),
                            ..Piece::new(text)
                        });
                    }
                },
                Kind::Script(_) => {
                    // another script's own text ends the run, apart from the
                    // neutral text around it
                    close(&mut run, &mut pending, &mut pieces);
                    pieces.append(&mut pending);
                    pieces.push(Piece {
                        foreign: true,
                        ..Piece::new(ch.into())
                    });
                }
                Kind::Inherited => {
                    // with its base: the last pending character, or the run
                    if let Some(p) = pending.last_mut() {
                        p.text.push(ch);
                    } else if let Some(r) = &mut run {
                        r.text.push(ch);
                    } else {
                        pending.push(Piece::new(ch.into()));
                    }
                }
                Kind::Tied(ties) => pending.push(Piece {
                    tied: Some(ties),
                    ..Piece::new(ch.into())
                }),
                Kind::Neutral => pending.push(Piece::new(ch.into())),
            }
        }
        close(&mut run, &mut pending, &mut pieces);
        pieces.append(&mut pending);

        let mut merged: Vec<Piece> = vec![];
        for p in pieces {
            match merged.last_mut() {
                Some(last)
                    if last.script.is_none()
                        && p.script.is_none()
                        && last.tied.is_none()
                        && p.tied.is_none()
                        && last.foreign == p.foreign =>
                {
                    last.text.push_str(&p.text);
                }
                _ => merged.push(p),
            }
        }
        merged
    }

    fn content(&mut self, inlines: &[Inline]) -> Content {
        let mut found: Option<&'static str> = None;
        let mut ties: Option<BTreeSet<&'static str>> = None;
        let mut untied = false;
        if !self.scan(inlines, &mut found, &mut ties, &mut untied) {
            return Content::Opaque;
        }
        match (found, ties) {
            (Some(script), _) => Content::Script(script),
            (None, Some(t)) if !untied && !t.is_empty() => Content::Neutral(Some(Rc::new(t))),
            _ => Content::Neutral(None),
        }
    }

    /// Reads the text of a list, false as soon as it is opaque.
    fn scan(
        &mut self,
        items: &[Inline],
        found: &mut Option<&'static str>,
        ties: &mut Option<BTreeSet<&'static str>>,
        untied: &mut bool,
    ) -> bool {
        for x in items {
            match x {
                Inline::Str(text) => {
                    for ch in text.chars() {
                        match self.classify(ch) {
                            Kind::Script(s) => {
                                if !self.mapped(s) || found.is_some_and(|f| f != s) {
                                    return false;
                                }
                                *found = Some(s);
                            }
                            Kind::Tied(t) => {
                                *ties = Some(match ties.take() {
                                    None => (*t).clone(),
                                    Some(before) => before.intersection(&t).copied().collect(),
                                });
                            }
                            Kind::Neutral => *untied = true,
                            Kind::Inherited => {} // (with its base: it neither ties nor unties)
                        }
                    }
                }
                Inline::Space | Inline::SoftBreak | Inline::LineBreak => *untied = true,
                _ => match transparent(x) {
                    Some(content) => {
                        if !self.scan(content, found, ties, untied) {
                            return false;
                        }
                    }
                    None => return false,
                },
            }
        }
        true
    }

    fn span(&self, script: &str, content: Vec<Inline>) -> Inline {
        let mut attributes = vec![("lang", self.languages[script].as_str())];
        if RTL.binary_search(&script).is_ok() {
            attributes.push(("dir", "rtl"));
        }
        Span {
            attr: Attr::new("", &[], &attributes),
            content,
        }
        .into()
    }

    /// Whether `group` would make a span: a run of a mapped script at the
    /// top of the list, or a container of nothing but one. (Where it would
    /// make none, the list stays as it was, its texts unsplit.)
    fn has_run(&mut self, inlines: &[Inline]) -> bool {
        for x in inlines {
            let run = match x {
                Inline::Str(text) => text
                    .chars()
                    .any(|ch| matches!(self.classify(ch), Kind::Script(s) if self.mapped(s))),
                _ => transparent(x)
                    .is_some_and(|content| matches!(self.content(content), Content::Script(_))),
            };
            if run {
                return true;
            }
        }
        false
    }

    /// The list with its runs in spans, a run going on across spaces and the
    /// containers of nothing but its script; the same if there is none.
    fn group(&mut self, inlines: &mut Vec<Inline>) {
        // A run with the whole list to itself takes the neutral text around it.
        if let Content::Script(script) = self.content(inlines) {
            *inlines = vec![self.span(script, std::mem::take(inlines))];
            return;
        }
        if !self.has_run(inlines) {
            return;
        }
        let mut grouping = Grouping::default();
        for x in std::mem::take(inlines) {
            match x {
                Inline::Space | Inline::SoftBreak => grouping.add(self, x, None, None),
                Inline::Str(text) => {
                    for piece in self.split(&text) {
                        if piece.foreign {
                            grouping.close(self);
                            grouping.out.push(Inline::Str(piece.text.into()));
                        } else {
                            grouping.add(
                                self,
                                Inline::Str(piece.text.into()),
                                piece.script,
                                piece.tied,
                            );
                        }
                    }
                }
                _ => match transparent(&x).map(|content| self.content(content)) {
                    Some(Content::Script(s)) => grouping.add(self, x, Some(s), None),
                    Some(Content::Neutral(ties)) => grouping.add(self, x, None, ties),
                    Some(Content::Opaque) | None => {
                        grouping.close(self);
                        grouping.out.push(x);
                    }
                },
            }
        }
        grouping.close(self);
        *inlines = grouping.out;
    }
}

/// `group`'s state: what is out, the run being made, and what is pending
/// since the run last took something.
#[derive(Default)]
struct Grouping {
    out: Vec<Inline>,
    run: Vec<Inline>,
    script: Option<&'static str>,
    pending: Vec<(Inline, Option<Ties>)>,
}

impl Grouping {
    fn close(&mut self, tag: &Tag) {
        if let Some(script) = self.script.take() {
            let n = self
                .pending
                .iter()
                .take_while(|(_, t)| tied_to(script, t))
                .count();
            self.run.extend(self.pending.drain(..n).map(|(x, _)| x));
            let run = std::mem::take(&mut self.run);
            self.out.push(tag.span(script, run));
        }
        self.out.extend(self.pending.drain(..).map(|(x, _)| x));
    }

    fn add(&mut self, tag: &Tag, x: Inline, of: Option<&'static str>, tied: Option<Ties>) {
        match of {
            None => self.pending.push((x, tied)),
            Some(s) if Some(s) == self.script => {
                self.run.extend(self.pending.drain(..).map(|(x, _)| x));
                self.run.push(x);
            }
            Some(s) => {
                // the first run, or another script's: the tied text at the end
                // of what is pending is the new run's
                let first = self.pending.len()
                    - self
                        .pending
                        .iter()
                        .rev()
                        .take_while(|(_, t)| tied_to(s, t))
                        .count();
                let carried: Vec<Inline> = self.pending.drain(first..).map(|(x, _)| x).collect();
                self.close(tag);
                self.script = Some(s);
                self.run.extend(carried);
                self.run.push(x);
            }
        }
    }
}

impl Filter for Tag {
    fn order(&self) -> Order {
        Order::Topdown
    }

    /// A language written by hand wins: the span and what is in it stay.
    fn inline(&mut self, x: &mut Inline, ctx: &mut Ctx) -> Option<Vec<Inline>> {
        if let Inline::Span(span) = x
            && span.attr.get("lang").is_some()
        {
            ctx.skip_children();
        }
        None
    }

    fn inlines(&mut self, xs: &mut Vec<Inline>, _: &mut Ctx) -> Option<()> {
        if !self.languages.is_empty() {
            self.group(xs);
        }
        Some(())
    }
}

/// `auto-lang`, the scripts' languages. In a filter of its own, as a
/// filter's metadata function runs after its inlines'.
#[derive(Default)]
struct Configure(Languages);

impl Filter for Configure {
    fn meta(&mut self, meta: &mut Meta, ctx: &mut Ctx) -> Option<()> {
        let scripts: Vec<(String, Option<&MetaValue>)> = match meta.get("auto-lang") {
            None | Some(MetaValue::MetaBool(false)) => vec![],
            Some(MetaValue::MetaMap(map)) => {
                map.iter().map(|(k, v)| (k.to_string(), Some(v))).collect()
            }
            // (Lua's `pairs` over a list: its indices, none a script's name)
            Some(MetaValue::MetaList(xs)) => {
                (1..=xs.len()).map(|i| (i.to_string(), None)).collect()
            }
            Some(MetaValue::MetaInlines(xs)) => {
                (1..=xs.len()).map(|i| (i.to_string(), None)).collect()
            }
            Some(MetaValue::MetaBlocks(xs)) => {
                (1..=xs.len()).map(|i| (i.to_string(), None)).collect()
            }
            Some(other) => {
                return ctx.fail(format!(
                    "auto-lang: the map of scripts to languages is {}",
                    other.tag()
                ));
            }
        };
        for (script, language) in scripts {
            match (NAMES.binary_search(&script.as_str()), language) {
                (Ok(i), Some(language)) => {
                    self.0.insert(NAMES[i], stringify(language));
                }
                _ => eprintln!(
                    "[WARNING] auto-lang: no Unicode script named {script} (Unicode {VERSION})"
                ),
            }
        }
        Some(())
    }
}

fn main() {
    panir::filter(|doc, format| {
        let mut configure = Configure::default();
        panir::apply(doc, &mut configure, format)?;
        let mut tag = Tag {
            languages: configure.0,
            memo: HashMap::new(),
        };
        panir::apply(doc, &mut tag, format)
    });
}
