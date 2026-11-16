//! Several bibliographies, one a topic: a pandoc filter.
//!
//! The metadata's `bibliography` maps topics to files; a div `#refs-TOPIC`
//! (or `refs_TOPIC`, `refsTOPIC`) gets the bibliography of that topic's
//! file, of every work the document cites. The document's citations are
//! resolved with all the files together.
//!
//! citeproc is pandoc's (`pandoc.utils.citeproc` in the original), which a
//! Rust filter gets from the host running it when built for wasm (`cargo
//! build --target wasm32-wasip1`: libpandoc's `convert` goes to pandocrs,
//! pandocpy, pandocjl or libpandoc.wasm). Built natively, as a JSON filter,
//! it has no pandoc to call, and fails, saying so.
//!
//! A port of pandoc-ext/multibib (Copyright © 2018-2024 Albert Krewinkel;
//! MIT, see ../LICENSE).

use panir::{Attr, Block, Ctx, Div, Filter, Inline, Meta, MetaValue, Pandoc, stringify};

/// What citeproc reads of the metadata, copied for a topic's bibliography.
const FIELDS: [&str; 11] = [
    "bibliography",
    "references",
    "csl",
    "citation-style",
    "link-citations",
    "citation-abbreviations",
    "lang",
    "suppress-bibliography",
    "reference-section-title",
    "notes-after-punctuation",
    "nocite",
];

/// pandoc's citeproc on a document (the host's, its warnings in its log).
#[cfg(target_family = "wasm")]
fn citeproc(doc: &Pandoc) -> Result<Pandoc, String> {
    let options = serde_json::json!({"from": "json", "to": "json", "citeproc": true});
    let out = libpandoc::convert(&options, Some(panir::to_string(doc).as_bytes()))
        .map_err(|e| e.to_string())?;
    let text = String::from_utf8(out.bytes).map_err(|e| e.to_string())?;
    panir::from_str(&text).map_err(|e| e.to_string())
}

/// Natively, a JSON filter has no pandoc to call.
#[cfg(not(target_family = "wasm"))]
fn citeproc(_: &Pandoc) -> Result<Pandoc, String> {
    Err(
        "multibib needs pandoc's citeproc, which a Rust filter gets from the host running it: \
         build it for wasm32-wasip1 and run it in pandocrs, pandocpy, pandocjl or libpandoc.wasm"
            .into(),
    )
}

fn refs_div() -> Block {
    Block::Div(Box::new(Div {
        attr: Attr::new("refs", &[], &[]),
        content: vec![],
    }))
}

/// Every citation of the document, and its metadata, as the first pass saw
/// them.
#[derive(Default)]
struct Collect {
    cites: Vec<Inline>,
    meta: Meta,
}

impl Filter for Collect {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        if matches!(x, Inline::Cite(_)) {
            self.cites.push(x.clone());
        }
        None
    }

    fn meta(&mut self, meta: &mut Meta, _: &mut Ctx) -> Option<()> {
        self.meta = meta.clone();
        None
    }
}

/// The citations resolved with every bibliography at once; the
/// bibliography's div, as citeproc made it.
fn resolve_doc_citations(doc: &mut Pandoc) -> Result<Block, String> {
    // (Lua's pandoc.utils.type: a map is a "table"; its values in any order)
    let files = match doc.meta.get("bibliography") {
        Some(MetaValue::MetaMap(m)) => m
            .values()
            .map(|v| MetaValue::MetaString(stringify(v).into()))
            .collect(),
        _ => vec![],
    };
    doc.meta
        .insert("bibliography".into(), MetaValue::MetaList(files));
    doc.blocks.push(refs_div());
    *doc = citeproc(doc)?;
    let refs = doc.blocks.pop().ok_or("citeproc left no blocks")?;
    // (the original restores `orig_bib`, which it never set: the field goes)
    doc.meta.remove("bibliography");
    Ok(refs)
}

struct Topics {
    collected: Collect,
    refs: Block,
}

impl Filter for Topics {
    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let Block::Div(div) = x else {
            return None;
        };
        let name = div.attr.identifier.strip_prefix("refs")?;
        let name = name.strip_prefix(['-', '_']).unwrap_or(name);
        if !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return None;
        }
        // (a map's entry; Lua indexes a list or text by a name to nothing)
        let Some(MetaValue::MetaMap(files)) = self.collected.meta.get("bibliography") else {
            return None;
        };
        let bibfile = files.get(name)?.clone();
        let mut meta: Meta = FIELDS
            .iter()
            .filter_map(|&k| Some((k.into(), self.collected.meta.get(k)?.clone())))
            .collect();
        meta.insert("bibliography".into(), bibfile);
        let mut doc = Pandoc::new(vec![Block::Para(self.collected.cites.clone()), refs_div()]);
        doc.meta = meta;
        let res = ctx.ok(citeproc(&doc))?;
        let Some(Block::Div(filled)) = res.blocks.into_iter().nth(1) else {
            return ctx.fail("citeproc's result has no bibliography's div");
        };
        let Block::Div(refs) = &self.refs else {
            return ctx.fail("citeproc's bibliography is not a div");
        };
        div.content = filled.content;
        let mut classes = Vec::new();
        for c in &refs.attr.classes {
            if !classes.contains(c) {
                classes.push(c.clone());
            }
        }
        div.attr.classes = classes;
        div.attr.attributes = refs.attr.attributes.clone();
        if matches!(ctx.format(), Some("odt" | "docx")) {
            // pandoc gives Word's bibliography this style
            let style = div
                .attr
                .get("custom-style")
                .filter(|s| !s.is_empty())
                .unwrap_or("Bibliography")
                .to_string();
            div.attr.set("custom-style", style);
        }
        None
    }
}

fn main() {
    panir::filter_with(|doc, conversion| {
        let mut collected = Collect::default();
        panir::apply_with(doc, &mut collected, conversion)?;
        let refs = match resolve_doc_citations(doc) {
            Ok(refs) => refs,
            Err(e) => {
                eprintln!("multibib: {e}");
                std::process::exit(1);
            }
        };
        panir::apply_with(doc, &mut Topics { collected, refs }, conversion)
    });
}
