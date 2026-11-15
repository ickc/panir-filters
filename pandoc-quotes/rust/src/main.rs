//! Quotation marks by the document's language: a pandoc filter.
//!
//! Each quotation (`Quoted`) becomes its content between the marks of the
//! language: `quot-marks` (four marks, as text of four characters or a list
//! of four), else those of `quot-lang`, else of `lang` (an RFC 5646 code: the
//! language and region, then the language alone, then one of its regions).
//! `quot-marks-by-lang` adds languages or changes them. Without any, the
//! quotations stay; a bad field or an unknown language is an error.
//!
//! A port of pandoc/lua-filters' pandoc-quotes.lua (Copyright (c) 2018 Odin
//! Kroeger; MIT, see ../LICENSE).

use std::collections::BTreeMap;

use panir::{Ctx, Filter, Inline, Meta, MetaValue, QuoteType, given, stringify, truthy};

/// Primary left, primary right, secondary left, secondary right.
type Marks = [String; 4];

/// From <https://en.wikipedia.org/w/index.php?title=Quotation_mark&oldid=836731669>.
const QUOT_MARKS_BY_LANG: &[(&str, [&str; 4])] = &[
    ("de-CH", ["«", "»", "‹", "›"]),
    ("en-CA", ["“", "”", "‘", "’"]),
    ("en-GB", ["‘", "’", "“", "”"]),
    ("en-UK", ["‘", "’", "“", "”"]),
    ("en-US", ["“", "”", "‘", "’"]),
    ("it-CH", ["«", "»", "‹", "›"]),
    ("ko-KR", ["“", "”", "‘", "’"]),
    ("pt-BR", ["“", "”", "‘", "’"]),
    ("ar", ["”", "“", "’", "‘"]),
    ("bo", ["「", "」", "『", "』"]),
    ("bs", ["”", "”", "’", "’"]),
    ("cn", ["「", "」", "『", "』"]),
    ("cs", ["„", "“", "‚", "‘"]),
    ("cy", ["‘", "’", "“", "”"]),
    ("da", ["»", "«", "›", "‹"]),
    ("de", ["„", "“", "‚", "‘"]),
    ("el", ["«", "»", "“", "”"]),
    ("en", ["“", "”", "‘", "’"]),
    ("eo", ["“", "”", "‘", "’"]),
    ("es", ["«", "»", "“", "”"]),
    ("et", ["„", "“", "‚", "‘"]),
    ("fa", ["«", "»", "‹", "›"]),
    ("fi", ["”", "”", "’", "’"]),
    ("fil", ["“", "”", "‘", "’"]),
    ("fr", ["«", "»", "‹", "›"]),
    ("ga", ["“", "”", "‘", "’"]),
    ("gd", ["‘", "’", "“", "”"]),
    ("gl", ["«", "»", "‹", "›"]),
    ("he", ["“", "”", "‘", "’"]),
    ("hi", ["“", "”", "‘", "’"]),
    ("hr", ["„", "“", "‚", "‘"]),
    ("hu", ["„", "”", "»", "«"]),
    ("ia", ["“", "”", "‘", "’"]),
    ("id", ["“", "”", "‘", "’"]),
    ("is", ["„", "“", "‚", "‘"]),
    ("it", ["«", "»", "“", "”"]),
    ("ja", ["「", "」", "『", "』"]),
    ("jbo", ["lu", "li'u", "lu", "li'u"]),
    ("ka", ["„", "“", "‚", "‘"]),
    ("khb", ["《", "》", "〈", "〉"]),
    ("kk", ["«", "»", "‹", "›"]),
    ("km", ["«", "»", "‹", "›"]),
    ("ko", ["《", "》", "〈", "〉"]),
    ("lo", ["«", "»", "‹", "›"]),
    ("lt", ["„", "“", "‚", "‘"]),
    ("lv", ["„", "“", "‚", "‘"]),
    ("mk", ["„", "“", "’", "‘"]),
    ("mn", ["«", "»", "‹", "›"]),
    ("mt", ["“", "”", "‘", "’"]),
    ("nl", ["„", "”", "‚", "’"]),
    ("no", ["«", "»", "«", "»"]),
    ("pl", ["„", "”", "»", "«"]),
    ("ps", ["«", "»", "‹", "›"]),
    ("pt", ["«", "»", "“", "”"]),
    ("rm", ["«", "»", "‹", "›"]),
    ("ro", ["„", "”", "«", "»"]),
    ("ru", ["«", "»", "„", "“"]),
    ("sk", ["„", "“", "‚", "‘"]),
    ("sl", ["„", "“", "‚", "‘"]),
    ("sq", ["„", "“", "‚", "‘"]),
    ("sr", ["„", "“", "’", "’"]),
    ("sv", ["”", "”", "’", "’"]),
    ("tdd", ["「", "」", "『", "』"]),
    ("th", ["“", "”", "‘", "’"]),
    ("thi", ["「", "」", "『", "』"]),
    ("ti", ["«", "»", "‹", "›"]),
    ("tr", ["«", "»", "‹", "›"]),
    ("ug", ["«", "»", "‹", "›"]),
    ("uk", ["«", "»", "„", "“"]),
    ("uz", ["«", "»", "„", "“"]),
    ("vi", ["“", "”", "‘", "’"]),
    ("wen", ["„", "“", "‚", "‘"]),
];

/// Four marks: text of four characters, or a list of four. (The original
/// warns, then fails as its warning does: NOTES.md.)
fn quotation_marks(value: &MetaValue, field: &str) -> Result<Marks, String> {
    let marks: Vec<String> = match value {
        MetaValue::MetaInlines(_) => stringify(value).chars().map(String::from).collect(),
        MetaValue::MetaList(items) => items.iter().map(stringify).collect(),
        _ => return Err(format!("{field}: neither a string nor a list")),
    };
    marks
        .try_into()
        .map_err(|_| format!("{field}: not four quotation marks"))
}

fn configure(meta: &Meta) -> Result<Option<Marks>, String> {
    let mut by_lang: BTreeMap<String, Marks> = QUOT_MARKS_BY_LANG
        .iter()
        .map(|(lang, marks)| (lang.to_string(), marks.map(String::from)))
        .collect();
    match meta.get("quot-marks-by-lang") {
        Some(MetaValue::MetaMap(custom)) => {
            for (lang, marks) in custom {
                let field = format!("metadata field \"quot-marks-by-lang\": lang \"{lang}\"");
                by_lang.insert(lang.to_string(), quotation_marks(marks, &field)?);
            }
        }
        other if truthy(other) => {
            return Err("metadata field \"quot-marks-by-lang\": not a map".into());
        }
        _ => {}
    }
    if let Some(marks) = given(meta.get("quot-marks")) {
        return quotation_marks(marks, "metadata field \"quot-marks\"").map(Some);
    }
    let Some(field) = ["quot-lang", "lang"]
        .iter()
        .find_map(|k| given(meta.get(*k)))
    else {
        return Ok(None);
    };
    // the language and region, the language, a region of the language
    let lang = stringify(field);
    if let Some(marks) = by_lang.get(&lang) {
        return Ok(Some(marks.clone()));
    }
    let language: String = lang.chars().take_while(char::is_ascii_alphabetic).collect(); // Lua's %a
    if language.is_empty() {
        return Err(format!("{lang}: unknown language."));
    }
    // a region: the original's Lua pattern `^lang-`, where `-` repeats the
    // last letter, takes any key starting with the language but its last
    // letter (the first in this table, where Lua's order is random: NOTES.md)
    let region = &language[..language.len() - 1];
    let found = by_lang.get(&language).or_else(|| {
        by_lang
            .iter()
            .find(|(k, _)| k.starts_with(region))
            .map(|(_, v)| v)
    });
    found
        .cloned()
        .map(Some)
        .ok_or_else(|| format!("{language}: unknown language."))
}

/// The first filter: the marks, from the metadata.
#[derive(Default)]
struct Configure {
    marks: Option<Marks>,
}

impl Filter for Configure {
    fn meta(&mut self, meta: &mut Meta, ctx: &mut Ctx) -> Option<()> {
        self.marks = ctx.ok(configure(meta))?;
        Some(())
    }
}

/// The second: every quotation, inner ones first.
struct InsertMarks {
    marks: Marks,
}

impl Filter for InsertMarks {
    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        let Inline::Quoted(quoted) = x else {
            return None;
        };
        let [left, right] = match quoted.quote_type {
            QuoteType::DoubleQuote => [&self.marks[0], &self.marks[1]],
            QuoteType::SingleQuote => [&self.marks[2], &self.marks[3]],
        };
        let mut out = vec![Inline::Str(left.into())];
        out.append(&mut quoted.content);
        out.push(Inline::Str(right.into()));
        Some(out)
    }
}

fn main() {
    panir::filter(|doc, format| {
        let mut configure = Configure::default();
        panir::apply(doc, &mut configure, format)?;
        match configure.marks {
            Some(marks) => panir::apply(doc, &mut InsertMarks { marks }, format),
            None => Ok(()),
        }
    });
}
