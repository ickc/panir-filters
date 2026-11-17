//! amsthm's theorem and proof environments for any output format: a pandoc
//! filter.
//!
//! The metadata declares the environments (`amsthm: {plain: [Theorem:
//! [Lemma]], definition: [Definition], ...}`, with options); a div of one's
//! class is one. In LaTeX (and beamer) they are amsthm's own: `\newtheorem`s
//! in `header-includes`, `\begin{...}...\end{...}` around each div, and
//! citations of their identifiers `\ref`s. In any other format the filter
//! numbers them as amsthm would (counters shared, within sections), sets
//! their headings and bodies in their style's fonts, ends proofs with a
//! symbol, and makes citations of them links.
//!
//! A div's `info` is its note, `Theorem 1 (note)`, which the original reads
//! as markdown with pandoc. This port has no markdown reader: a note of
//! plain text (words of letters and digits, a hyphen inside a word, and
//! `,;:?!` after one, single spaces between) is read as pandoc reads it; any
//! other is kept as its text, with a warning.
//!
//! A port of ickc/pandoc-amsthm's amsthm.lua (Copyright (c) 2016-2026, Kolen
//! Cheung; BSD-3-Clause, see ../LICENSE).

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt::Display;

use panir::{
    Attr, Block, CitationMode, Cite, Conversion, Ctx, Div, Filter, Inline, Link, Math, MathType,
    Meta, MetaValue, Order, Pandoc, Span, Target, inlines_of, items, lua_number, stringify,
    to_roman_numeral,
};

/// LaTeX's sectioning units, by depth: parent_counter's values.
const PARENT_COUNTERS: [(&str, i64); 7] = [
    ("part", 0),
    ("chapter", 1),
    ("section", 2),
    ("subsection", 3),
    ("subsubsection", 4),
    ("paragraph", 5),
    ("subparagraph", 6),
];
/// Document classes whose level-1 headings pandoc's LaTeX writer makes chapters.
const CHAPTER_CLASSES: [&str; 9] = [
    "memoir",
    "book",
    "report",
    "scrreprt",
    "scrreport",
    "scrbook",
    "extreport",
    "extbook",
    "tufte-book",
];
/// Keys of the metadata that are options, not styles.
const OPTION_KEYS: [&str; 8] = [
    "styles",
    "name_to_text",
    "parent_counter",
    "counter_depth",
    "counter_ignore_headings",
    "css",
    "swapnumbers",
    "qed_symbol",
];
const LATEX_LIKE: [&str; 2] = ["latex", "beamer"];
/// Formats whose header-includes are HTML.
const HTML_LIKE: [&str; 12] = [
    "html",
    "html4",
    "html5",
    "chunkedhtml",
    "epub",
    "epub2",
    "epub3",
    "revealjs",
    "slidy",
    "slideous",
    "dzslides",
    "s5",
];
const CSS: &str = "<style>\n:where(.amsthm-qed) { float: right; }\n</style>";

fn depth_of(unit: &str) -> Option<i64> {
    PARENT_COUNTERS
        .iter()
        .find(|(name, _)| *name == unit)
        .map(|&(_, d)| d)
}

fn warn(message: impl Display) {
    eprintln!("[amsthm] warning: {message}");
}

/// A note of plain text: words of letters and digits (anything not ASCII
/// but a space), a hyphen inside one, and one of `,;:?!` after one, single
/// spaces between. pandoc's markdown reader gives such a text as it is, a
/// Str a word.
#[cfg(not(target_family = "wasm"))]
fn is_plain(note: &str) -> bool {
    let letter = |c: char| {
        c.is_ascii_alphanumeric()
            || (!c.is_ascii()
                && !matches!(
                    c,
                    '\u{85}' | '\u{a0}' | '\u{1680}' | '\u{2000}'
                        ..='\u{200a}'
                            | '\u{2028}'
                            | '\u{2029}'
                            | '\u{202f}'
                            | '\u{205f}'
                            | '\u{3000}'
                            | '\u{feff}'
                ))
    };
    note.split(' ').all(|word| {
        let word = word.strip_suffix([',', ';', ':', '?', '!']).unwrap_or(word);
        word.split('-')
            .all(|part| !part.is_empty() && part.chars().all(letter))
    })
}

/// A div's note, as the original reads it with pandoc's markdown reader
/// where it is plain text; otherwise its text, with a warning.
/// A div's note, read by pandoc's markdown reader as the original reads it
/// (`pandoc.read`): the host's, built for wasm; natively (a JSON filter has
/// no pandoc to call) as that reader reads plain text, and any other note
/// as its text, with a warning.
fn note_inlines(info: &str) -> Result<Vec<Inline>, String> {
    #[cfg(target_family = "wasm")]
    {
        let doc = libpandoc::read(info, "markdown").map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for block in doc.blocks {
            match block {
                Block::Para(xs) | Block::Plain(xs) => out.extend(xs),
                // (the original puts the block among inlines, and pandoc fails)
                other => return Err(format!("the note {info:?} is not inlines: {}", other.tag())),
            }
        }
        Ok(out)
    }
    #[cfg(not(target_family = "wasm"))]
    {
        if !is_plain(info) {
            warn(format!(
                "the note {info:?} has markup, which needs pandoc's markdown reader: kept as text"
            ));
        }
        Ok(panir::inlines(info))
    }
}

/// Whether a note holds math, a citation, an image or raw TeX, where a `]`
/// would end LaTeX's optional argument: the original braces it then.
fn needs_brace(note: &mut [Inline], ctx: &mut Ctx) -> Option<bool> {
    struct Scan(bool);
    impl Filter for Scan {
        fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
            self.0 |= matches!(
                x,
                Inline::Math(_) | Inline::Cite(_) | Inline::Image(_) | Inline::RawInline(_)
            );
            None
        }
    }
    let mut scan = Scan(false);
    for x in note {
        // (a walk of x is of what is in it, as Lua's `e:walk`)
        scan.inline(x, ctx);
        ctx.apply(x, &mut scan)?;
    }
    Some(scan.0)
}

// ---------------------------------------------------------------------------
// Fonts: an element of one around each Str, and what that does to nesting

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Font {
    Bold,
    Italic,
    SmallCaps,
}

impl Font {
    fn named(name: &str) -> Option<Font> {
        match name {
            "bold" => Some(Font::Bold),
            "italic" => Some(Font::Italic),
            "smallcaps" => Some(Font::SmallCaps),
            _ => None,
        }
    }

    fn latex(self) -> &'static str {
        match self {
            Font::Bold => "\\bfseries",
            Font::Italic => "\\itshape",
            Font::SmallCaps => "\\scshape",
        }
    }

    fn wrap(self, content: Vec<Inline>) -> Inline {
        match self {
            Font::Bold => Inline::Strong(content),
            Font::Italic => Inline::Emph(content),
            Font::SmallCaps => Inline::SmallCaps(content),
        }
    }

    fn is(self, x: &Inline) -> bool {
        matches!(
            (self, x),
            (Font::Bold, Inline::Strong(_))
                | (Font::Italic, Inline::Emph(_))
                | (Font::SmallCaps, Inline::SmallCaps(_))
        )
    }

    /// The content of an inline that is this font's element.
    fn content_mut(self, x: &mut Inline) -> Option<&mut Vec<Inline>> {
        match (self, x) {
            (Font::Bold, Inline::Strong(xs))
            | (Font::Italic, Inline::Emph(xs))
            | (Font::SmallCaps, Inline::SmallCaps(xs)) => Some(xs),
            _ => None,
        }
    }

    /// In LaTeX, the font of the text around a reference applies to what
    /// `\ref` and an in-text citation print; `\eqref` is upright, which
    /// undoes a shape (italic, small caps) but not bold.
    fn applies_to(self, x: &Inline) -> bool {
        let shape = self != Font::Bold;
        match x {
            Inline::Cite(cite) => {
                let mode = match cite.citations.as_slice() {
                    [c] => Some(c.mode),
                    [] => None,
                    _ => Some(CitationMode::NormalCitation), // (`\eqref{a}, \eqref{b}`)
                };
                mode == Some(CitationMode::AuthorInText)
                    || (!shape && mode == Some(CitationMode::NormalCitation))
            }
            Inline::RawInline(raw) => {
                let command = |name: &str| {
                    raw.text
                        .strip_prefix(name)
                        .is_some_and(|rest| rest.starts_with('{') && rest.ends_with('}'))
                };
                raw.format.as_str() == "tex" && (command("\\ref") || (!shape && command("\\eqref")))
            }
            _ => false,
        }
    }
}

/// Joins elements of the font next to each other, or with one Space between.
fn merge_consecutive(font: Font, content: &mut Vec<Inline>) {
    let mut out: Vec<Inline> = vec![];
    let mut run: Option<Vec<Inline>> = None;
    let mut taken = std::mem::take(content).into_iter().peekable();
    while let Some(mut x) = taken.next() {
        if let Some(xs) = font.content_mut(&mut x) {
            run.get_or_insert_with(Vec::new).append(xs);
        } else if let (Some(r), Inline::Space) = (&mut run, &x)
            && taken.peek().is_some_and(|next| font.is(next))
        {
            r.push(Inline::Space);
        } else {
            if let Some(r) = run.take() {
                out.push(font.wrap(r));
            }
            out.push(x);
        }
    }
    if let Some(r) = run {
        out.push(font.wrap(r));
    }
    *content = out;
}

/// Each Str in the font; an element of the font in it undone for emphasis
/// (`\emph` toggles), flattened for the others (bold in bold is bold); the
/// references the font applies to in it; then neighbouring elements of the
/// font in a block joined.
struct Wrap(Font);

impl Filter for Wrap {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        let font = self.0;
        if matches!(x, Inline::Str(_)) || font.applies_to(x) {
            return Some(vec![font.wrap(vec![std::mem::replace(x, Inline::Space)])]);
        }
        let content = font.content_mut(x)?;
        if font == Font::Italic {
            let undone = std::mem::take(content).into_iter().flat_map(|mut child| {
                if let Some(xs) = font.content_mut(&mut child) {
                    std::mem::take(xs)
                } else if matches!(child, Inline::Space | Inline::SoftBreak | Inline::LineBreak) {
                    vec![child]
                } else {
                    vec![font.wrap(vec![child])]
                }
            });
            return Some(undone.collect());
        }
        let flat = std::mem::take(content).into_iter().flat_map(|mut child| {
            match font.content_mut(&mut child) {
                Some(xs) => std::mem::take(xs),
                None => vec![child],
            }
        });
        *content = flat.collect();
        None
    }

    fn block(&mut self, x: &mut Block, _: &mut Ctx) -> Option<Vec<Block>> {
        if let Some(content) = x.inlines_mut() {
            merge_consecutive(self.0, content);
        }
        None
    }
}

// ---------------------------------------------------------------------------
// The environments

#[derive(Clone)]
struct Style {
    headfont: Vec<Font>,
    bodyfont: Vec<Font>,
    headpunct: String,
    /// " ", "newline", or a length (LaTeX).
    headspace: String,
    /// LaTeX only, as below and indent.
    above: String,
    below: String,
    indent: String,
}

impl Style {
    fn builtin(headfont: &[Font], bodyfont: &[Font]) -> Self {
        Style {
            headfont: headfont.to_vec(),
            bodyfont: bodyfont.to_vec(),
            headpunct: ".".into(),
            headspace: " ".into(),
            above: String::new(),
            below: String::new(),
            indent: String::new(),
        }
    }
}

fn builtin_styles() -> BTreeMap<String, Style> {
    BTreeMap::from([
        (
            "plain".into(),
            Style::builtin(&[Font::Bold], &[Font::Italic]),
        ),
        ("definition".into(), Style::builtin(&[Font::Bold], &[])),
        ("remark".into(), Style::builtin(&[Font::Italic], &[])),
    ])
}

struct Theorem {
    style: String,
    env_name: String,
    text: String,
    parent_counter: Option<String>,
    shared_counter: Option<String>,
    numbered: bool,
    proof: bool,
}

impl Theorem {
    fn new(
        style: &str,
        name: &str,
        text: &str,
        parent_counter: Option<String>,
        shared_counter: Option<String>,
    ) -> Self {
        let (env_name, numbered) = match name.strip_suffix('*') {
            Some(stripped) => (stripped, false),
            None => (name, true),
        };
        let parent_counter = parent_counter.filter(|p| {
            let known = depth_of(p).is_some();
            if !known {
                warn(format!("unsupported parent_counter {p}, ignoring"));
            }
            known
        });
        let shared_counter = shared_counter.filter(|_| !(numbered && parent_counter.is_some()));
        Theorem {
            style: style.into(),
            env_name: env_name.into(),
            text: if text.is_empty() {
                env_name.into()
            } else {
                text.into()
            },
            parent_counter,
            shared_counter,
            numbered,
            proof: false,
        }
    }

    /// `name_to_text: {proof: ""}` is an empty name, which a theorem's text
    /// takes as none: the proof's is then its environment's, "proof".
    fn proof(text: Option<&str>) -> Self {
        Theorem {
            proof: true,
            ..Theorem::new("proof", "proof*", text.unwrap_or("Proof"), None, None)
        }
    }

    fn latex(&self) -> String {
        let (env, text) = (&self.env_name, &self.text);
        match (&self.shared_counter, &self.parent_counter) {
            _ if !self.numbered => format!("\\newtheorem*{{{env}}}{{{text}}}"),
            (Some(shared), _) => format!("\\newtheorem{{{env}}}[{shared}]{{{text}}}"),
            (None, Some(parent)) => format!("\\newtheorem{{{env}}}{{{text}}}[{parent}]"),
            (None, None) => format!("\\newtheorem{{{env}}}{{{text}}}"),
        }
    }

    fn class_name(&self) -> String {
        self.env_name.replace(' ', "_")
    }

    fn counter_name(&self) -> &str {
        self.shared_counter.as_deref().unwrap_or(&self.env_name)
    }
}

/// The heading levels in a counter's number, `first` to `depth`; a heading
/// at one of them restarts it.
#[derive(Clone, Copy)]
struct Counter {
    first: i64,
    depth: f64,
}

impl Default for Counter {
    fn default() -> Self {
        Counter {
            first: 1,
            depth: 0.0,
        }
    }
}

/// The writer's options the original reads (`PANDOC_WRITER_OPTIONS`): the
/// conversion's options (defaults-file keys) where a host running the filter
/// in process knows them, else its `writer_options`, which libpandoc gives
/// every filter it runs (Lua's values under the same keys, but
/// `top-level-chapter` for `chapter`); plain pandoc gives a JSON filter
/// neither.
#[derive(Default)]
struct Writer {
    number_sections: bool,
    top_level_division: String,
    number_offset: Vec<i64>,
    /// A variable (-V).
    secnumdepth: Option<String>,
}

impl Writer {
    fn of(conversion: &Conversion) -> Self {
        let Some(o) = conversion
            .options
            .as_ref()
            .or(conversion.writer_options.as_ref())
        else {
            return Writer::default();
        };
        let division = o["top-level-division"].as_str().unwrap_or("default");
        Writer {
            number_sections: o["number-sections"].as_bool().unwrap_or(false),
            top_level_division: division
                .strip_prefix("top-level-")
                .unwrap_or(division)
                .into(),
            number_offset: o["number-offset"]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|n| n.as_i64())
                .collect(),
            secnumdepth: Some(&o["variables"]["secnumdepth"])
                .filter(|v| !v.is_null())
                .map(|v| v.as_str().map_or_else(|| v.to_string(), String::from)),
        }
    }
}

struct Options {
    css: bool,
    styles: BTreeMap<String, Style>,
    user_styles: Vec<String>,
    swapnumbers: bool,
    qed_symbol: String,
    counters: BTreeMap<String, Counter>,
    numbered_depth: f64,
    part_level: Option<i64>,
    proof_name: Option<String>,
    /// In order.
    theorems: Vec<Theorem>,
    /// Each class's theorem, an index into `theorems`.
    by_class: HashMap<String, usize>,
    counter_depth: f64,
    ignored_headings: BTreeSet<String>,
    header_counters: BTreeMap<i64, i64>,
    theorem_counters: HashMap<String, i64>,
    /// Each identifier's number ("" in LaTeX).
    identifiers: HashMap<String, String>,
    names: HashMap<String, String>,
    unnumbered: BTreeSet<String>,
    warned: BTreeSet<String>,
}

/// A map's entries, as Lua's `pairs`: a list's are its indices, which no
/// name matches; text or a boolean fails, as in Lua.
fn lua_pairs<'a>(
    value: &'a MetaValue,
    what: &str,
) -> Result<Vec<(&'a str, &'a MetaValue)>, String> {
    match value {
        MetaValue::MetaMap(map) => Ok(map.iter().map(|(k, v)| (k.as_str(), v)).collect()),
        MetaValue::MetaList(_) | MetaValue::MetaInlines(_) | MetaValue::MetaBlocks(_) => Ok(vec![]),
        other => Err(format!("amsthm: {what} is {}, not a map", other.tag())),
    }
}

fn read_font(value: Option<&MetaValue>, place: &str) -> Vec<Font> {
    let mut font = vec![];
    for v in items(value) {
        let name = stringify(v);
        match Font::named(&name) {
            Some(f) => font.push(f),
            None if name == "normal" => {}
            None => warn(format!(
                "unknown font {name} in {place}; use bold, italic, smallcaps or normal"
            )),
        }
    }
    font
}

/// amsthm's styles, and those under `styles` (`\newtheoremstyle`'s
/// arguments; what is left out as for a theorem), with the names of the
/// user's, sorted.
fn read_styles(value: Option<&MetaValue>) -> (BTreeMap<String, Style>, Vec<String>) {
    let mut styles = builtin_styles();
    let Some(MetaValue::MetaMap(map)) = value else {
        return (styles, vec![]);
    };
    let mut kept = vec![];
    for (name, spec) in map {
        let name = name.as_str();
        if OPTION_KEYS.contains(&name) {
            warn(format!(
                "a style cannot be called {name}, which is an option of its own; ignoring it"
            ));
        } else if name == "proof" {
            warn("a style cannot be called proof, which lists the proof environments; ignoring it");
        } else if let MetaValue::MetaMap(settings) = spec {
            let get = |key: &str, default: &str| {
                settings.get(key).map_or_else(|| default.into(), stringify)
            };
            let place = format!("style {name}");
            let style = Style {
                headfont: settings
                    .get("headfont")
                    .map_or_else(|| vec![Font::Bold], |v| read_font(Some(v), &place)),
                bodyfont: read_font(settings.get("bodyfont"), &place),
                headpunct: get("headpunct", "."),
                headspace: get("headspace", " "),
                above: get("above", ""),
                below: get("below", ""),
                indent: get("indent", ""),
            };
            styles.insert(name.into(), style);
            kept.push(name.into());
        } else {
            warn(format!(
                "style {name} is not a map of its settings; ignoring it"
            ));
        }
    }
    (styles, kept)
}

/// The LaTeX unit that level-1 headings are, as pandoc decides it.
fn top_level_division(meta: &Meta, writer: &Writer) -> String {
    if depth_of(&writer.top_level_division).is_some() {
        return writer.top_level_division.clone();
    }
    let class = panir::given(meta.get("documentclass")).map(stringify);
    if class.is_some_and(|c| CHAPTER_CLASSES.contains(&c.as_str())) {
        "chapter"
    } else {
        "section"
    }
    .into()
}

/// How many heading levels step their LaTeX counters: none without -N, else
/// as deep as secnumdepth.
fn numbered_depth(meta: &Meta, writer: &Writer, top: &str) -> f64 {
    if !writer.number_sections {
        return 0.0;
    }
    let text = match meta.get("secnumdepth") {
        // (false: as if unset, but not the variable)
        Some(v) => panir::given(v).map(stringify),
        None => writer.secnumdepth.clone(),
    };
    let secnumdepth = text.and_then(|t| lua_number(&t)).unwrap_or(5.0);
    secnumdepth - depth_of(top).unwrap_or(0) as f64 + 2.0
}

/// The text of the math, raw TeX and words in a value.
#[derive(Default)]
struct Texts(Vec<String>);

impl Filter for Texts {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        match x {
            Inline::Math(m) => self.0.push(m.text.to_string()),
            Inline::RawInline(r) => self.0.push(r.text.to_string()),
            Inline::Str(s) => self.0.push(s.to_string()),
            _ => {}
        }
        None
    }
}

fn read_options(meta: &Meta, writer: &Writer) -> Result<Options, String> {
    let empty = Meta::new();
    // (an empty `amsthm:` is text, not a map: it defines nothing)
    let opt = match meta.get("amsthm") {
        Some(MetaValue::MetaMap(map)) => map,
        _ => &empty,
    };

    let mut name_to_text = HashMap::new();
    if let Some(v) = opt.get("name_to_text") {
        for (k, v) in lua_pairs(v, "name_to_text")? {
            name_to_text.insert(k.to_string(), stringify(v));
        }
    }
    // one unit for every environment, or a map of them by name
    let (parent_counters, parent_counter): (HashMap<String, String>, Option<String>) =
        match opt.get("parent_counter") {
            Some(MetaValue::MetaMap(map)) => (
                map.iter()
                    .map(|(k, v)| (k.to_string(), stringify(v)))
                    .collect(),
                None,
            ),
            Some(other) => (HashMap::new(), Some(stringify(other))),
            None => (HashMap::new(), None),
        };
    let parent_of = |name: &str| {
        parent_counters
            .get(name)
            .cloned()
            .or_else(|| parent_counter.clone())
    };
    let text_of = |name: &str| name_to_text.get(name).cloned().unwrap_or_default();

    let mut theorems = vec![];
    let (styles, user_styles) = read_styles(opt.get("styles"));
    let order = ["plain", "definition", "remark"].into_iter().chain(
        user_styles
            .iter()
            .map(String::as_str)
            .filter(|s| !["plain", "definition", "remark"].contains(s)),
    );
    for style in order {
        for entry in items(opt.get(style)) {
            if let MetaValue::MetaMap(map) = entry {
                // a name, and the names sharing its counter (YAML's keys come
                // sorted, and so are a BTreeMap's)
                for (key, value) in map {
                    theorems.push(Theorem::new(
                        style,
                        key,
                        &text_of(key),
                        parent_of(key),
                        None,
                    ));
                    let shared = match value {
                        MetaValue::MetaList(xs) => xs.iter().map(stringify).collect(),
                        other => vec![stringify(other)],
                    };
                    for name in shared {
                        theorems.push(Theorem::new(
                            style,
                            &name,
                            &text_of(&name),
                            None,
                            Some(key.to_string()),
                        ));
                    }
                }
            } else {
                let name = stringify(entry);
                theorems.push(Theorem::new(
                    style,
                    &name,
                    &text_of(&name),
                    parent_of(&name),
                    None,
                ));
            }
        }
    }
    theorems.push(Theorem::proof(
        name_to_text.get("proof").map(String::as_str),
    ));
    let by_class: HashMap<String, usize> = theorems
        .iter()
        .enumerate()
        .map(|(i, t)| (t.class_name(), i))
        .collect();

    let top = top_level_division(meta, writer);
    // with parts at level 1: a part steps no other counter, and is in a
    // theorem's number only when theorems are numbered within parts
    let part_level = (top == "part").then_some(1);
    let explicit = panir::given(opt.get("counter_depth")).and_then(|v| lua_number(&stringify(v)));
    let mut counters = BTreeMap::new();
    let mut counter_depth = explicit.unwrap_or(0.0);
    for t in theorems
        .iter()
        .filter(|t| t.numbered && t.shared_counter.is_none())
    {
        let mut c = Counter {
            first: 1,
            depth: explicit.unwrap_or(0.0),
        };
        if let (None, Some(parent)) = (explicit, &t.parent_counter) {
            // within the heading level that LaTeX would number within
            c.depth =
                (depth_of(parent).unwrap_or(0) - depth_of(&top).unwrap_or(0) + 1).max(0) as f64;
            if part_level.is_some() && parent != "part" {
                c.first = 2;
            }
        }
        counters.insert(t.counter_name().to_string(), c);
        counter_depth = counter_depth.max(c.depth);
    }
    for name in parent_counters.keys() {
        if let Some(shared) = by_class
            .get(&name.replace(' ', "_"))
            .and_then(|&i| theorems[i].shared_counter.as_ref())
        {
            warn(format!(
                "{name} shares the counter of {shared}, so its parent_counter is {shared}'s; ignoring the one given"
            ));
        }
    }

    let ignored_headings = panir::given(opt.get("counter_ignore_headings"))
        .map_or_else(BTreeSet::new, |v| items(v).iter().map(stringify).collect());
    // where pandoc starts numbering sections (--number-offset)
    let header_counters = (1..=counter_depth.floor() as i64)
        .map(|i| {
            (
                i,
                writer
                    .number_offset
                    .get(i as usize - 1)
                    .copied()
                    .unwrap_or(0),
            )
        })
        .collect();

    // the end-of-proof symbol as TeX math: the text of its math, raw TeX and words
    let mut qed_symbol = "\\Box".to_string();
    if let Some(v) = opt.get("qed_symbol") {
        let mut texts = Texts::default();
        panir::apply(&mut inlines_of(v)?, &mut texts, None).map_err(|e| e.to_string())?;
        if !texts.0.is_empty() {
            qed_symbol = texts.0.join(" ");
        }
    }

    Ok(Options {
        css: opt.get("css").is_none_or(|v| stringify(v) != "false"),
        styles,
        user_styles,
        swapnumbers: opt
            .get("swapnumbers")
            .is_some_and(|v| stringify(v) == "true"),
        qed_symbol,
        counters,
        numbered_depth: numbered_depth(meta, writer, &top),
        part_level,
        proof_name: name_to_text.get("proof").cloned(),
        theorems,
        by_class,
        counter_depth,
        ignored_headings,
        header_counters,
        theorem_counters: HashMap::new(),
        identifiers: HashMap::new(),
        names: HashMap::new(),
        unnumbered: BTreeSet::new(),
        warned: BTreeSet::new(),
    })
}

impl Options {
    fn latex(&self) -> String {
        let mut lines = vec![];
        if self.qed_symbol != "\\Box" {
            lines.push(format!(
                "\\renewcommand{{\\qedsymbol}}{{\\ensuremath{{{}}}}}",
                self.qed_symbol
            ));
        }
        // \newtheoremstyle{name}{above}{below}{body font}{indent}{heading font}
        //   {punctuation}{space after the heading}{heading layout}
        // (\normalfont for none: an empty argument is the font around it)
        let fonts = |font: &[Font]| match font.iter().map(|f| f.latex()).collect::<String>() {
            s if s.is_empty() => "\\normalfont".to_string(),
            s => s,
        };
        for name in &self.user_styles {
            let s = &self.styles[name];
            let space = if s.headspace == "newline" {
                "\\newline"
            } else {
                &s.headspace
            };
            lines.push(format!(
                "\\newtheoremstyle{{{name}}}{{{}}}{{{}}}{{{}}}{{{}}}{{{}}}{{{}}}{{{space}}}{{}}",
                s.above,
                s.below,
                fonts(&s.bodyfont),
                s.indent,
                fonts(&s.headfont),
                s.headpunct,
            ));
        }
        if self.swapnumbers {
            lines.push("\\swapnumbers".into()); // before \newtheorem, which takes the order from it
        }
        if let Some(name) = &self.proof_name {
            // at the start of the document, after babel names the proof
            lines.push(format!(
                "\\AtBeginDocument{{\\renewcommand{{\\proofname}}{{{name}}}}}"
            ));
        }
        let mut style = "";
        for t in self.theorems.iter().filter(|t| !t.proof) {
            if t.style != style {
                style = &t.style;
                lines.push(format!("\\theoremstyle{{{style}}}"));
            }
            lines.push(t.latex());
        }
        lines.join("\n")
    }

    /// The environment a div is, by its classes; none for one naming two.
    fn find(&self, classes: &[panir::Text], warn_two: bool) -> Option<usize> {
        let mut found = None;
        let mut others: Vec<&str> = vec![];
        for c in classes {
            match (self.by_class.get(c.as_str()), found) {
                (Some(&t), None) => found = Some(t),
                (Some(&t), Some(f)) if t != f && !others.contains(&c.as_str()) => others.push(c),
                _ => {}
            }
        }
        if !others.is_empty() {
            if let (true, Some(f)) = (warn_two, found) {
                warn(format!(
                    "multiple environments found: {}, {}; skipping",
                    self.theorems[f].class_name(),
                    others.join(", ")
                ));
            }
            return None;
        }
        found
    }

    /// The heading, as amsthm's `\thmhead` writes it: the name and the
    /// number in the style's heading font, the number upright, the note in
    /// parentheses in the note font, and the punctuation; then the space
    /// after the heading.
    fn header(
        &mut self,
        t: usize,
        identifier: &str,
        info: Option<&str>,
        ctx: &mut Ctx,
    ) -> Option<Vec<Inline>> {
        let theorem = &self.theorems[t];
        if theorem.proof {
            // "Proof.", in italics; or the note, in italics but its
            // references, for the whole heading
            let Some(info) = info.filter(|i| !i.is_empty()) else {
                return Some(vec![
                    Inline::Emph(panir::inlines(&format!("{}.", theorem.text))),
                    Inline::Space,
                ]);
            };
            // each Str in italics, emphasis in it upright, references as the
            // font applies to them; then neighbouring italics joined
            let mut note = ctx.ok(note_inlines(info))?;
            ctx.apply(&mut note, &mut Wrap(Font::Italic))?;
            merge_consecutive(Font::Italic, &mut note);
            note.extend([Inline::Emph(vec![".".into()]), Inline::Space]);
            return Some(note);
        }
        let style = &self.styles[&theorem.style];
        let head = style.headfont.clone();
        let mut number = None;
        if theorem.numbered {
            let name = theorem.counter_name().to_string();
            let count = self.theorem_counters.entry(name.clone()).or_insert(0);
            *count += 1;
            let count = *count;
            let counter = self.counters.get(&name).copied().unwrap_or_default();
            let mut parts: Vec<String> = (counter.first..=counter.depth.floor() as i64)
                .map(|level| {
                    let n = self.header_counters.get(&level).copied().unwrap_or(0);
                    // \thepart is a Roman numeral, and empty before the first part
                    match (Some(level) == self.part_level, n > 0) {
                        (true, true) => to_roman_numeral(n),
                        (true, false) => String::new(),
                        (false, _) => n.to_string(),
                    }
                })
                .collect();
            parts.push(count.to_string());
            let joined = parts.join(".");
            if !identifier.is_empty() {
                self.identifiers.insert(identifier.into(), joined.clone());
                self.names.insert(identifier.into(), theorem.text.clone());
            }
            number = Some(joined);
        }

        let mut runs: Vec<(Vec<Inline>, Vec<Font>)> =
            vec![(panir::inlines(&theorem.text), head.clone())];
        match number {
            // \swappedhead: the number first, in the heading font, then a tie
            Some(n) if self.swapnumbers => {
                runs.insert(0, (vec![n.into(), "\u{a0}".into()], head.clone()))
            }
            Some(n) => runs.push((
                vec![Inline::Space, n.into()],
                head.iter().copied().filter(|&f| f == Font::Bold).collect(),
            )),
            None => {}
        }
        if let Some(info) = info.filter(|i| !i.is_empty()) {
            let mut note = vec![Inline::Space, "(".into()];
            note.extend(ctx.ok(note_inlines(info))?);
            note.push(")".into());
            runs.push((note, vec![]));
        }
        runs.push((vec![style.headpunct.as_str().into()], head));
        let mut out = join_runs(runs);
        out.push(if style.headspace == "newline" {
            Inline::LineBreak
        } else {
            Inline::Space
        });
        Some(out)
    }
}

/// The inlines in the elements of a font.
fn in_font(content: Vec<Inline>, font: &[Font]) -> Vec<Inline> {
    font.iter()
        .rev()
        .fold(content, |inner, f| vec![f.wrap(inner)])
}

/// Runs of inlines, each with its font, each stretch of one font in its
/// elements once ("Theorem 1." one Strong, not three).
fn join_runs(runs: Vec<(Vec<Inline>, Vec<Font>)>) -> Vec<Inline> {
    let mut out = vec![];
    let mut current: Option<(Vec<Inline>, Vec<Font>)> = None;
    for (run, font) in runs {
        match &mut current {
            Some((content, f)) if *f == font => content.extend(run),
            _ => {
                if let Some((content, f)) = current.take() {
                    out.extend(in_font(content, &f));
                }
                current = Some((run, font));
            }
        }
    }
    if let Some((content, f)) = current {
        out.extend(in_font(content, &f));
    }
    out
}

/// `name`, a no-break space, then `number`: "Theorem 1".
fn named(name: &str, number: Inline) -> Vec<Inline> {
    let mut out = panir::inlines(name);
    out.extend(["\u{a0}".into(), number]);
    out
}

fn add_header_include(meta: &mut Meta, block: Block, at_start: bool) {
    let existing = panir::into_items(meta.remove("header-includes"));
    let new = MetaValue::from(block);
    let values: Vec<MetaValue> = if at_start {
        std::iter::once(new).chain(existing).collect()
    } else {
        existing.into_iter().chain(std::iter::once(new)).collect()
    };
    meta.insert("header-includes".into(), MetaValue::MetaList(values));
}

// ---------------------------------------------------------------------------
// The body of an environment

/// Environments in a body, held out of it (`Hold`) while the body is set
/// in its font, then put back (`Restore`).
struct Hold<'a> {
    options: &'a Options,
    held: Vec<Block>,
}

impl Filter for Hold<'_> {
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let Block::Div(d) = x else { return None };
        self.options.find(&d.attr.classes, false)?;
        let placeholder = Div {
            attr: Attr::new(
                "",
                &[],
                &[("amsthm-held", &(self.held.len() + 1).to_string())],
            ),
            content: vec![],
        };
        self.held.push(std::mem::replace(x, placeholder.into()));
        ctx.skip_children();
        None
    }
}

struct Restore(Vec<Block>);

impl Filter for Restore {
    fn block(&mut self, x: &mut Block, _: &mut Ctx) -> Option<Vec<Block>> {
        let Block::Div(d) = x else { return None };
        let i = lua_number(d.attr.get("amsthm-held")?)?;
        // (any div with the attribute, the user's too: a copy each, as each
        // return is in Lua)
        let held = self
            .0
            .get((i.fract() == 0.0 && i >= 1.0).then(|| i as usize - 1)?)?;
        Some(vec![held.clone()])
    }
}

/// amsthm's \mathqed, \quad\qedsymbol (a space between where the symbol
/// starts with a letter).
fn qed_math(options: &Options) -> String {
    let space = if options
        .qed_symbol
        .starts_with(|c: char| c.is_ascii_alphabetic())
    {
        " "
    } else {
        ""
    };
    format!("\\quad{space}{}", options.qed_symbol)
}

fn qed_span(options: &Options) -> Inline {
    Span {
        attr: Attr::new("", &["amsthm-qed"], &[]),
        content: vec![
            Math {
                math_type: MathType::InlineMath,
                text: qed_math(options).into(),
            }
            .into(),
        ],
    }
    .into()
}

/// amsthm's \qedhere: the symbol where it is, \mathqed in math, \qed (after
/// the spaces before it) in text; environments in this one keep theirs.
struct Qedhere<'a> {
    options: &'a Options,
    placed: bool,
}

impl Filter for Qedhere<'_> {
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        if let Block::Div(d) = x
            && self.options.find(&d.attr.classes, false).is_some()
        {
            ctx.skip_children();
        }
        None
    }

    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Math(m) = x else { return None };
        // `\qedhere` not followed by a letter
        let mut out = String::new();
        let mut rest: &str = &m.text;
        while let Some(i) = rest.find("\\qedhere") {
            let after = &rest[i + "\\qedhere".len()..];
            out += &rest[..i];
            if after.starts_with(|c: char| c.is_ascii_alphabetic()) {
                out += "\\qedhere";
            } else {
                out += &qed_math(self.options);
                self.placed = true;
            }
            rest = after;
        }
        out += rest;
        m.text = out.into();
        None
    }

    fn inlines(&mut self, xs: &mut Vec<Inline>, _: &mut Ctx) -> Option<()> {
        let is_qedhere = |x: &Inline| {
            matches!(x, Inline::RawInline(r) if matches!(r.format.as_str(), "tex" | "latex")
                && r.text.trim_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r']) == "\\qedhere")
        };
        if !xs.iter().any(is_qedhere) {
            return Some(());
        }
        let mut out: Vec<Inline> = vec![];
        for x in std::mem::take(xs) {
            if is_qedhere(&x) {
                // (\qed starts with \unskip)
                while matches!(
                    out.last(),
                    Some(Inline::Space | Inline::SoftBreak | Inline::LineBreak)
                ) {
                    out.pop();
                }
                out.push(qed_span(self.options));
                self.placed = true;
            } else {
                out.push(x);
            }
        }
        *xs = out;
        Some(())
    }
}

/// Not LaTeX: the heading, the body's font, the end of a proof.
fn environment(div: &mut Div, options: &mut Options, ctx: &mut Ctx) -> Option<()> {
    let t = options.find(&div.attr.classes, true)?;
    let identifier = div.attr.identifier.to_string();
    if !options.theorems[t].numbered && !identifier.is_empty() {
        options.unnumbered.insert(identifier.clone());
    }
    let info = div.attr.get("info").map(str::to_string);
    let mut title = options.header(t, &identifier, info.as_deref(), ctx)?;
    let space = title.pop()?;
    let header = vec![
        Span {
            attr: Attr::new("", &["amsthm-title"], &[]),
            content: title,
        }
        .into(),
        space,
    ];

    let style = options.theorems[t].style.clone();
    let bodyfont = options
        .styles
        .get(&style)
        .map(|s| s.bodyfont.clone())
        .unwrap_or_default();
    if !bodyfont.is_empty() {
        let mut hold = Hold {
            options,
            held: vec![],
        };
        ctx.apply(&mut div.content, &mut hold)?;
        let held = hold.held;
        for font in bodyfont {
            ctx.apply(&mut div.content, &mut Wrap(font))?;
        }
        if !held.is_empty() {
            ctx.apply(&mut div.content, &mut Restore(held))?;
        }
    }
    match div.content.first_mut().and_then(Block::inlines_mut) {
        Some(first) => {
            first.splice(0..0, header);
        }
        None => div.content.insert(0, Block::Para(header)),
    }

    if options.theorems[t].proof {
        let mut qedhere = Qedhere {
            options,
            placed: false,
        };
        ctx.apply(&mut div.content, &mut qedhere)?;
        if !qedhere.placed {
            // amsthm's \qed, \nobreak\hfill\quad\openbox: as math, \quad\Box
            // (the \hfill is the CSS's)
            match div.content.last_mut().and_then(Block::inlines_mut) {
                Some(last) => last.push(qed_span(options)),
                None => div.content.push(Block::Para(vec![qed_span(options)])),
            }
        }
    }
    div.attr.classes.push("amsthm".into());
    let class = format!("amsthm-{style}");
    if !div.attr.classes.iter().any(|c| c.as_str() == class) {
        div.attr.classes.push(class.into());
    }
    Some(())
}

// ---------------------------------------------------------------------------
// References

/// The environment a citation's id names, and whether to name it: `@Euler`
/// is `@euler`, the environment's name before the number.
fn reference_target(id: &str, known: impl Fn(&str) -> bool) -> Option<(String, bool)> {
    if known(id) {
        return Some((id.into(), false));
    }
    let mut chars = id.chars();
    let first = chars.next().filter(char::is_ascii_uppercase)?;
    let lower = format!("{}{}", first.to_ascii_lowercase(), chars.as_str());
    known(&lower).then_some((lower, true))
}

/// The ids of `[@a; @b]`, each an environment's, none with a prefix or
/// suffix (which the list would drop); else None, for citeproc.
fn multi_ref(cite: &Cite, known: &HashMap<String, String>) -> Option<Vec<String>> {
    if cite.citations.len() < 2 {
        return None;
    }
    cite.citations
        .iter()
        .map(|c| {
            let ok = c.mode == CitationMode::NormalCitation
                && known.contains_key(c.id.as_str())
                && c.prefix.is_empty()
                && c.suffix.is_empty();
            ok.then(|| c.id.to_string())
        })
        .collect()
}

fn ref_list(ids: &[String], one: impl Fn(&str) -> Vec<Inline>) -> Vec<Inline> {
    let mut out = vec![];
    for (i, id) in ids.iter().enumerate() {
        if i > 0 {
            out.extend([",".into(), Inline::Space]);
        }
        out.extend(one(id));
    }
    out
}

fn single(cite: &Cite) -> Option<(&str, CitationMode)> {
    match cite.citations.as_slice() {
        [c] => Some((c.id.as_str(), c.mode)),
        _ => None,
    }
}

/// `\ref{id}` or `\eqref{id}` (any command, its argument up to the last
/// brace).
fn raw_ref(text: &str) -> Option<(&str, &str)> {
    let rest = text.strip_prefix('\\')?;
    let name_end = rest.find(|c: char| !c.is_ascii_alphabetic())?;
    let (name, arg) = rest.split_at(name_end);
    let id = arg.strip_prefix('{')?.strip_suffix('}')?;
    (!name.is_empty()).then_some((name, id))
}

/// Once per id, a reference to an unnumbered environment; and a citation of
/// several that is left to citeproc.
fn warn_unnumbered(x: &Inline, options: &mut Options) {
    let ids: Vec<String> = match x {
        Inline::Cite(cite) => cite.citations.iter().map(|c| c.id.to_string()).collect(),
        Inline::RawInline(r) if r.format.as_str() == "tex" => match raw_ref(&r.text) {
            Some(("ref" | "eqref", id)) => vec![id.into()],
            _ => vec![],
        },
        _ => vec![],
    };
    for id in ids {
        if let Some((target, _)) = reference_target(&id, |i| options.unnumbered.contains(i))
            && options.warned.insert(target.clone())
        {
            warn(format!(
                "reference to the unnumbered environment {target}; LaTeX prints the last number before it, other output leaves it unresolved"
            ));
        }
    }
    let Inline::Cite(cite) = x else { return };
    if cite.citations.len() < 2 {
        return;
    }
    let (mut ours, mut others, mut named_, mut affixed) = (false, false, false, false);
    for c in &cite.citations {
        affixed |= !c.prefix.is_empty() || !c.suffix.is_empty();
        if options.identifiers.contains_key(c.id.as_str())
            || options.unnumbered.contains(c.id.as_str())
        {
            ours = true;
        } else if reference_target(&c.id, |i| options.identifiers.contains_key(i)).is_some() {
            named_ = true;
        } else {
            others = true;
        }
    }
    let text = stringify(x);
    if named_ {
        warn(format!(
            "a reference to several environments cannot name them, and is left to citeproc: {text}"
        ));
    } else if ours && others {
        warn(format!(
            "a citation mixes environments with other keys, and is left to citeproc: {text}"
        ));
    } else if ours && affixed {
        warn(format!(
            "a reference to several environments cannot carry a prefix or suffix, and is left to citeproc: {text}"
        ));
    }
}

/// The number, a link to the environment, perhaps led by its name, in
/// parentheses with `paren`.
fn ref_link(id: &str, number: &str, paren: bool, name: Option<&str>) -> Vec<Inline> {
    let link: Inline = Link {
        attr: Attr::default(),
        content: vec![number.into()],
        target: Target::new(&format!("#{id}"), ""),
    }
    .into();
    let out = match name {
        Some(name) => named(name, link),
        None => vec![link],
    };
    parenthesised(out, paren)
}

fn parenthesised(mut out: Vec<Inline>, paren: bool) -> Vec<Inline> {
    if paren {
        out.insert(0, "(".into());
        out.push(")".into());
    }
    out
}

/// Not LaTeX: `[@id]`, `@id`, `\ref{id}`, `\eqref{id}` to numbers.
fn resolve(x: &Inline, options: &Options) -> Option<Vec<Inline>> {
    let known = &options.identifiers;
    match x {
        Inline::Cite(cite) => {
            if let Some(ids) = multi_ref(cite, known) {
                return Some(ref_list(&ids, |i| ref_link(i, &known[i], true, None)));
            }
            let (id, mode) = single(cite)?;
            let (id, is_named) = reference_target(id, |i| known.contains_key(i))?;
            let name = if is_named {
                options.names.get(&id).map(String::as_str)
            } else {
                None
            };
            match mode {
                CitationMode::NormalCitation => Some(ref_link(&id, &known[&id], true, name)),
                CitationMode::AuthorInText => Some(ref_link(&id, &known[&id], false, name)),
                CitationMode::SuppressAuthor => None,
            }
        }
        Inline::RawInline(r) if r.format.as_str() == "tex" => match raw_ref(&r.text) {
            Some((kind @ ("ref" | "eqref"), id)) if known.contains_key(id) => {
                Some(ref_link(id, &known[id], kind == "eqref", None))
            }
            _ => None,
        },
        _ => None,
    }
}

/// LaTeX: a citation of an environment to `\ref` or `\eqref`, perhaps led by
/// its name (`Theorem~\ref{euler}`).
fn cite_to_ref(cite: &Cite, options: &Options) -> Option<Vec<Inline>> {
    let known = &options.identifiers;
    if let Some(ids) = multi_ref(cite, known) {
        return Some(ref_list(&ids, |i| {
            vec![Inline::raw("latex", format!("\\eqref{{{i}}}"))]
        }));
    }
    let (id, mode) = single(cite)?;
    if !matches!(
        mode,
        CitationMode::NormalCitation | CitationMode::AuthorInText
    ) {
        return None;
    }
    let paren = mode == CitationMode::NormalCitation;
    let (id, is_named) = reference_target(id, |i| known.contains_key(i))?;
    match options.names.get(&id).filter(|_| is_named) {
        None => Some(vec![Inline::raw(
            "latex",
            format!("\\{}{{{id}}}", if paren { "eqref" } else { "ref" }),
        )]),
        Some(name) => Some(parenthesised(
            named(name, Inline::raw("latex", format!("\\ref{{{id}}}"))),
            paren,
        )),
    }
}

/// LaTeX: the div's own blocks between `\begin{...}` and `\end{...}`, sharing
/// a line with the first and last paragraphs, in a bare div.
fn latex_environment(div: &mut Div, options: &Options, ctx: &mut Ctx) -> Option<Vec<Block>> {
    let t = options.find(&div.attr.classes, false)?;
    let env = &options.theorems[t].env_name;
    let mut opening = vec![Inline::raw("latex", format!("\\begin{{{env}}}"))];
    if let Some(info) = div.attr.get("info").filter(|i| !i.is_empty()) {
        let mut note = ctx.ok(note_inlines(info))?;
        let brace = needs_brace(&mut note, ctx)?;
        opening.push(Inline::raw("latex", if brace { "[{" } else { "[" }));
        opening.extend(note);
        opening.push(Inline::raw("latex", if brace { "}]" } else { "]" }));
    }
    if !div.attr.identifier.is_empty() {
        opening.push(Inline::raw(
            "latex",
            format!("\\label{{{}}}", div.attr.identifier),
        ));
    }
    let closing = Inline::raw("latex", format!("\\end{{{env}}}"));
    let mut content = std::mem::take(&mut div.content);
    match content.first_mut() {
        Some(Block::Para(first) | Block::Plain(first)) => {
            opening.push(Inline::raw("latex", "\n"));
            first.splice(0..0, opening);
        }
        _ => content.insert(0, Block::Plain(opening)),
    }
    match content.last_mut() {
        Some(Block::Para(last) | Block::Plain(last)) => {
            last.extend([Inline::raw("latex", "\n"), closing])
        }
        _ => content.push(Block::Plain(vec![closing])),
    }
    Some(vec![
        Div {
            attr: Attr::default(),
            content,
        }
        .into(),
    ])
}

// ---------------------------------------------------------------------------
// The two passes

/// The options, the header-includes, the headings' numbers, and (not LaTeX)
/// the environments; in LaTeX, the identifiers, for the references before
/// their environments.
#[derive(Default)]
struct First {
    options: Option<Options>,
    latex: bool,
}

impl Filter for First {
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) -> Option<()> {
        let format = ctx.format().unwrap_or_default();
        self.latex = LATEX_LIKE.contains(&format);
        let options = ctx.ok(read_options(&doc.meta, &Writer::of(ctx.conversion())))?;
        if options
            .counters
            .values()
            .any(|c| c.depth >= c.first as f64 && options.numbered_depth < c.first as f64)
        {
            warn(
                "theorems are numbered within sections that are not numbered, so their numbers start with 0 as in LaTeX; pass -N (--number-sections)",
            );
        }
        if self.latex {
            // amsthm first, for the user's own \newtheoremstyle; the
            // environments last, to use those styles
            add_header_include(
                &mut doc.meta,
                Block::raw("latex", "\\usepackage{amsthm}"),
                true,
            );
            add_header_include(&mut doc.meta, Block::raw("latex", options.latex()), false);
        } else if HTML_LIKE.contains(&format) && options.css {
            add_header_include(&mut doc.meta, Block::raw("html", CSS), false);
        }
        self.options = Some(options);
        Some(())
    }

    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let options = self.options.as_mut()?;
        match x {
            Block::Header(h) if !self.latex => count(h, options),
            Block::Div(d) if self.latex => {
                let t = options.find(&d.attr.classes, true)?;
                if !d.attr.identifier.is_empty() {
                    let (id, theorem) = (d.attr.identifier.to_string(), &options.theorems[t]);
                    options.names.insert(id.clone(), theorem.text.clone());
                    if !theorem.numbered {
                        options.unnumbered.insert(id.clone());
                    }
                    options.identifiers.insert(id, String::new());
                }
            }
            Block::Div(d) => environment(d, options, ctx)?,
            _ => {}
        }
        None
    }
}

/// \section* and the like step no counter, nor do headings LaTeX doesn't
/// number.
fn count(h: &panir::Header, o: &mut Options) {
    let level = h.level;
    if level as f64 > o.counter_depth
        || level as f64 > o.numbered_depth
        || h.attr.has_class("unnumbered")
    {
        return;
    }
    if o.ignored_headings.contains(&stringify(&h.content)) {
        return;
    }
    *o.header_counters.entry(level).or_insert(0) += 1;
    if Some(level) != o.part_level {
        for deeper in level + 1..=o.counter_depth.floor() as i64 {
            o.header_counters.insert(deeper, 0);
        }
    }
    // restart each counter numbered within this level or above it
    for (name, c) in &o.counters {
        if c.first <= level && level as f64 <= c.depth {
            o.theorem_counters.remove(name);
        }
    }
}

/// In LaTeX, the environments; the references, everywhere.
struct Second {
    options: Options,
    latex: bool,
}

impl Filter for Second {
    fn order(&self) -> Order {
        Order::Topdown
    }

    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        match x {
            Block::Div(d) if self.latex => latex_environment(d, &self.options, ctx),
            _ => None,
        }
    }

    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        if !matches!(x, Inline::Cite(_) | Inline::RawInline(_)) {
            return None;
        }
        warn_unnumbered(x, &mut self.options);
        match x {
            Inline::Cite(cite) if self.latex => cite_to_ref(cite, &self.options),
            _ if self.latex => None,
            _ => resolve(x, &self.options),
        }
    }
}

/// The two passes, with the conversion pandoc (or a host) gives.
fn amsthm(doc: &mut Pandoc, conversion: &Conversion) -> Result<(), panir::FilterError> {
    let mut first = First::default();
    panir::apply_with(doc, &mut first, conversion)?;
    let Some(options) = first.options else {
        return Ok(());
    };
    let mut second = Second {
        options,
        latex: first.latex,
    };
    panir::apply_with(doc, &mut second, conversion)
}

fn main() {
    panir::filter_with(amsthm);
}

/// The cases with the conversion's options (`NAME.opts`), run here in
/// process with them as the conversion's `options` (defaults-file keys), as
/// a host knowing them would, against what the original made (`run.py test`
/// runs the port as a JSON filter given `$PANDOC_WRITER_OPTIONS`, as
/// libpandoc gives it).
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cases_with_options() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../cases/expected.jsonl");
        let mut differ = BTreeSet::new();
        let mut checked = 0;
        for line in std::fs::read_to_string(path).unwrap().lines() {
            let case: serde_json::Value = serde_json::from_str(line).unwrap();
            let Some(options) = case.get("options") else {
                continue;
            };
            let conversion = Conversion {
                format: case["format"].as_str().map(String::from),
                options: Some(options.clone()),
                ..Conversion::default()
            };
            let mut doc = panir::from_str(&case["input"].to_string()).unwrap();
            let ran = amsthm(&mut doc, &conversion);
            let same = match (ran, case.get("output")) {
                (Ok(()), Some(want)) => {
                    let mut got: serde_json::Value =
                        serde_json::from_str(&panir::to_string(&doc)).unwrap();
                    let mut want = want.clone();
                    got.as_object_mut().unwrap().remove("pandoc-api-version");
                    want.as_object_mut().unwrap().remove("pandoc-api-version");
                    got == want
                }
                (ran, want) => ran.is_err() && want.is_none(), // (the original failed)
            };
            if !same {
                differ.insert(case["name"].as_str().unwrap().to_string());
            }
            checked += 1;
        }
        assert!(checked > 100);
        // the notes with markup, as in every language (known.tsv)
        let known = ["example-proofs", "model-source-chapter", "model-source-n"].map(String::from);
        assert_eq!(differ, BTreeSet::from(known));
    }
}
