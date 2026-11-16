//! Code blocks of class `include` replaced by the files they name: a pandoc
//! filter.
//!
//! Each line of the block names a file (`//` starts a comment); each is
//! read by pandoc, as markdown or as the block's `format`, with the
//! document's reader options, and its blocks (not its metadata) take the
//! block's place. Headings shift by `shift-heading-level-by`, or, with
//! `include-auto` in the metadata, by the level of the last heading before
//! the block. Included files may include others, named relative to their
//! own directory; relative links, images and `include` attributes
//! (include-code-files') in an included file are rewritten relative to the
//! including one.
//!
//! Reading the files is Rust's; parsing them is pandoc's (`pandoc.read` in
//! the original), which a Rust filter gets from the host running it when
//! built for wasm (`cargo build --target wasm32-wasip1`: libpandoc's calls
//! go to pandocrs, pandocpy, pandocjl or libpandoc.wasm). Built natively,
//! as a JSON filter, it has no pandoc to call, and fails at the first file
//! it would read, saying so.
//!
//! A port of pandoc-ext/include-files (© 2019–2021 Albert Krewinkel; MIT,
//! see ../LICENSE).

use panir::{Block, Conversion, Ctx, Filter, Inline, Meta, Pandoc, truthy};

/// A number as Lua's `tonumber` reads text, decimal or hexadecimal, blanks
/// around it; None if not one.
fn lua_number(text: &str) -> Option<f64> {
    let text = text.trim_matches([' ', '\t', '\n', '\x0b', '\x0c', '\r']);
    let digits = text.strip_prefix(['+', '-']).unwrap_or(text);
    if let Some(hex) = digits
        .strip_prefix("0x")
        .or_else(|| digits.strip_prefix("0X"))
    {
        let n = i64::from_str_radix(hex, 16).ok()? as f64;
        return Some(if text.starts_with('-') { -n } else { n });
    }
    // (f64's parse also takes `inf` and `nan`, which Lua's doesn't)
    let decimal = digits
        .chars()
        .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '+' | '-'));
    if decimal && digits.starts_with(|c: char| c.is_ascii_digit() || c == '.') {
        text.parse().ok()
    } else {
        None
    }
}

// pandoc.path's functions, as pandoc has them (Haskell's System.FilePath,
// POSIX): `normalize` drops `.` and doubled slashes but keeps `..`, unlike
// std::path's components.

fn is_relative(path: &str) -> bool {
    !path.starts_with('/')
}

fn filename(path: &str) -> &str {
    path.rsplit('/').next().unwrap_or(path)
}

fn directory(path: &str) -> String {
    match path.rfind('/') {
        None if path.is_empty() => String::new(),
        None => ".".into(),
        Some(i) => match path[..i].trim_end_matches('/') {
            "" => "/".into(),
            dir => dir.into(),
        },
    }
}

fn join(a: &str, b: &str) -> String {
    if b.starts_with('/') || a.is_empty() {
        b.into()
    } else if b.is_empty() {
        a.into()
    } else if a.ends_with('/') {
        format!("{a}{b}")
    } else {
        format!("{a}/{b}")
    }
}

fn normalize(path: &str) -> String {
    let rest = path.trim_start_matches('/');
    let parts: Vec<&str> = rest
        .split('/')
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();
    let out = format!(
        "{}{}",
        if path.starts_with('/') { "/" } else { "" },
        parts.join("/")
    );
    if out.is_empty() {
        return ".".into();
    }
    let is_dir =
        rest.ends_with('/') || (rest.ends_with('.') && rest[..rest.len() - 1].ends_with('/'));
    if is_dir && !out.ends_with('/') {
        out + "/"
    } else {
        out
    }
}

/// `text` read by pandoc as `format`, with the conversion's reader
/// options: the host's pandoc, built for wasm.
#[cfg(target_family = "wasm")]
fn read(text: &str, format: &str, conversion: &Conversion) -> Result<Pandoc, String> {
    let mut options = libpandoc::read_options(conversion);
    options["from"] = format.into();
    let mut docs = libpandoc::read_many(&[text], &options)
        .and_then(libpandoc::ReadMany::all)
        .map_err(|e| e.to_string())?;
    Ok(docs.remove(0))
}

/// Natively, a JSON filter has no pandoc to call.
#[cfg(not(target_family = "wasm"))]
fn read(_: &str, _: &str, _: &Conversion) -> Result<Pandoc, String> {
    Err(
        "include-files needs pandoc's reader, which a Rust filter gets from the host running it: \
         build it for wasm32-wasip1 and run it in pandocrs, pandocpy, pandocjl or libpandoc.wasm"
            .into(),
    )
}

#[derive(Default)]
struct State {
    /// `include-auto`.
    auto: bool,
    /// The level of the last heading met.
    last_level: i64,
}

struct Configure<'a>(&'a mut State);

impl Filter for Configure<'_> {
    fn meta(&mut self, meta: &mut Meta, _: &mut Ctx) -> Option<()> {
        if truthy(meta.get("include-auto")) {
            self.0.auto = true;
        }
        None
    }
}

/// Headings shifted; relative paths taken to the including file's
/// directory.
struct Update {
    shift_by: Option<f64>,
    include_path: String,
}

impl Filter for Update {
    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        match x {
            Block::Header(h) => {
                let level = h.level as f64 + self.shift_by?;
                if level.fract() != 0.0 {
                    // (Lua's error, as pandoc's Header takes an integer)
                    return ctx.fail(format!(
                        "shift-heading-level-by: {}: number has no integer representation",
                        self.shift_by?
                    ));
                }
                h.level = level as i64;
            }
            Block::CodeBlock(cb) => {
                if let Some(path) = cb.attr.get("include").filter(|p| is_relative(p)) {
                    let path = normalize(&join(&self.include_path, path));
                    cb.attr.set("include", path);
                }
            }
            _ => {}
        }
        None
    }

    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        match x {
            Inline::Link(link) => {
                let url = &link.target.url;
                if is_relative(url) && !filename(url).starts_with('#') && !has_scheme(url) {
                    link.target.url = normalize(&join(&self.include_path, url)).into();
                }
            }
            Inline::Image(image) if is_relative(&image.target.url) => {
                image.target.url = normalize(&join(&self.include_path, &image.target.url)).into();
            }
            _ => {}
        }
        None
    }
}

/// Lua's `string.find(target, "https?://")`.
fn has_scheme(url: &str) -> bool {
    url.contains("http://") || url.contains("https://")
}

/// The filter including files, for blocks of a file in `base` (the
/// original changes the working directory to it: here the names are joined
/// to it, as the working directory is the process's).
struct Transclude<'a> {
    base: String,
    state: &'a mut State,
}

impl Filter for Transclude<'_> {
    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        let cb = match x {
            Block::Header(h) => {
                self.state.last_level = h.level;
                return None;
            }
            Block::CodeBlock(cb) if cb.attr.classes.iter().any(|c| c == "include") => cb,
            _ => return None,
        };
        let format = cb.attr.get("format").unwrap_or("markdown").to_string();
        let shift_by = match cb.attr.get("shift-heading-level-by") {
            Some(text) => lua_number(text),
            None if self.state.auto => Some(self.state.last_level as f64),
            None => Some(0.0),
        };
        let before = self.state.last_level;
        let mut blocks = Vec::new();
        for line in cb.text.split('\n') {
            if line.is_empty() || line.starts_with("//") {
                continue;
            }
            let path = join(&self.base, line);
            // (Lua opens a directory, and pandoc fails to read nothing; what
            // reading one does here depends on the system and the wasm host)
            // ("" is the working directory in WASI, a missing file elsewhere)
            if !path.is_empty() && std::path::Path::new(&path).is_dir() {
                return ctx.fail(format!("{line}: a directory"));
            }
            let text = match std::fs::read(&path) {
                Ok(bytes) => ctx.ok(String::from_utf8(bytes))?,
                Err(_) => {
                    eprintln!("Cannot open file {line} | Skipping includes");
                    continue;
                }
            };
            let mut contents = ctx.ok(read(&text, &format, ctx.conversion()))?.blocks;
            self.state.last_level = 0;
            let mut nested = Transclude {
                base: join(&self.base, &directory(line)),
                state: self.state,
            };
            ctx.apply(&mut contents, &mut nested)?;
            self.state.last_level = before;
            let mut update = Update {
                shift_by,
                include_path: directory(line),
            };
            ctx.apply(&mut contents, &mut update)?;
            blocks.append(&mut contents);
        }
        Some(blocks)
    }
}

fn main() {
    panir::filter_with(|doc, conversion| {
        let mut state = State::default();
        panir::apply_with(doc, &mut Configure(&mut state), conversion)?;
        let mut transclude = Transclude {
            base: String::new(),
            state: &mut state,
        };
        panir::apply_with(doc, &mut transclude, conversion)
    });
}
