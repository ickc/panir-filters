//! Code blocks of class `table`, CSV with a YAML header of options, made
//! tables: a pandoc filter.
//!
//! The block holds CSV, after a YAML header of options (between `---` and
//! `---` or `...`, starting the block): `caption`, `short-caption`,
//! `alignment` (a letter a column among L, R, C, D), `alignment-cells` (a
//! word of letters a row), `width` (a number or fraction a column, or D),
//! `table-width` (then the columns without a width get one from the length
//! of their text), `ms` (rows of the head, of each body's head and rows, of
//! the foot), `ns-head` (row head columns of each body), `markdown` (cells,
//! captions and attributes read as markdown), `fancy-table` (with markdown:
//! the first column marks the head, bodies and foot, with attributes; a
//! cell's first line its span and attributes), `format` (csv only). The
//! document's metadata under `pantable` gives defaults, a header's keys
//! replacing them. `csv-kwargs` (Python's csv.reader's arguments) is not
//! supported: a warning, then the CSV read as it is.
//!
//! The file a block's `include` attribute names is panclude's to include
//! (`pandoc -L panclude.lua`, or libpandoc's `panclude` step, run before
//! this filter): a block still carrying the attribute is left as it is.
//!
//! The header is read by pandoc, as a metadata block (`-f markdown-smart`;
//! panir's `yaml_filter`), the markdown of the cells too: built for wasm
//! (`cargo build --target wasm32-wasip1`), the host's pandoc, in one call
//! (`read_many`); built natively, as a JSON filter, the `pandoc` command, a
//! process a text. The CSV is the `csv` crate's.
//!
//! A port of ickc/pantable's `pantable` filter, 0.15.0 with the include
//! attribute (© 2016-2026 Kolen Cheung; BSD-3-Clause, see ../LICENSE).

use std::collections::HashMap;
use std::sync::LazyLock;

use panir::{
    Alignment, Attr, Block, Caption, Cell, CodeBlock, ColSpec, ColWidth, Conversion, Inline,
    MetaValue, Options, Pandoc, Row, Table, TableBody, TableFoot, TableHead, blocks_of, inlines_of,
    lua_number, to_data,
};
use regex::Regex;
use serde_json::Value;

fn warn(message: &str) {
    eprintln!("pantable: {message}");
}

/// Texts read by pandoc, a document each, as `format` says, the
/// conversion's way: the host's pandoc, built for wasm, all in one call;
/// tabs expanded, as the original's `pandoc` command does (a read with a
/// conversion keeps them).
#[cfg(target_family = "wasm")]
fn read(texts: &[String], conversion: &Conversion, format: &str) -> Result<Vec<Pandoc>, String> {
    let mut options = libpandoc::read_options(conversion, Some(format));
    options["preserve-tabs"] = false.into();
    libpandoc::read_many(texts, &options)
        .and_then(libpandoc::ReadMany::all)
        .map_err(|e| e.to_string())
}

/// Natively, as a JSON filter: the `pandoc` command, a process a text.
#[cfg(not(target_family = "wasm"))]
fn read(texts: &[String], _: &Conversion, format: &str) -> Result<Vec<Pandoc>, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};
    let run = |text: &String| -> Result<Pandoc, String> {
        let mut child = Command::new("pandoc")
            .args(["-f", format, "-t", "json"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("pantable: the pandoc command: {e}"))?;
        let written = child
            .stdin
            .take()
            .map(|mut stdin| stdin.write_all(text.as_bytes()));
        let out = child.wait_with_output().map_err(|e| e.to_string())?;
        if !out.status.success() {
            let message = String::from_utf8_lossy(&out.stderr);
            return Err(format!(
                "pantable: pandoc could not read a text: {}",
                message.trim()
            ));
        }
        written.transpose().map_err(|e| e.to_string())?;
        panir::from_slice(&out.stdout).map_err(|e| e.to_string())
    };
    texts.iter().map(run).collect()
}

// The options, from the metadata pandoc read: a key as written or with `_`
// for `-` (the hyphenated one if both); a value of another shape than the
// option's is its default, with a warning.

/// An option's value as `shape` reads it (`Options::text`, `boolean`,
/// `list`, or `raw`); None if missing or of another shape.
fn get<T>(options: &Options, key: &str, shape: impl Fn(&Options, &str) -> Option<T>) -> Option<T> {
    let underscored = key.replace('-', "_");
    let key = if options.contains_key(key) {
        key
    } else {
        underscored.as_str()
    };
    let raw = options.get(key)?;
    let value = shape(options, key);
    if value.is_none() {
        warn(&format!(
            "option {key} with value {} has an invalid type: the default",
            to_data(raw)
        ));
    }
    value
}

fn raw(options: &Options, key: &str) -> Option<MetaValue> {
    options.get(key).cloned()
}

/// A number as Lua reads one, or two with `/` between (`1/3`); None for
/// anything else. A zero denominator fails, as the original.
fn fraction(text: &str) -> Result<Option<f64>, String> {
    let (top, bottom) = text
        .split_once('/')
        .map_or((text, None), |(a, b)| (a, Some(b)));
    let (Some(a), Some(b)) = (lua_number(top), bottom.map_or(Some(1.0), lua_number)) else {
        return Ok(None);
    };
    if b == 0.0 {
        return Err(format!("division by zero: {text}"));
    }
    Ok(Some(a / b))
}

/// A whole number as Lua reads one (`3`, `3.0`).
fn whole(text: &str) -> Option<i64> {
    lua_number(text)
        .filter(|x| x.is_finite() && x.fract() == 0.0)
        .map(|x| x as i64)
}

/// Whole numbers (`ms`, `ns-head`), else the default.
fn integers(values: Option<Vec<Value>>, key: &str) -> Option<Vec<i64>> {
    let values = values?;
    let out: Option<Vec<i64>> = values.iter().map(|x| x.as_str().and_then(whole)).collect();
    if out.is_none() {
        warn(&format!(
            "option {key} with value {values:?} has an invalid type: the default"
        ));
    }
    out
}

/// Words of text: split at spaces, tabs and line breaks.
fn words(text: &str) -> Vec<String> {
    text.split([' ', '\t', '\n', '\r'])
        .filter(|w| !w.is_empty())
        .map(String::from)
        .collect()
}

struct Settings {
    caption: Option<MetaValue>,
    short_caption: Option<MetaValue>,
    alignment: String,
    alignment_cells: Vec<String>,
    width: Option<Vec<Option<f64>>>,
    table_width: Option<f64>,
    ms: Option<Vec<i64>>,
    ns_head: Option<Vec<i64>>,
    markdown: bool,
    fancy_table: bool,
    format: String,
}

impl Settings {
    fn of(options: &Options) -> Result<Self, String> {
        if options.contains_key("include") {
            warn(
                "the include key is not read: name the file with the include attribute, and run panclude first",
            );
        }
        if get(options, "csv-kwargs", raw).is_some() {
            warn("csv-kwargs (Python's csv.reader's) is not supported in this port: read as CSV");
        }
        let width = get(options, "width", Options::list)
            .map(|ws| {
                ws.iter()
                    .map(|w| w.as_str().map_or(Ok(None), fraction))
                    .collect()
            })
            .transpose()?;
        let table_width =
            get(options, "table-width", Options::text).map_or(Ok(None), |t| fraction(&t))?;
        Ok(Settings {
            caption: get(options, "caption", raw),
            short_caption: get(options, "short-caption", raw),
            alignment: get(options, "alignment", Options::text).unwrap_or_default(),
            alignment_cells: words(
                &get(options, "alignment-cells", Options::text).unwrap_or_default(),
            ),
            width,
            table_width,
            ms: integers(get(options, "ms", Options::list), "ms"),
            ns_head: integers(get(options, "ns-head", Options::list), "ns-head"),
            markdown: get(options, "markdown", Options::boolean).unwrap_or(false),
            fancy_table: get(options, "fancy-table", Options::boolean).unwrap_or(false),
            format: get(options, "format", Options::text)
                .filter(|f| !f.is_empty())
                .unwrap_or("csv".into()),
        })
    }

    /// What is wrong beyond the type, for a table of m rows and n columns:
    /// the widths, None for the default.
    fn normalize(&mut self, m: usize, n: usize) -> Vec<Option<f64>> {
        let mut total = 0.0;
        let mut widths = vec![None; n];
        for (i, w) in self.width.iter().flatten().take(n).enumerate() {
            if let Some(x) = w.filter(|&x| x >= 0.0) {
                widths[i] = Some(x);
                total += x;
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
        widths
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

/// The rows, as long as the longest.
fn load_csv(data: &str) -> Result<Vec<Vec<String>>, String> {
    // (the csv crate's defaults, but for a first row that is no header, and
    // rows of other lengths, which pantable pads)
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .flexible(true)
        .from_reader(data.as_bytes());
    let rows: Vec<Vec<String>> = reader
        .records()
        .map(|r| r.map(|r| r.iter().map(String::from).collect()))
        .collect::<Result<_, _>>()
        .map_err(|e| e.to_string())?;
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

/// The anchored cells' contents and attributes, the rows' attributes, the
/// blocks' attributes.
type Parts = (Vec<Vec<Block>>, Vec<Attr>, Vec<Attr>, Vec<Attr>);

fn text_or_none(key: &str, value: Option<MetaValue>) -> Option<MetaValue> {
    match value {
        None
        | Some(MetaValue::MetaString(_) | MetaValue::MetaInlines(_) | MetaValue::MetaBlocks(_)) => {
            value
        }
        Some(v) => {
            warn(&format!(
                "option {key} with value {} has an invalid type: the default",
                to_data(&v)
            ));
            None
        }
    }
}

/// The caption: with markdown, as pandoc read the header (the original
/// reads its text as markdown); else its text, one `Str`, as the
/// original's (a plain table's caption is an empty paragraph). Not text:
/// none, with a warning; a short caption of blocks too.
fn caption(
    long: Option<MetaValue>,
    short: Option<MetaValue>,
    markdown: bool,
) -> Result<Caption, String> {
    let long = text_or_none("caption", long);
    let mut short = text_or_none("short-caption", short);
    if !markdown {
        let text = |v: &Option<MetaValue>| {
            v.as_ref()
                .map(to_data)
                .and_then(|d| d.as_str().map(String::from))
        };
        return Ok(Caption {
            short: Some(vec![str_(&text(&short).unwrap_or_default())]),
            content: vec![Block::Para(vec![str_(&text(&long).unwrap_or_default())])],
        });
    }
    if matches!(short, Some(MetaValue::MetaBlocks(_))) {
        warn("a short-caption of blocks: none");
        short = None;
    }
    let content = match &long {
        Some(v @ MetaValue::MetaBlocks(_)) => blocks_of(v)?,
        Some(v) => Some(inlines_of(v)?)
            .filter(|x| !x.is_empty())
            .map(Block::Para)
            .into_iter()
            .collect(),
        None => Vec::new(),
    };
    let short = short
        .map(|v| inlines_of(&v))
        .transpose()?
        .filter(|x| !x.is_empty());
    Ok(Caption { short, content })
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

fn table(
    options: &Options,
    data: &str,
    block: &CodeBlock,
    conversion: &Conversion,
) -> Result<Option<Vec<Block>>, String> {
    if block
        .attr
        .attributes
        .iter()
        .any(|(k, _)| k.as_str() == "include")
    {
        warn("a code block with an include attribute: left as it is (run panclude first)");
        return Ok(None);
    }
    let mut settings = Settings::of(options)?;
    if settings.format != "csv" {
        return Err(format!("Unknown format: {}", settings.format));
    }
    let rows = load_csv(data.trim_matches('\n'))?; // (blank lines around the data dropped)
    if !rows.iter().flatten().any(|cell| !cell.trim().is_empty()) {
        warn("table is empty: deleted");
        return Ok(Some(Vec::new()));
    }
    let md = settings
        .markdown
        .then(|| markdown_cells(&rows, settings.fancy_table));
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
    let mut widths = settings.normalize(rows_in, n);
    let ms = md
        .as_ref()
        .and_then(|md| md.ms.clone())
        .or(settings.ms.clone())
        .unwrap_or(vec![1, 0, rows_in as i64 - 1, 0]);
    // the table has the rows of its blocks: with fancy-table, the rows after
    // the last marker are dropped (as the original, which sizes it by ms)
    let m = ms.iter().sum::<i64>() as usize;
    let ns_head = settings
        .ns_head
        .clone()
        .unwrap_or(vec![0; ms.len() / 2 - 1]);
    if let Some(tw) = settings.table_width {
        widths = auto_width(grid, m, &widths, tw)?;
    }
    let col_specs = aligns(&settings.alignment, n)
        .into_iter()
        .zip(&widths)
        .map(|(alignment, w)| ColSpec {
            alignment,
            width: w.map_or(ColWidth::ColWidthDefault, ColWidth::ColWidth),
        })
        .collect();
    let cell_aligns: Vec<Vec<Alignment>> = (0..m)
        .map(|i| {
            aligns(
                settings.alignment_cells.get(i).map_or("", String::as_str),
                n,
            )
        })
        .collect();

    let anchors: Vec<(usize, usize)> = (0..m)
        .flat_map(|i| (0..n).map(move |j| (i, j)))
        .filter(|&(i, j)| grid.is_at(i, j))
        .collect();
    let blocks = ms.len() / 2 + 1;
    let (mut contents, mut attrs, row_attrs, block_attrs): Parts;
    if let Some(md) = &md {
        // every text of the table in one call: cells and their attributes,
        // rows' and blocks' attributes
        let none = vec![String::new(); blocks];
        let texts: Vec<&str> = (anchors.iter().map(|&(i, j)| grid.text(i, j)))
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
        let todo: Vec<String> = texts
            .iter()
            .filter(|t| !t.is_empty())
            .map(|t| t.to_string())
            .collect();
        let mut done = if todo.is_empty() {
            Vec::new()
        } else {
            read(&todo, conversion, "markdown")?
        }
        .into_iter();
        let mut got = texts.iter().map(|t| {
            if t.is_empty() {
                Vec::new()
            } else {
                done.next().map(|doc| doc.blocks).unwrap_or_default()
            }
        });
        let k = anchors.len();
        contents = got.by_ref().take(k).map(plain).collect();
        attrs = got.by_ref().take(k).map(attr_of).collect();
        row_attrs = got.by_ref().take(m).map(attr_of).collect();
        block_attrs = got.map(attr_of).collect();
    } else {
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
        caption: caption(settings.caption, settings.short_caption, settings.markdown)?,
        col_specs,
        head,
        bodies,
        foot,
    }))]))
}

fn main() {
    panir::filter_with(|doc, conversion| {
        panir::yaml_filter(
            doc,
            &["table"],
            "pantable",
            |texts| read(texts, conversion, "markdown-smart"),
            |options, data, block| table(options, data, block, conversion),
        )
    });
}
