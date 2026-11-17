//! A code block with an `include` attribute takes the file's text: a pandoc
//! filter.
//!
//! `startLine` and `endLine` (or `start-line`, `end-line`) take a range of
//! lines, the first giving `startFrom` (for `numberLines`); `dedent=n`
//! removes the spaces among each line's first n bytes. The attributes it
//! reads are removed. A file that can't be opened leaves the block empty,
//! with a message.
//!
//! A port of pandoc/lua-filters' include-code-files (© 2020 Bruno BEAUFILS;
//! MIT, see ../LICENSE).

use panir::{Block, Ctx, Filter, lua_number};

/// Text as Lua's string functions take it for an integer argument.
fn lua_integer(text: &str, what: &str) -> Result<i64, String> {
    match lua_number(text) {
        None => Err(format!(
            "bad argument to '{what}' (number expected, got string)"
        )),
        Some(n) if n.fract() != 0.0 => Err(format!(
            "bad argument to '{what}' (number has no integer representation)"
        )),
        Some(n) => Ok(n as i64),
    }
}

/// Lua's `s:sub(i, j)`: bytes i to j, counted from 1, negative from the end.
fn lua_sub(s: &[u8], i: i64, j: i64) -> &[u8] {
    let n = s.len() as i64;
    let from = (if i < 0 { n + i + 1 } else { i }).max(1);
    let to = (if j < 0 { n + j + 1 } else { j }).min(n);
    if from <= to {
        &s[from as usize - 1..to as usize]
    } else {
        &[]
    }
}

/// The spaces among the line's first n bytes removed (Lua's strings are
/// bytes: an é is two).
fn dedent(line: &[u8], n: &str) -> Result<Vec<u8>, String> {
    let k = lua_integer(n, "sub")?;
    let mut out: Vec<u8> = lua_sub(line, 1, k)
        .iter()
        .copied()
        .filter(|&b| b != b' ')
        .collect();
    out.extend_from_slice(lua_sub(line, k + 1, -1));
    Ok(out)
}

fn need(n: Option<f64>) -> Result<f64, &'static str> {
    n.ok_or("attempt to compare nil with number")
}

struct IncludeCode;

impl Filter for IncludeCode {
    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let Block::CodeBlock(cb) = x else {
            return None;
        };
        let path = cb.attr.get("include")?.to_string();
        let mut content = Vec::new();
        // (Lua opens a directory, then fails to read it; what reading one
        // does here depends on the system and the wasm host)
        // ("" is the working directory in WASI, a missing file elsewhere)
        if !path.is_empty() && std::path::Path::new(&path).is_dir() {
            return ctx.fail(format!("{path}: a directory"));
        }
        match std::fs::read(&path) {
            Err(_) => eprintln!("Cannot open file {path} | Skipping includes"),
            Ok(data) => {
                // hyphenated attributes to camel case
                for (camel, hyphen) in [("startLine", "start-line"), ("endLine", "end-line")] {
                    if let Some(value) = cb.attr.get(hyphen).map(String::from) {
                        cb.attr.set(camel, value);
                        cb.attr.remove(hyphen);
                    }
                }
                let mut start = Some(1.0);
                if let Some(first) = cb.attr.get("startLine").map(String::from) {
                    start = lua_number(&first);
                    cb.attr.set("startFrom", first);
                }
                // Lua's `fh:lines("L")`: each line with its newline
                for (i, raw) in data.split_inclusive(|&b| b == b'\n').enumerate() {
                    let number = (i + 1) as f64;
                    let line = match cb.attr.get("dedent") {
                        Some(n) => ctx.ok(dedent(raw, n))?,
                        None => raw.to_vec(),
                    };
                    if number >= ctx.ok(need(start))? {
                        let last = cb.attr.get("endLine");
                        if last.is_none() || number <= ctx.ok(need(last.and_then(lua_number)))? {
                            content.extend(line);
                        }
                    }
                }
            }
        }
        for key in ["include", "startLine", "endLine", "dedent"] {
            cb.attr.remove(key);
        }
        cb.text = String::from_utf8_lossy(&content).into_owned().into();
        None
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut IncludeCode, format));
}
