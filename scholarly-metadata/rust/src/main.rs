//! Normalise the authors and institutes of a document's metadata: a pandoc
//! filter.
//!
//! `author` and `institute` become lists of maps, each with a `name` and an
//! `id`; an author's `institute` becomes the numbers of its institutes in
//! `institute` (each given an `index`), which collects the top-level ones
//! and those given in an author. An author or institute may be a name, a map
//! with a `name`, or a map of one entry, its key the name and its value the
//! rest (or the name); text with commas is a list of names; an author refers
//! to an institute by identifier, by name or by number.
//!
//! A port of pandoc/lua-filters' scholarly-metadata (Copyright (c) 2017-2021
//! Albert Krewinkel, Robert Winkler; ISC; the repository's MIT licence, see
//! ../LICENSE).

use std::collections::BTreeMap;

use panir::{Ctx, Filter, Inline, Meta, MetaValue, Text, into_items, stringify, truthy};

/// A named object, as the original's Lua table.
type Named = BTreeMap<Text, MetaValue>;

/// Lua's %s (and its `tonumber`'s).
const LUA_SPACE: &[char] = &[' ', '\t', '\n', '\x0b', '\x0c', '\r'];

/// A number as Lua's `tonumber` reads a string (decimal only).
fn number(text: &str) -> Option<f64> {
    let t = text.trim_matches(LUA_SPACE);
    // (Rust's parse also takes "inf" and "nan", which Lua doesn't)
    let decimal = t
        .bytes()
        .all(|b| b.is_ascii_digit() || b"+-.eE".contains(&b));
    if decimal { t.parse().ok() } else { None }
}

fn text_of(text: &str) -> MetaValue {
    Inline::Str(text.into()).into()
}

/// A list as the original takes one: a list's items, text with commas split
/// there (each a string), anything else alone (nothing if missing).
fn as_list(value: Option<MetaValue>) -> Vec<MetaValue> {
    if let Some(v @ MetaValue::MetaInlines(_)) = &value {
        let text = stringify(v);
        let names: Vec<&str> = text.split(',').map(|s| s.trim_matches(LUA_SPACE)).collect();
        if names.len() >= 2 {
            return names.into_iter().map(MetaValue::from).collect();
        }
    }
    into_items(value)
}

/// An author or institute as a map with a name and an identifier.
fn named(obj: MetaValue) -> Result<Named, String> {
    match obj {
        MetaValue::MetaInlines(_) => {
            let id = stringify(&obj);
            Ok(Named::from([
                ("name".into(), obj),
                ("id".into(), id.into()),
            ]))
        }
        MetaValue::MetaString(_) | MetaValue::MetaBool(_) => {
            let text = stringify(&obj); // (Lua's tostring)
            Ok(Named::from([
                ("name".into(), text_of(&text)),
                ("id".into(), text.into()),
            ]))
        }
        MetaValue::MetaMap(mut entries) if entries.contains_key("name") => {
            let field = if truthy(entries.get("id")) {
                "id"
            } else {
                "name"
            };
            let id = stringify(&entries[field]);
            entries.insert("id".into(), id.into());
            Ok(entries)
        }
        MetaValue::MetaMap(entries) if entries.len() == 1 => {
            let (key, value) = entries.into_iter().next().expect("one entry");
            let mut out = match value {
                MetaValue::MetaString(_) | MetaValue::MetaInlines(_) => {
                    Named::from([("name".into(), value)])
                }
                MetaValue::MetaMap(mut more) => {
                    if !truthy(more.get("name")) {
                        more.insert("name".into(), text_of(&key));
                    }
                    more
                }
                _ => {
                    return Err(format!(
                        "scholarly-metadata: {key}: neither a name nor a map"
                    ));
                }
            };
            let id = if truthy(out.get("id")) {
                stringify(&out["id"]).into()
            } else {
                key
            };
            out.insert("id".into(), MetaValue::MetaString(id));
            Ok(out)
        }
        // a map of none or several entries; a list or blocks, which the
        // original names after Lua's tostring of them (NOTES.md)
        other => Err(format!(
            "scholarly-metadata: not a named object: {}",
            stringify(&other)
        )),
    }
}

fn ident(x: &Named) -> String {
    stringify(&x["id"]) // (a string's text)
}

/// An author's institute: a top-level one, or one of its own. (In Lua both
/// are the same table, which ownership doesn't allow.)
enum Ref {
    Known(usize),
    New(Named),
}

/// An author's institutes: by number (a string's), by identifier, or new.
fn resolve(refs: Vec<MetaValue>, institutes: &[Named]) -> Result<Vec<Ref>, String> {
    let index = |r: &MetaValue| match r {
        MetaValue::MetaString(s) => {
            number(s).filter(|n| n.fract() == 0.0 && *n >= 1.0 && *n <= institutes.len() as f64)
        }
        _ => None,
    };
    refs.into_iter()
        .map(|r| {
            if let Some(n) = index(&r) {
                return Ok(Ref::Known(n as usize - 1));
            }
            let key = stringify(&r);
            match institutes.iter().position(|x| ident(x) == key) {
                Some(i) => Ok(Ref::Known(i)),
                None => named(r).map(Ref::New),
            }
        })
        .collect()
}

/// What `from` has that `to` lacks (or has false), into `to`.
fn fill(to: &mut Named, from: Named) {
    for (key, value) in from {
        if !truthy(to.get(&key)) {
            to.insert(key, value);
        }
    }
}

fn canonicalize(meta: &mut Meta) -> Result<(), String> {
    let mut institutes: Vec<Named> = as_list(meta.remove("institute"))
        .into_iter()
        .map(named)
        .collect::<Result<_, _>>()?;
    let mut authors: Vec<Named> = as_list(meta.remove("author"))
        .into_iter()
        .map(named)
        .collect::<Result<_, _>>()?;
    let theirs: Vec<Vec<Ref>> = authors
        .iter_mut()
        .map(|a| resolve(as_list(a.remove("institute")), &institutes))
        .collect::<Result<_, _>>()?;
    let ids: Vec<Vec<String>> = theirs
        .iter()
        .map(|refs| {
            refs.iter()
                .map(|r| match r {
                    Ref::Known(i) => ident(&institutes[*i]),
                    Ref::New(inst) => ident(inst),
                })
                .collect()
        })
        .collect();
    // merged: what each has that the first of its identifier lacks
    for r in theirs.into_iter().flatten() {
        let (id, inst) = match r {
            Ref::New(inst) => (ident(&inst), inst),
            Ref::Known(i) => (ident(&institutes[i]), institutes[i].clone()),
        };
        match institutes.iter().position(|x| ident(x) == id) {
            Some(first) => fill(&mut institutes[first], inst),
            None => institutes.push(inst),
        }
    }
    let mut numbers = BTreeMap::new(); // each identifier's first institute
    for (i, inst) in institutes.iter_mut().enumerate() {
        inst.insert("index".into(), text_of(&(i + 1).to_string()));
        numbers.entry(ident(inst)).or_insert(i + 1);
    }
    for (author, ids) in authors.iter_mut().zip(ids) {
        let numbered = ids.iter().map(|id| numbers[id].to_string()).collect();
        author.insert("institute".into(), numbered);
    }
    meta.insert("author".into(), authors.into_iter().collect());
    meta.insert("institute".into(), institutes.into_iter().collect());
    Ok(())
}

struct ScholarlyMetadata;

impl Filter for ScholarlyMetadata {
    fn meta(&mut self, meta: &mut Meta, ctx: &mut Ctx) -> Option<()> {
        ctx.ok(canonicalize(meta))
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut ScholarlyMetadata, format));
}
