//! Code blocks of class `table`, CSV with YAML options, made tables: a
//! pandoc filter.
//!
//! The block holds CSV, and YAML blocks (each between `---` and `---` or
//! `...`) of options: `caption`, `short-caption`, `alignment` (a letter a
//! column among L, R, C, D), `alignment-cells` (a line of letters a row),
//! `width` (a number or fraction a column, or D), `table-width` (then the
//! columns without a width get one from the length of their text), `ms`
//! (rows of the head, of each body's head and rows, of the foot), `ns-head`
//! (row head columns of each body), `markdown` (cells, captions and
//! attributes read as markdown), `fancy-table` (with markdown: the first
//! column marks the head, bodies and foot, with attributes; a cell's first
//! line its span and attributes), `include` (the CSV from a file, relative
//! to the working directory, `include-encoding` its encoding), `format`
//! (csv only). `csv-kwargs` (Python's csv.reader's arguments) is not
//! supported: a warning, then the CSV read as it is.
//!
//! The CSV is read by the `csv` crate, the YAML by `yaml-rust2` (YAML 1.2,
//! where the original's PyYAML reads 1.1), each option made the type the
//! original wants as Python makes it (`str(1.5)`, `list("12")`,
//! `bool("false")`). The markdown of the cells is pandoc's, which a Rust
//! filter gets from the host running it when built for wasm (`cargo build
//! --target wasm32-wasip1`): all of a table's texts in one call
//! (`read_many`). Built natively, as a JSON filter, it has no pandoc to
//! call, and fails at the first table with markdown, saying so.
//!
//! A port of ickc/pantable's `pantable` filter, 0.15.0 (© 2016-2026 Kolen
//! Cheung; BSD-3-Clause, see ../LICENSE).

use std::collections::HashMap;
use std::sync::LazyLock;

use panir::{
    Alignment, Attr, Block, Caption, Cell, ColSpec, ColWidth, Conversion, Ctx, Filter, Inline, Row,
    Table, TableBody, TableFoot, TableHead,
};
use regex::Regex;
use yaml_rust2::{Yaml, YamlLoader};

fn warn(message: &str) {
    eprintln!("pantable: {message}");
}

/// What the original does with an error: Python's ValueError and TypeError,
/// which it catches to fall back to a default, and anything else, which
/// fails the filter.
enum PyErr {
    Value(String),
    Fail(String),
}

type Py<T> = Result<T, PyErr>;

fn value<T>(message: impl Into<String>) -> Py<T> {
    Err(PyErr::Value(message.into()))
}

fn fail<T>(message: impl Into<String>) -> Py<T> {
    Err(PyErr::Fail(message.into()))
}

impl From<PyErr> for String {
    fn from(e: PyErr) -> String {
        match e {
            PyErr::Value(m) | PyErr::Fail(m) => m,
        }
    }
}

// The code block: YAML and data, as panflute's yaml_filter (strict_yaml)
// splits it: at lines of three or more dots or dashes; a chunk starting
// with --- opens YAML (even one of data), --- or ... closes it.

fn is_separator(line: &str) -> bool {
    line.len() >= 3 && (line.bytes().all(|b| b == b'.') || line.bytes().all(|b| b == b'-'))
}

type Options = Vec<(String, Yaml)>;

/// The options and the data; None for YAML that can't be read.
fn split(text: &str) -> Result<Option<(Options, String)>, String> {
    // the chunks between separator lines, and the separators
    let mut chunks: Vec<String> = vec![String::new()];
    for line in text.split('\n') {
        if is_separator(line) {
            chunks.push(line.to_string());
            chunks.push(String::new());
        } else {
            let last = chunks.last_mut().unwrap();
            last.push_str(line);
            last.push('\n');
        }
    }
    let mut options: Options = Vec::new();
    let mut data = Vec::new();
    let mut raw = true;
    for chunk in &chunks {
        let chunk = chunk.trim_matches('\n');
        if chunk.is_empty() {
            continue;
        }
        if raw {
            if chunk.starts_with("---") {
                raw = false;
            } else {
                data.push(chunk);
            }
        } else if chunk.starts_with("---") || chunk.starts_with("...") {
            raw = true;
        } else {
            let docs = match YamlLoader::load_from_str(chunk) {
                Ok(docs) => docs,
                Err(e) => {
                    warn(&format!("malformed YAML block: left as it is ({e})"));
                    return Ok(None);
                }
            };
            // (Python's dict.update: a map, or a list of pairs)
            for (k, v) in pairs(docs.first().unwrap_or(&Yaml::Null))? {
                let key = py_str(&k);
                match options.iter_mut().find(|(name, _)| *name == key) {
                    Some(entry) => entry.1 = v,
                    None => options.push((key, v)),
                }
            }
        }
    }
    Ok(Some((options, data.join("\n"))))
}

// A value as Python has it, made another type as Python makes it

/// Python's str() of a value.
fn py_str(v: &Yaml) -> String {
    match v {
        Yaml::String(s) => s.clone(),
        _ => py_repr(v),
    }
}

fn py_repr(v: &Yaml) -> String {
    match v {
        Yaml::Null | Yaml::BadValue => "None".into(),
        Yaml::Boolean(b) => (if *b { "True" } else { "False" }).into(),
        Yaml::Integer(i) => i.to_string(),
        Yaml::Real(_) => py_float(v.as_f64().unwrap_or(f64::NAN)),
        Yaml::String(s) => {
            let q = if s.contains('\'') && !s.contains('"') {
                '"'
            } else {
                '\''
            };
            let mut out = String::from(q);
            for c in s.chars() {
                match c {
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if c == q => out.extend(['\\', c]),
                    c if (c as u32) < 0x20 || c == '\x7f' => {
                        out.push_str(&format!("\\x{:02x}", c as u32))
                    }
                    c => out.push(c),
                }
            }
            out.push(q);
            out
        }
        Yaml::Array(xs) => format!(
            "[{}]",
            xs.iter().map(py_repr).collect::<Vec<_>>().join(", ")
        ),
        Yaml::Hash(map) => {
            let items: Vec<_> = map
                .iter()
                .map(|(k, x)| format!("{}: {}", py_repr(k), py_repr(x)))
                .collect();
            format!("{{{}}}", items.join(", "))
        }
        Yaml::Alias(_) => "None".into(),
    }
}

/// Python's repr of a float: the shortest digits, in exponent form below
/// 1e-4 and from 1e16.
fn py_float(x: f64) -> String {
    if x.is_nan() {
        return "nan".into();
    }
    if x.is_infinite() {
        return (if x > 0.0 { "inf" } else { "-inf" }).into();
    }
    let sci = format!("{x:e}");
    let (mantissa, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    if !(-4..16).contains(&exp) {
        return format!(
            "{mantissa}e{}{:02}",
            if exp < 0 { '-' } else { '+' },
            exp.abs()
        );
    }
    let s = x.to_string();
    if s.contains('.') { s } else { s + ".0" }
}

/// Python's bool().
fn py_bool(v: &Yaml) -> bool {
    match v {
        Yaml::Null | Yaml::BadValue | Yaml::Alias(_) => false,
        Yaml::Boolean(b) => *b,
        Yaml::Integer(i) => *i != 0,
        Yaml::Real(_) => v.as_f64() != Some(0.0),
        Yaml::String(s) => !s.is_empty(),
        Yaml::Array(xs) => !xs.is_empty(),
        Yaml::Hash(map) => !map.is_empty(),
    }
}

/// Python's list(): of text its characters, of a map its keys.
fn py_list(v: &Yaml) -> Py<Vec<Yaml>> {
    match v {
        Yaml::String(s) => Ok(s.chars().map(|c| Yaml::String(c.to_string())).collect()),
        Yaml::Array(xs) => Ok(xs.clone()),
        Yaml::Hash(map) => Ok(map.keys().cloned().collect()),
        _ => value(format!("'{}' object is not iterable", py_repr(v))),
    }
}

/// Python's int() of a value (`ms`, `ns-head`).
fn py_int(v: &Yaml) -> Py<i64> {
    match v {
        Yaml::Integer(i) => Ok(*i),
        Yaml::Boolean(b) => Ok(i64::from(*b)),
        Yaml::Real(_) => {
            let x = v.as_f64().unwrap_or(f64::NAN);
            if x.is_nan() {
                value("cannot convert float NaN to integer")
            } else if x.is_infinite() {
                fail("cannot convert float infinity to integer")
            } else {
                Ok(x.trunc() as i64)
            }
        }
        Yaml::String(s) => {
            digits(s.trim()).ok_or(PyErr::Value(format!("invalid literal for int(): {s}")))
        }
        _ => value(format!("int() of {}", py_repr(v))),
    }
}

/// Python's integer literal, `[-+]?\d+(_\d+)*`.
fn digits(s: &str) -> Option<i64> {
    let body = s.strip_prefix(['-', '+']).unwrap_or(s);
    let ok = !body.is_empty()
        && body
            .split('_')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    ok.then(|| s.replace('_', "").parse().ok()).flatten()
}

/// float(Fraction(v)): a number, or a fraction or decimal as text.
fn fraction(v: &Yaml) -> Py<f64> {
    match v {
        Yaml::Integer(i) => Ok(*i as f64),
        Yaml::Boolean(b) => Ok(f64::from(u8::from(*b))),
        Yaml::Real(_) => {
            let x = v.as_f64().unwrap_or(f64::NAN);
            if x.is_nan() {
                value("cannot convert NaN to integer ratio")
            } else if x.is_infinite() {
                fail("cannot convert Infinity to integer ratio")
            } else {
                Ok(x)
            }
        }
        Yaml::String(s) => fraction_of(s),
        _ => value("argument should be a string or a number"),
    }
}

/// Python's grammar of a Fraction's text: `[sign]num[/denom]`, or a
/// decimal with an exponent; blanks around, `_` between digits.
fn fraction_of(text: &str) -> Py<f64> {
    let invalid = || PyErr::Value(format!("Invalid literal for Fraction: '{text}'"));
    let s = text.trim();
    let unsigned = s.strip_prefix(['-', '+']).unwrap_or(s);
    let number = |d: &str| {
        d.split('_')
            .all(|p| !p.is_empty() && p.bytes().all(|b| b.is_ascii_digit()))
    };
    if let Some((num, denom)) = s.split_once('/') {
        let (num, denom) = (num.trim_end(), denom.trim_start());
        let n = num.strip_prefix(['-', '+']).unwrap_or(num);
        if !number(n) || !number(denom) {
            return Err(invalid());
        }
        let d: f64 = denom.replace('_', "").parse().map_err(|_| invalid())?;
        if d == 0.0 {
            return fail(format!("Fraction({num}, 0)"));
        }
        return Ok(num.replace('_', "").parse::<f64>().map_err(|_| invalid())? / d);
    }
    let (mantissa, exp) = match unsigned.find(['e', 'E']) {
        Some(i) => (&unsigned[..i], Some(&unsigned[i + 1..])),
        None => (unsigned, None),
    };
    let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let ok = (int.is_empty() || number(int))
        && (frac.is_empty() || number(frac))
        && (!int.is_empty() || !frac.is_empty())
        && exp.is_none_or(|e| number(e.strip_prefix(['-', '+']).unwrap_or(e)));
    if !ok || (int.is_empty() && !frac.as_bytes().first().is_some_and(u8::is_ascii_digit)) {
        return Err(invalid());
    }
    s.replace('_', "").parse().map_err(|_| invalid())
}

/// Python's dict(): a map, or a list of pairs.
fn pairs(v: &Yaml) -> Result<Vec<(Yaml, Yaml)>, String> {
    match v {
        Yaml::Hash(map) => Ok(map.iter().map(|(k, x)| (k.clone(), x.clone())).collect()),
        Yaml::Array(xs) => xs
            .iter()
            .map(|p| match p {
                Yaml::Array(pair) if pair.len() == 2 => Ok((pair[0].clone(), pair[1].clone())),
                Yaml::String(s) if s.chars().count() == 2 => {
                    let mut cs = s.chars().map(|c| Yaml::String(c.to_string()));
                    Ok((cs.next().unwrap(), cs.next().unwrap()))
                }
                _ => Err("dictionary update sequence element has not length 2".to_string()),
            })
            .collect(),
        _ => Err(format!("'{}' can't update a dict", py_repr(v))),
    }
}

#[derive(Default)]
struct PanTableOptions {
    short_caption: String,
    caption: String,
    alignment: String,
    alignment_cells: String,
    width: Option<Vec<Yaml>>,
    table_width: Option<f64>,
    // (header: read by the original, and never used)
    ms: Option<Vec<i64>>,
    ns_head: Option<Vec<i64>>,
    markdown: bool,
    fancy_table: bool,
    include: String,
    include_encoding: String,
    format: String,
    csv_kwargs: bool, // (given: not supported here)
}

impl PanTableOptions {
    fn of(given: &Options) -> Result<Self, String> {
        let mut o = PanTableOptions {
            format: "csv".into(),
            ..Default::default()
        };
        // keys with - or _ alike, the later winning
        let mut named: Vec<(String, &Yaml)> = Vec::new();
        for (k, v) in given {
            let name = k.replace('_', "-");
            match named.iter_mut().find(|(n, _)| *n == name) {
                Some(entry) => entry.1 = v,
                None => named.push((name, v)),
            }
        }
        let (mut ms, mut ns_head) = (None, None);
        for (name, v) in named {
            let set = match name.as_str() {
                "width" => maybe(v, py_list).map(|x| o.width = x),
                "ms" => maybe(v, py_list).map(|x| ms = x),
                "ns-head" => maybe(v, py_list).map(|x| ns_head = x),
                // (a float as it is, infinity too)
                "table-width" => match v {
                    Yaml::Real(_) => {
                        o.table_width = v.as_f64();
                        Ok(())
                    }
                    _ => maybe(v, fraction).map(|x| o.table_width = x),
                },
                // the rest can't fail: str() and bool() take anything
                other => {
                    match other {
                        "short-caption" => o.short_caption = py_str(v),
                        "caption" => o.caption = py_str(v),
                        "alignment" => o.alignment = py_str(v),
                        "alignment-cells" => o.alignment_cells = py_str(v),
                        "include" => o.include = py_str(v),
                        "include-encoding" => o.include_encoding = py_str(v),
                        "format" => o.format = py_str(v),
                        "markdown" => o.markdown = py_bool(v),
                        "fancy-table" => o.fancy_table = py_bool(v),
                        "csv-kwargs" => o.csv_kwargs = true,
                        _ => {}
                    }
                    Ok(())
                }
            };
            match set {
                Err(PyErr::Value(e)) => warn(&format!(
                    "option {name} with value {}: {e}: the default",
                    py_str(v)
                )),
                Err(PyErr::Fail(e)) => return Err(e),
                Ok(()) => {}
            }
        }
        // ms and ns-head: lists of int(), else the default
        for (name, list, out) in [("ms", ms, &mut o.ms), ("ns-head", ns_head, &mut o.ns_head)] {
            *out = match list
                .map(|xs: Vec<Yaml>| xs.iter().map(py_int).collect::<Py<Vec<_>>>())
                .transpose()
            {
                Ok(x) => x,
                Err(PyErr::Value(e)) => {
                    warn(&format!("option {name}: {e}: the default"));
                    None
                }
                Err(PyErr::Fail(e)) => return Err(e),
            };
        }
        Ok(o)
    }

    /// What is wrong beyond the type, for a table of m rows and n columns:
    /// the widths, None for the default.
    fn normalize(&mut self, m: usize, n: usize) -> Result<Vec<Option<f64>>, String> {
        let mut total = 0.0;
        let mut widths = vec![None; n];
        for (i, w) in self.width.iter().flatten().take(n).enumerate() {
            match fraction(w) {
                Ok(x) if x >= 0.0 => {
                    widths[i] = Some(x);
                    total += x;
                }
                Ok(_) | Err(PyErr::Value(_)) => {}
                Err(PyErr::Fail(e)) => return Err(e),
            }
        }
        if self.table_width.is_some_and(|w| w < total) {
            warn(&format!(
                "table-width smaller than the sum of the widths, {total}: dropped"
            ));
            self.table_width = None;
        }
        if let Some(ms) = &self.ms
            && (ms.len() < 4
                || ms.len() % 2 == 1
                || ms.iter().any(|&x| x < 0)
                || ms.iter().sum::<i64>() != m as i64)
        {
            warn(&format!(
                "ms {ms:?}: too short, of odd length, negative or not adding up to the rows, {m}: dropped"
            ));
            self.ms = None;
        }
        let bodies = self.ms.as_ref().map_or(1, |ms| ms.len() / 2 - 1);
        if let Some(ns) = &self.ns_head
            && (ns.len() != bodies || ns.iter().any(|&x| x > n as i64))
        {
            warn(&format!(
                "ns-head {ns:?}: not one a body, {bodies}, or more than the columns, {n}: dropped"
            ));
            self.ns_head = None;
        }
        Ok(widths)
    }
}

/// None stays None (Python's `value is None`), else `f` of it.
fn maybe<T>(v: &Yaml, f: impl Fn(&Yaml) -> Py<T>) -> Py<Option<T>> {
    match v {
        Yaml::Null => Ok(None),
        _ => f(v).map(Some),
    }
}

const ALIGN: [Alignment; 4] = [
    Alignment::AlignDefault,
    Alignment::AlignLeft,
    Alignment::AlignRight,
    Alignment::AlignCenter,
];

/// A row of alignments, by the original's arithmetic on each letter's code
/// (L, R, C, D, and anything else somewhere); all default if one isn't
/// ASCII.
fn aligns(text: &str, n: usize) -> Vec<Alignment> {
    let letters = text.trim().to_uppercase();
    if !letters.is_ascii() {
        warn(&format!("non-ASCII character in {letters}: the default"));
        return vec![ALIGN[0]; n];
    }
    let mut out: Vec<Alignment> = letters
        .bytes()
        .take(n)
        .map(|c| ALIGN[(i32::from(c) - 3).rem_euclid(11) as usize % 6 % 4])
        .collect();
    out.resize(n, ALIGN[0]);
    out
}

/// Python's str.splitlines.
fn lines(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut chars = s.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if matches!(
            c,
            '\n' | '\r'
                | '\x0b'
                | '\x0c'
                | '\x1c'
                | '\x1d'
                | '\x1e'
                | '\u{85}'
                | '\u{2028}'
                | '\u{2029}'
        ) {
            out.push(&s[start..i]);
            start = i + c.len_utf8();
            if c == '\r' && chars.peek().is_some_and(|&(_, d)| d == '\n') {
                chars.next();
                start += 1;
            }
        }
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

// The data

enum LoadError {
    NotFound(String),
    Empty,
    Fail(String),
}

/// A file's bytes as text in an encoding named as Python names it: UTF-8
/// and Latin-1 by the standard library, the others by encoding_rs (WHATWG's
/// names; its Latin-1 is windows-1252).
fn decode(bytes: &[u8], name: &str) -> Result<String, String> {
    let key: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    let invalid = || format!("can't decode the file as {name}");
    match key.as_str() {
        "utf_8" | "utf8" | "u8" | "utf" | "cp65001" => {
            String::from_utf8(bytes.to_vec()).map_err(|_| invalid())
        }
        "utf_8_sig" | "utf8_sig" => String::from_utf8(
            bytes
                .strip_prefix(b"\xef\xbb\xbf")
                .unwrap_or(bytes)
                .to_vec(),
        )
        .map_err(|_| invalid()),
        "latin_1" | "latin1" | "iso_8859_1" | "iso8859_1" | "8859" | "cp819" | "l1" | "latin" => {
            Ok(bytes.iter().map(|&b| char::from(b)).collect())
        }
        "utf_16" | "utf16" => {
            let encoding = if bytes.starts_with(b"\xfe\xff") {
                encoding_rs::UTF_16BE
            } else {
                encoding_rs::UTF_16LE
            };
            let (text, errors) = encoding.decode_with_bom_removal(bytes);
            if errors {
                Err(invalid())
            } else {
                Ok(text.into_owned())
            }
        }
        _ => {
            let encoding = encoding_rs::Encoding::for_label(key.replace('_', "-").as_bytes())
                .ok_or(format!("unknown encoding: {name}"))?;
            let (text, errors) = encoding.decode_without_bom_handling(bytes);
            if errors {
                Err(invalid())
            } else {
                Ok(text.into_owned())
            }
        }
    }
}

/// The rows, as long as the longest.
fn load_csv(data: &str, options: &PanTableOptions) -> Result<Vec<Vec<String>>, LoadError> {
    let text = if options.include.is_empty() {
        data.to_string()
    } else {
        let bytes = std::fs::read(&options.include).map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => {
                LoadError::NotFound(format!("include path {} not found.", options.include))
            }
            _ => LoadError::Fail(format!("{}: {e}", options.include)),
        })?;
        let encoding = if options.include_encoding.is_empty() {
            "utf-8"
        } else {
            &options.include_encoding
        };
        decode(&bytes, encoding).map_err(LoadError::Fail)?
    };
    if options.csv_kwargs {
        warn("csv-kwargs (Python's csv.reader's) is not supported in this port: read as CSV");
    }
    // (the csv crate's defaults, but for a first row that is no header, and
    // rows of other lengths, which pantable pads)
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(text.as_bytes());
    let rows: Vec<Vec<String>> = reader
        .records()
        .map(|r| r.map(|r| r.iter().map(String::from).collect()))
        .collect::<Result<_, _>>()
        .map_err(|e| LoadError::Fail(e.to_string()))?;
    if !rows.iter().flatten().any(|cell| !cell.trim().is_empty()) {
        return Err(LoadError::Empty);
    }
    let n = rows.iter().map(Vec::len).max().unwrap_or(0);
    Ok(rows
        .into_iter()
        .map(|mut row| {
            row.resize(n, String::new());
            row
        })
        .collect())
}

/// A table's texts, cells spanning others: each covered cell holds its
/// span's text and anchor, as the original's array does (a later span
/// overwrites an earlier one's).
struct Grid {
    texts: Vec<Vec<Option<String>>>,
    spans: HashMap<(usize, usize), (usize, usize, usize, usize)>, // (rows, columns, anchor)
}

impl Grid {
    fn new(texts: Vec<Vec<Option<String>>>) -> Grid {
        Grid {
            texts,
            spans: HashMap::new(),
        }
    }

    fn put(&mut self, text: &str, rows: usize, cols: usize, i: usize, j: usize) {
        for a in i..i + rows {
            for b in j..j + cols {
                self.texts[a][b] = Some(text.to_string());
                if (rows, cols) != (1, 1) {
                    self.spans.insert((a, b), (rows, cols, i, j));
                }
            }
        }
    }

    fn shape(&self, i: usize, j: usize) -> (usize, usize) {
        self.spans
            .get(&(i, j))
            .map_or((1, 1), |&(r, c, _, _)| (r, c))
    }

    fn is_at(&self, i: usize, j: usize) -> bool {
        self.spans
            .get(&(i, j))
            .is_none_or(|&(r, c, a, b)| (r, c) == (1, 1) || (a, b) == (i, j))
    }

    fn text(&self, i: usize, j: usize) -> &str {
        self.texts[i][j].as_deref().unwrap_or("")
    }
}

static CELL_LINE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\([0-9, ]+\))?(\{[^{}]*\})?$").unwrap());
static FANCY: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(\{[^{}]*\})?? ?(---|===|___)? ?(\{[^{}]*\})?$").unwrap());

/// The cells of a table of markdown: a cell's first line may be its span,
/// `(rows, columns)`, and attributes, `{...}`; with `fancy`, the first
/// column marks the ends of the head (===), bodies' heads (---), bodies
/// (___) and the foot (===), with attributes. Attributes as markdown,
/// `[]{...}`, or "".
struct Markdown {
    grid: Grid,
    ms: Option<Vec<i64>>,
    block_attrs: Option<Vec<String>>,
    row_attrs: Vec<String>,
    cell_attrs: Vec<Vec<String>>,
}

fn markdown_cells(rows: &[Vec<String>], fancy: bool) -> Markdown {
    let off = usize::from(fancy);
    let (m, n) = (rows.len(), rows[0].len() - off);
    let mut grid = Grid::new(vec![vec![None; n]; m]);
    let mut cell_attrs = vec![vec![String::new(); n]; m];
    for i in 0..m {
        for j in 0..n {
            if grid.texts[i][j].is_some() {
                continue; // (in a span)
            }
            let string = &rows[i][j + off];
            let ls = lines(string);
            let (mut attr, mut shape, mut content) = (String::new(), (1, 1), string.clone());
            if let Some(found) = ls.first().and_then(|l| CELL_LINE.captures(l)) {
                let span = found.get(1).map_or("", |x| x.as_str());
                let braces = found.get(2).map_or("", |x| x.as_str());
                let size: Option<Vec<usize>> = if span.is_empty() {
                    Some(vec![1, 1])
                } else {
                    span[1..span.len() - 1]
                        .split(',')
                        .map(|x| x.trim().parse().ok())
                        .collect()
                };
                match size.as_deref() {
                    None => warn(&format!("invalid cell shape {span}: ignored")),
                    Some(&[r, c]) if r > 0 && c > 0 => {
                        if r + i > m || c + j > n {
                            warn(&format!(
                                "a cell overflowing the table: its attributes ignored: {string}"
                            ));
                        } else {
                            attr = if braces.is_empty() {
                                String::new()
                            } else {
                                format!("[]{braces}")
                            };
                            shape = (r, c);
                            content = ls[1..].join("\n");
                        }
                    }
                    Some(size) => warn(&format!("invalid cell shape {size:?}: ignored")),
                }
            }
            cell_attrs[i][j] = attr;
            grid.put(&content, shape.0, shape.1, i, j);
        }
    }
    let mut row_attrs = vec![String::new(); m];
    let (mut ms, mut block_attrs) = (None, None);
    if fancy {
        let mut markers = Vec::new(); // (row, marker, the block's attributes)
        for (i, row) in rows.iter().enumerate() {
            if row[0].trim().is_empty() {
                continue;
            }
            let Some(found) = FANCY.captures(&row[0]) else {
                warn(&format!(
                    "can't read the fancy-table cell {}: ignored",
                    row[0]
                ));
                continue;
            };
            if let Some(marker) = found.get(2) {
                markers.push((i, marker.as_str(), found.get(1).map_or("", |x| x.as_str())));
            }
            if let Some(a) = found.get(3) {
                row_attrs[i] = format!("[]{}", a.as_str());
            }
        }
        if !markers.is_empty() {
            let (ms_, attrs) = fancy_blocks(&markers, rows);
            ms = Some(ms_);
            block_attrs = Some(attrs);
        }
    }
    Markdown {
        grid,
        ms,
        block_attrs,
        row_attrs,
        cell_attrs,
    }
}

/// `ms` and the blocks' attributes from the markers.
fn fancy_blocks(markers: &[(usize, &str, &str)], rows: &[Vec<String>]) -> (Vec<i64>, Vec<String>) {
    let sizes: Vec<i64> = markers
        .iter()
        .enumerate()
        .map(|(k, m)| m.0 as i64 - if k > 0 { markers[k - 1].0 as i64 } else { -1 })
        .collect();
    let head = markers[0].1 == "===";
    let foot = markers.len() > 1 && markers[markers.len() - 1].1 == "===";
    #[derive(Default)]
    struct Body<'a> {
        head: Option<(i64, &'a str)>,
        body: Option<(i64, &'a str)>,
    }
    let mut bodies: Vec<Body> = Vec::new();
    for k in usize::from(head)..markers.len() - usize::from(foot) {
        let (i, marker, attr) = markers[k];
        match marker {
            "___" => match bodies.last_mut() {
                Some(last) if last.body.is_none() => last.body = Some((sizes[k], attr)),
                _ => bodies.push(Body {
                    body: Some((sizes[k], attr)),
                    ..Default::default()
                }),
            },
            "---" => bodies.push(Body {
                head: Some((sizes[k], attr)),
                ..Default::default()
            }),
            _ => warn(&format!(
                "a fancy-table row neither head nor foot: ignored: {}",
                rows[i][0]
            )),
        }
    }
    let wrap = |a: &str| {
        if a.is_empty() {
            String::new()
        } else {
            format!("[]{a}")
        }
    };
    let mut ms = vec![if head { sizes[0] } else { 0 }];
    let mut attrs = vec![if head {
        wrap(markers[0].2)
    } else {
        String::new()
    }];
    for body in &bodies {
        let mut attr = "";
        for (size, a) in [body.head.unwrap_or((0, "")), body.body.unwrap_or((0, ""))] {
            ms.push(size);
            if !a.is_empty() {
                attr = a; // (the body's rows' attributes win)
            }
        }
        attrs.push(wrap(attr));
    }
    ms.push(if foot { sizes[sizes.len() - 1] } else { 0 });
    attrs.push(if foot {
        wrap(markers[markers.len() - 1].2)
    } else {
        String::new()
    });
    (ms, attrs)
}

/// The longest line's characters (code points, as Python counts), and 3
/// (as pandoc's widths).
fn text_width(s: &str) -> usize {
    lines(s)
        .iter()
        .map(|l| l.chars().count() + 3)
        .max()
        .unwrap_or(3)
}

enum Found {
    One(usize),
    Spanning(usize, usize),
}

/// The columns without a width get one, by the length of their text, so
/// that the widths add up to `table_width`; a cell spanning columns gives
/// what its first column lacks to the next. The first m rows only.
fn auto_width(
    grid: &Grid,
    m: usize,
    widths: &[Option<f64>],
    table_width: f64,
) -> Result<Vec<Option<f64>>, String> {
    let n = widths.len();
    let mut found: Vec<Vec<Found>> = (0..n).map(|_| Vec::new()).collect();
    for i in 0..m {
        for (j, column) in found.iter_mut().enumerate() {
            if grid.is_at(i, j) {
                let (w, cols) = (text_width(grid.text(i, j)), grid.shape(i, j).1);
                column.push(if cols > 1 {
                    Found::Spanning(w, cols)
                } else {
                    Found::One(w)
                });
            }
        }
    }
    let mut lengths = Vec::with_capacity(n);
    for j in 0..n {
        let longest = found[j]
            .iter()
            .filter_map(|f| {
                if let Found::One(w) = f {
                    Some(*w)
                } else {
                    None
                }
            })
            .max()
            .ok_or("max() arg is an empty sequence: a column of spanning cells only")?;
        lengths.push(longest);
        let next: Vec<Found> = found[j]
            .iter()
            .filter_map(|f| match *f {
                Found::Spanning(w, cols) if w > longest => Some(if cols > 2 {
                    Found::Spanning(w - longest, cols - 1)
                } else {
                    Found::One(w - longest)
                }),
                _ => None,
            })
            .collect();
        if !next.is_empty() {
            found[j + 1].extend(next);
        }
    }
    let unset: Vec<usize> = (0..n).filter(|&j| widths[j].is_none()).collect();
    let total: usize = unset.iter().map(|&j| lengths[j]).sum();
    let mut out = widths.to_vec();
    if total > 0 {
        let scale = (table_width - widths.iter().flatten().sum::<f64>()) / total as f64;
        for j in unset {
            out[j] = Some(lengths[j] as f64 * scale);
        }
    } else {
        for j in unset {
            out[j] = Some(0.0);
        }
    }
    Ok(out)
}

// The table

/// The caption, the anchored cells' contents and attributes, the rows'
/// attributes, the blocks' attributes.
type Parts = (Caption, Vec<Vec<Block>>, Vec<Attr>, Vec<Attr>, Vec<Attr>);

/// Each text's blocks, read by pandoc as markdown, the conversion's way, in
/// one call: the host's pandoc, built for wasm. Tabs are expanded, as the
/// original's `pandoc` command does (a read with a conversion keeps them).
#[cfg(target_family = "wasm")]
fn read_markdown(texts: &[&str], conversion: &Conversion) -> Result<Vec<Vec<Block>>, String> {
    let mut options = libpandoc::read_options(conversion, Some("markdown"));
    options["preserve-tabs"] = false.into();
    let docs = libpandoc::read_many(texts, &options)
        .and_then(libpandoc::ReadMany::all)
        .map_err(|e| e.to_string())?;
    Ok(docs.into_iter().map(|doc| doc.blocks).collect())
}

/// Natively, a JSON filter has no pandoc to call.
#[cfg(not(target_family = "wasm"))]
fn read_markdown(_: &[&str], _: &Conversion) -> Result<Vec<Vec<Block>>, String> {
    Err("pantable: markdown cells need pandoc's reader, which a Rust filter gets from the host running it: \
         build it for wasm32-wasip1 and run it in pandocrs, pandocpy, pandocjl or libpandoc.wasm"
        .into())
}

/// The attributes of `[]{...}` read: its span's; else none.
fn attr_of(blocks: Vec<Block>) -> Attr {
    if let Some(Block::Plain(inlines) | Block::Para(inlines)) = blocks.into_iter().next() {
        if let Some(Inline::Span(span)) = inlines.into_iter().next() {
            return span.attr;
        }
        warn("can't read the attributes: none");
    }
    Attr::default()
}

/// A cell of one paragraph is plain text.
fn plain(mut blocks: Vec<Block>) -> Vec<Block> {
    if let [Block::Para(inlines)] = blocks.as_mut_slice() {
        return vec![Block::Plain(std::mem::take(inlines))];
    }
    blocks
}

fn str_(text: &str) -> Inline {
    Inline::Str(text.into())
}

fn table(block: &panir::CodeBlock, conversion: &Conversion) -> Result<Option<Vec<Block>>, String> {
    let Some((given, data)) = split(&block.text)? else {
        return Ok(None);
    };
    let mut options = PanTableOptions::of(&given)?;
    if options.format != "csv" {
        return Err(format!("Unknown format: {}", options.format));
    }
    let rows = match load_csv(&data, &options) {
        Ok(rows) => rows,
        Err(LoadError::NotFound(e)) => {
            warn(&format!("{e} The code block is left as it is."));
            return Ok(None);
        }
        Err(LoadError::Empty) => {
            warn("table is empty: deleted");
            return Ok(Some(Vec::new()));
        }
        Err(LoadError::Fail(e)) => return Err(e),
    };
    let md = options
        .markdown
        .then(|| markdown_cells(&rows, options.fancy_table));
    let text_grid;
    let grid = match &md {
        Some(md) => &md.grid,
        None => {
            text_grid = Grid::new(
                rows.into_iter()
                    .map(|row| row.into_iter().map(Some).collect())
                    .collect(),
            );
            &text_grid
        }
    };
    let (rows_in, n) = (grid.texts.len(), grid.texts[0].len());
    let mut widths = options.normalize(rows_in, n)?;
    let ms = md
        .as_ref()
        .and_then(|md| md.ms.clone())
        .or(options.ms.clone())
        .unwrap_or(vec![1, 0, rows_in as i64 - 1, 0]);
    // the table has the rows of its blocks: with fancy-table, the rows after
    // the last marker are dropped (as the original, which sizes it by ms)
    let m = ms.iter().sum::<i64>() as usize;
    let ns_head = options.ns_head.clone().unwrap_or(vec![0; ms.len() / 2 - 1]);
    if let Some(tw) = options.table_width {
        widths = auto_width(grid, m, &widths, tw)?;
    }
    let col_specs = aligns(&options.alignment, n)
        .into_iter()
        .zip(&widths)
        .map(|(alignment, w)| ColSpec {
            alignment,
            width: w.map_or(ColWidth::ColWidthDefault, ColWidth::ColWidth),
        })
        .collect();
    let align_lines = lines(options.alignment_cells.trim());
    let cell_aligns: Vec<Vec<Alignment>> = (0..m)
        .map(|i| aligns(align_lines.get(i).unwrap_or(&""), n))
        .collect();

    let anchors: Vec<(usize, usize)> = (0..m)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .filter(|&(i, j)| grid.is_at(i, j))
        .collect();
    let blocks = ms.len() / 2 + 1;
    let (caption, mut contents, mut attrs, row_attrs, block_attrs): Parts;
    if let Some(md) = &md {
        // every text of the table in one call: captions, cells and their
        // attributes, rows' and blocks' attributes
        let none = vec![String::new(); blocks];
        let texts: Vec<&str> = [options.caption.as_str(), options.short_caption.as_str()]
            .into_iter()
            .chain(anchors.iter().map(|&(i, j)| grid.text(i, j)))
            .chain(anchors.iter().map(|&(i, j)| md.cell_attrs[i][j].as_str()))
            .chain(md.row_attrs[..m].iter().map(String::as_str))
            .chain(
                md.block_attrs
                    .as_ref()
                    .unwrap_or(&none)
                    .iter()
                    .map(String::as_str),
            )
            .collect();
        let todo: Vec<&str> = texts.iter().copied().filter(|t| !t.is_empty()).collect();
        let mut done = if todo.is_empty() {
            Vec::new()
        } else {
            read_markdown(&todo, conversion)?
        }
        .into_iter();
        let mut got: Vec<Vec<Block>> = texts
            .iter()
            .map(|t| {
                if t.is_empty() {
                    Vec::new()
                } else {
                    done.next().unwrap()
                }
            })
            .collect();
        let k = anchors.len();
        let rest = got.split_off(2);
        let mut rest = rest.into_iter();
        let short = got.pop().unwrap().into_iter().next().map(|b| match b {
            Block::Plain(inlines) | Block::Para(inlines) => Ok(inlines),
            Block::Header(h) => Ok(h.content),
            other => Err(format!("a short caption of {}: not inlines", other.tag())),
        });
        caption = Caption {
            short: short.transpose()?,
            content: got.pop().unwrap(),
        };
        contents = rest.by_ref().take(k).map(plain).collect();
        attrs = rest.by_ref().take(k).map(attr_of).collect();
        row_attrs = rest.by_ref().take(m).map(attr_of).collect();
        block_attrs = rest.map(attr_of).collect();
    } else {
        caption = Caption {
            short: Some(vec![str_(&options.short_caption)]),
            content: vec![Block::Para(vec![str_(&options.caption)])],
        };
        contents = anchors
            .iter()
            .map(|&(i, j)| vec![Block::Plain(vec![str_(grid.text(i, j))])])
            .collect();
        attrs = vec![Attr::default(); anchors.len()];
        row_attrs = vec![Attr::default(); m];
        block_attrs = vec![Attr::default(); blocks];
    }
    let at: HashMap<(usize, usize), usize> =
        anchors.iter().enumerate().map(|(k, &a)| (a, k)).collect();
    let mut row_attrs = row_attrs.into_iter();
    let mut rows: Vec<Row> = (0..m)
        .map(|i| Row {
            attr: row_attrs.next().unwrap_or_default(),
            cells: (0..n)
                .filter_map(|j| {
                    let k = *at.get(&(i, j))?;
                    let (row_span, col_span) = grid.shape(i, j);
                    Some(Cell {
                        attr: std::mem::take(&mut attrs[k]),
                        alignment: cell_aligns[i][j],
                        row_span: row_span as i64,
                        col_span: col_span as i64,
                        content: std::mem::take(&mut contents[k]),
                    })
                })
                .collect(),
        })
        .collect();
    // the rows of each block
    let mut parts: Vec<Vec<Row>> = Vec::new();
    for size in &ms[..ms.len() - 1] {
        let take = (*size).clamp(0, rows.len() as i64) as usize;
        parts.push(rows.drain(..take).collect());
    }
    parts.push(rows);
    let mut block_attrs = block_attrs.into_iter();
    let mut part = parts.into_iter();
    let head = TableHead {
        attr: block_attrs.next().unwrap_or_default(),
        rows: part.next().unwrap_or_default(),
    };
    let mut parts: Vec<Vec<Row>> = part.collect();
    let foot_rows = parts.pop().unwrap_or_default();
    let mut parts = parts.into_iter();
    let mut bodies = Vec::new();
    for ns in &ns_head {
        let (Some(head), Some(body)) = (parts.next(), parts.next()) else {
            return Err(format!("ns-head {ns_head:?}: more than the bodies"));
        };
        bodies.push(TableBody {
            attr: block_attrs.next().unwrap_or_default(),
            row_head_columns: *ns,
            head,
            body,
        });
    }
    let foot = TableFoot {
        attr: block_attrs.last().unwrap_or_default(),
        rows: foot_rows,
    };
    let attr = Attr {
        identifier: block.attr.identifier.clone(),
        classes: block
            .attr
            .classes
            .iter()
            .filter(|c| c.as_str() != "table")
            .cloned()
            .collect(),
        attributes: block.attr.attributes.clone(),
    };
    Ok(Some(vec![Block::Table(Box::new(Table {
        attr,
        caption,
        col_specs,
        head,
        bodies,
        foot,
    }))]))
}

struct Pantable;

impl Filter for Pantable {
    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let Block::CodeBlock(block) = x else {
            return None;
        };
        if !block.attr.classes.iter().any(|c| c == "table") {
            return None;
        }
        let made = table(block, ctx.conversion());
        ctx.ok(made)?
    }
}

fn main() {
    panir::filter_with(|doc, conversion| panir::apply_with(doc, &mut Pantable, conversion));
}
