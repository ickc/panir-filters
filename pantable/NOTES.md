# pantable

ickc/pantable's `pantable` filter (0.15.0, on panflute 2.3.1): a code
block of class `table` holds CSV, with options in YAML blocks before it
(between `---` and `---` or `...`), and becomes a pandoc `Table`. The
options: `caption` and `short-caption`, `alignment` (a letter a column),
`alignment-cells` (a line of letters a row), `width` (a number or a
fraction a column, or `D`), `table-width` (the columns without a width get
one from the length of their text), `ms` (the rows of the head, of each
body's head and rows, of the foot), `ns-head` (each body's row-head
columns), `markdown` (cells, captions and attributes read as markdown,
**a call into pandoc**), `fancy-table` (with markdown: the first column
marks the head, bodies and foot, with attributes; a cell's first line its
span and attributes), `include` (the CSV from a file, **round 2**:
relative to the working directory, `include-encoding` its encoding),
`csv-kwargs` (Python's `csv.reader`'s arguments), `format` (`csv`, the
only one), `header` (read, never used: below). The owner's own filter;
its licence BSD-3-Clause (`LICENSE`, its repository's; panflute's is
BSD-3-Clause too). The original is copied whole (`upstream/pantable/`,
the package; the filter is `cli/pantable.py`), and run as it is installed
from PyPI (`oracle-requirements.txt`).

**Ported**: `pantable` (CSV to `Table`), every option, in the four
languages. **Left out**: `pantable2csv` and `pantable2csvx` (a `Table` to
a CSV code block): a conversion tool more than a filter (run once on a
document, not in each build), and it would need again what the reading
direction needed and more: cells written back as markdown (the original's
joined-document trick in reverse), PyYAML's dumper's exact layout, and
Python's `csv` writer's quoting, which the owner ruled out (below). And
`csv-kwargs` in TypeScript, Rust and Julia (below); numpy, `tabulate`,
`map_parallel` (the original's optional speed-ups) have no part in what
it makes.

The cases (80, in `html`): the original's own tests (`tests/golden`, as
`golden-NAME`, with their CSV files under `cases/files/tests/`), its
documentation's examples (`example-NAME`, with `fruits.csv`) and README
(`readme`, with `comparison.csv`), and this repository's: each option
(`caption`, `alignment`, `alignment-cells`, `width`, `table-width`,
`header`, `ms`, `attributes`, `format`), wrong values (`caption-odd`,
`width-zero`, `width-inf`, `table-width-inf`, `format-unknown`), YAML
(`yaml-blocks`: where the YAML is; `yaml-values`: YAML 1.1's numbers and
booleans; `yaml-list`, `yaml-scalar`, `yaml-tag`, `yaml-scanner-error`,
`yaml-parser-error`), CSV (`csv-quoting`, `csv-unterminated`, `ragged`,
`empty`, and `csv-kwargs*`), `include` (`include`, `include-encoding`,
`include-invalid-utf8`, `include-unknown-encoding`, `include-directory`,
and `outside`, with canaries in `cases/outside`), text widths
(`unicode-width`: East Asian wide characters, characters outside the BMP,
combining marks, Python's line breaks), and markdown cells (`md-*`:
inlines, blocks, notes and links, attributes and spans, fancy tables,
widths, tabs, failures), big tables (`big-text`: 300 rows by 8; `big-markdown`: 120 by 6).
`NAME.needs` marks the 24 cases whose ports call pandoc (markdown):
`run.py test` leaves them out, `run.py host` runs them.

## Markdown cells: one call a table (round 3)

**The original** reads every text of a table that is markdown (the
caption, the short caption, each cell, each cell's, row's and block's
attributes written as `[]{...}`) with **one `pandoc` process a table**: it
wraps each text in a div (`::: PanTableDiv :::` ... `:::`), joins them
into one document, runs `pandoc -f markdown -t json` on it
(`panflute.convert_text`), and takes the divs' contents back in order
(`util.iter_convert_texts_markdown_to_panflute`; "the bottle neck is
calling pandoc so we batch them and call it once only", its comment). It
has consequences, kept as the original's (`known.tsv`) where the ports
can't share them:

- the texts are one document: a note or a reference link defined in one
  cell resolves in another (`md-notes`), and headings' identifiers are
  made unique across the table's cells (`md-blocks`: `a-heading-1`), but
  not against the document's;
- a cell holding a line `:::` closes its div early, and the original
  fails (`md-break`);
- a cell that fails to read fails the whole table, with an empty message
  (`md-yaml-cell`: panflute's `OSError('')`);
- the `pandoc` command expands tabs (to stops of four), and reads with its
  defaults, not the document's reader options.

**The ports** read each text on its own, **all of a table's texts in one
call** (`read_many`), as markdown: Python `libpandoc.read_many(texts,
ctx.conversion, {"from": "markdown", "preserve-tabs": False})`, TypeScript
the instance it runs in (`pandoc.readMany(texts, conversion, {from:
"markdown", "preserve-tabs": false})`, through its `hosted(pandoc)` for
panir's `libpandocFilter`), Rust `libpandoc::read_many` with
`read_options(conversion, Some("markdown"))` and `preserve-tabs` false
(built for wasm: calls to the host), Julia `LibPandoc.read_many(texts,
ctx.conversion; from = "markdown", preserve_tabs = false)`. The texts are
the original's (the same list, the same `[]{...}` for attributes), empty
ones not sent. Tabs are expanded, as the original's `pandoc` command does
(`md-tabs`; a read with a conversion keeps them, as Lua's `pandoc.read`).
The reader options are the conversion's, where the original's command
has its defaults: no case differs (the cases' conversions have the
defaults), and a document read with `--tab-stop=8` would have its cells
read so too, as the brief asked (`ctx.conversion`). Standalone (a JSON
filter under plain pandoc), the same honest choice as include-files':
Python with libpandoc as a library if installed, else a table with
markdown fails saying so; TypeScript the `pandoc` command, a process a
text; Rust has no pandoc natively and fails at the first table with
markdown, saying so (`run.py host` doesn't run those cases in
`pandocrs-native`); Julia LibPandoc if in the environment, else a table with markdown fails
saying so.

Batching, measured (the first port here that batches; `RESULTS.md`,
`HOSTS.md` H15): a table of 1,001 rows by 10 columns of markdown cells
(links, emphasis, code, math, superscripts; a caption; `table-width`:
10,011 texts), warm in each host, the conversion from JSON to HTML repeated,
best of 3 to 5; a call a cell is the same port with its `read_markdown`
reading one text a call:

| | batched (`read_many`): the conversion; the reads | a call a cell: the conversion; the reads |
|---|---|---|
| Python, `libpandoc.convert` | 0.44 s; 0.20 s | 3.72 s; 3.50 s |
| Julia, `LibPandoc.convert` | 0.36 s; 0.18 s | 0.75 s; 0.58 s |
| TypeScript, libpandoc.wasm in Node | 3.29 s; 2.73 s | 3.56 s; 3.03 s |
| Rust as wasm, a process: pandocrs, pandocpy, pandocjl | 0.40, 0.42, 0.71 s; 0.20, 0.17, 0.22 s | 0.82, 1.21, 1.26 s; 0.58, 0.92, 0.73 s |
| the original, a process (pandoc, panflute, its one `pandoc`) | 1.13 s (its `pandoc` reading the joined document: 0.51 s) | – |

Batching is what makes the native hosts cheap: 17 times fewer seconds
reading in Python (0.35 ms a call against 0.02 ms a text batched:
`HOSTS.md` H15), 3 to 6 times in the wasm hosts, 3 times in Julia (whose
single call is cheaper than Python's); in libpandoc.wasm, one thread,
10% only (its reading is the cost). Batched, every native host reads the
10,011 texts in under half the time of the original's one `pandoc`
process, and a warm host converts the whole document in about a third of
the original's 1.13 s (Python 0.44, Julia 0.36).

## The owner's decision: CSV is each language's own (2026-11-17)

The owner (pantable's maintainer) ruled that the ports don't replicate
Python's `csv` module: `csv-kwargs` passes keyword arguments straight to
`csv.reader`, coupling the option to that module's interface, and
matching it elsewhere would mean reimplementing it. So `csv-kwargs` is
kept in the Python port only (one line, the same module); in TypeScript,
Rust and Julia a table that sets it gets a warning (`csv-kwargs ... is
not supported in this port`) and is read as plain CSV. Each port reads
standard CSV with its language's library at its defaults (but for rows of
different lengths, which pantable pads, and the csv crate's first row,
which is no header), and no code matches Python on edge input. What
differs (`known.tsv`, "owner's decision"; not gaps): the `csv-kwargs*`
cases and the documentation's include example (which sets a delimiter)
in the three; csv-parse refuses a quote inside an unquoted field (`a"b`)
and a quote never closed; the csv crate skips blank lines and drops a
UTF-8 byte-order mark; DelimitedFiles refuses text after a closing quote
(`"c"d`) and a quote never closed, and skips blank lines.

The libraries: Python's `csv`; TypeScript csv-parse 7 (`relax_column_count`);
Rust's csv crate 1.4 (`has_headers(false)`, `flexible(true)`); Julia's
DelimitedFiles 1.9 (`readdlm(io, ',', String)`), the standard library's
(CSV.jl is the well-known one, larger; the standard library first, as
the brief asks).

## YAML

PyYAML reads YAML 1.1 (`yes`, `on` are booleans, `010` is 8, `1:30` is
90, `1_000` is 1000, `1e3` is text: a 1.1 float wants a dot); with its
safe loader (libyaml's where built, as panflute), which refuses Python's
tags (`yaml-tag`). panflute's `yaml_filter` (strict) leaves the block as
it is on a scanner error, and fails on a parser error. The libraries:

| | library | YAML | what differs from PyYAML (`known.tsv`) |
|---|---|---|---|
| TypeScript | yaml 2.9, `version: "1.1"`, `intAsBigInt` (an integer told from a float, for Python's `str`), no timestamps (dates stay text, as Python's `str` of a date), duplicate keys allowed | 1.1 | `n` and `y` are booleans (the 1.1 spec's; PyYAML's resolver has only yes/no/true/false/on/off), `1e3` a float; its errors are not split into scanner and parser errors: the port leaves the block as it is on any |
| Rust | yaml-rust2 0.13 (pure safe Rust) | 1.2 | `yes`, `on`, `Off` are text (`md-switch`: `markdown: no` is the text `no`, true); `010` is 10; `1:30`, `1_000` text; `1e3` a float; a key given twice is an error; unknown tags are taken; errors not split |
| Julia | YAML.jl 0.4 (`dicttype = OrderedDict`, so the later of `short-caption` and `short_caption` wins, as in Python) | 1.1, but booleans | `yes`, `on`, `Off` text; `1:30`, `1_000` text; `1e3` a float; `caption: a: b` a parser error (PyYAML: a scanner error, the block left as it is) |

Each port then makes each option the type the original wants as Python
makes it (`str(1.5)` is `1.5`, `str(True)` `True`, a list
`['one', 2, True]`, `list("12")` `['1', '2']`, `bool("false")` true,
`float(Fraction("1/3"))`), Python's `str.splitlines` (its eight line
breaks), `str.upper` (`ß` is `SS`; Julia's `uppercase` keeps `ß`, known)
and the original's arithmetic on alignment letters, so that the original's
odd results come out the same (below).

## The original's behaviour, kept

- **Widths** count code points (Python's `len`), not display width:
  `語` is one, `😀` one (two UTF-16 units in TypeScript: the port counts
  code points), a family emoji five. pandoc's own widths count East Asian
  wide characters twice; pantable's don't (`unicode-width`).
- **A plain table's caption** is `Caption [Str ""]` with an empty
  paragraph (below).
- **A covered cell** of a span holds the span's text, and a later span
  overwrites an earlier one; spans are read only with `markdown`.
- **`ms`** checked (four or more, even, none negative, adding up to the
  rows) else dropped; **`ns-head`** checked against `ms`'s bodies, not the
  fancy table's.
- **Errors**: a missing include leaves the block as it is (a warning); an
  empty table is deleted; anything else fails as the original does, with
  the language's message (an unknown encoding, a directory, invalid
  UTF-8, `width: 1/0`, an infinite width, YAML that is a word).

One bug of the ports found while checking them, fixed: the original
sizes a markdown table by `ms` (`PanTableAbstract.m` is `ms.sum()`), so
with `fancy-table` the rows after the last marker are dropped, and their
text doesn't count in the widths; the four ports put them in the foot
(`md-fancy-tail`, added).

## For the owner: bugs and oddities in pantable

Each seen running 0.15.0 under pandoc 3.14 (the case named), most also in
its source:

1. **`header` does nothing.** `PanTableOption.header` is read and typed,
   and never used when the table is built: `header: false` still makes
   the first row the head (`header`; the project's own
   `tests/golden/pantable/full_test.md` sets `header: False` and its
   expected output has a head). `simplify` sets it when writing back.
2. **Every plain table gets an empty caption**: `Caption (Just [Str ""])
   [Para [Str ""]]`, which the HTML writer prints as
   `<caption><p></p></caption>` (pandoc's own tables: `Caption Nothing
   []`, no caption element). `PanTableText.to_pantable` wraps the caption
   string in a `Para` even when it is empty, and the short caption too.
3. **Rows after the last fancy-table marker are dropped silently**, and
   left out of the widths (`md-fancy-tail`): the table is sized by the
   markers' `ms`, which nothing checks against the rows.
4. **`ns-head` is checked against the options' `ms`, not the fancy
   table's bodies**: one number passes with two bodies, and only one body
   is made, the second's rows lost (`md-fancy-ns-head`); `ns-head: [0]`
   with a fancy head and foot and no body fails (`IndexError`).
5. **A data line starting with `---` starts a YAML block** (panflute's
   `yaml_filter`): a fancy table whose first column's first cell is `---`
   (a body head marker), after an options block closed by `...`, loses
   all its data and the table is deleted as empty (`md-fancy-dashes`).
6. **The joined document** (above): notes and reference links resolve
   across cells, identifiers are unique across a table but not the
   document, a cell with a `:::` line fails the table with a panflute
   type error (`md-break`), a cell that pandoc can't read fails it with
   an empty `OSError` (`md-yaml-cell`).
7. **Unchecked numbers fail with Python's errors**: `width: [1/0]`
   (`ZeroDivisionError`, `width-zero`), `width: [.inf]` (`OverflowError`,
   `width-inf`), and a column of spanning cells only with `table-width`
   (`max() iterable argument is empty`, `md-width-span`): `normalize`
   catches only `ValueError` and `TypeError`, `auto_width` assumes a
   column has a cell of its own.
8. **`table-width: .inf` writes JSON pandoc can't read**: panflute writes
   the width as `Infinity` (Python's `json`, `allow_nan`), and pandoc
   fails (`table-width-inf`; panflute's, not pantable's; panir writes
   `"+inf"`, which pandoc reads). Perhaps refuse a width that isn't
   finite.
9. **A short caption that isn't one paragraph** (`short-caption: "- a
   list"` with markdown) fails with panflute's type error (`Caption
   received ListItem`, `md-short-caption`): only the first block's
   content is taken.
10. **YAML that isn't a map or a list of pairs** fails with Python's
    `dict.update` error (`yaml-scalar`), where malformed YAML is left as it
    is (`yaml-scanner-error`): two kinds of bad options, two outcomes; and
    a YAML parser error (`yaml-parser-error`) fails where a scanner error
    leaves the block (panflute catches only `ScannerError`).
11. **Options of the wrong type are converted as Python converts them**,
    with odd results: `markdown: "false"` turns markdown on
    (`bool("false")`, `md-switch`), a list caption is its Python repr
    (`['one', 2, True]`, `caption-odd`), `caption: yes` is `True` (YAML
    1.1), `width: "12"` is the widths 1 and 2 (`list("12")`).
12. **Alignment letters**: any character is taken by arithmetic on its
    code (`(ord(c) - 3) % 11 % 6 % 4`), so a digit or an unknown letter
    silently aligns its column (`alignment: 12` is right, centre); one
    non-ASCII letter makes the whole row default with an error, though
    `ß` upper-cases to `SS` and shifts the columns after it (`alignment`).
13. **The error for an invalid cell shape** names the wrong shape: the
    `except ValueError` in `parse_data_markdown` logs `shape`, the
    previous cell's (or the table's), not the text that failed (source).
14. **`csv-kwargs` couples the option to Python's `csv.reader`**: any
    keyword it takes, `quoting: 2` (numbers) then failing in the table
    (`csv-kwargs-nonnumeric`); the reason the ports don't follow it.

## By language

| | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|
| CSV | `csv.reader(**csv_kwargs)` | csv-parse | csv | DelimitedFiles |
| YAML | PyYAML (`CSafeLoader`) | yaml (1.1 mode) | yaml-rust2 (1.2) | YAML.jl |
| encodings | Python's codecs | `TextDecoder` (WHATWG's names; Latin-1 Node's own) | encoding_rs (WHATWG's; Latin-1 by hand) | StringEncodings (iconv) |
| markdown | `libpandoc.read_many` | the instance's `readMany`; `pandoc` command standalone | `libpandoc::read_many` (wasm) | `LibPandoc.read_many` |
| lines (code) | 496 (397) | 647 (545) | 1,219 (1,060) | 596 (505) |

The original: 2,405 lines of Python (1,929 without blank and comment
lines; the package, both directions). Each port has Python's conversions
of values (`str`, `bool`, `list`, `Fraction`) and `splitlines`, the
original's span grid, and the original's row-block arithmetic; Rust twice the others (Python's `str` of YAML values, PyErr
kinds, rustfmt's layout).

- **Python**: closest to the original (its `csv`, PyYAML, `Fraction`).
- **TypeScript**: Python's `repr` of floats (shortest digits, exponent
  from 1e16), of strings (its quote choice) by hand; `[...s].length` for
  code points.
- **Rust**: `Yaml` values matched by hand into Python's types; libpandoc-rs
  a dependency for wasm only, so the native build needs no libpandoc.
- **Julia**: `uppercase` (simple case mapping, known); a regular
  expression of Python's line breaks built from the characters (PCRE took
  `\x{2028}` in a class otherwise).

Copies: none (`CHECK_UNIQUE=1` in `run.py test`).

## In the hosts

`run.py host` (`CHECK_UNIQUE=1`; the hosts built from the bindings'
`main`s after they adopted libpandoc 1.13, `HOSTS.md`), 696 case runs:
554 the original's, 142 known, none failing. Each mode's known ones:

| mode | runs | known |
|---|---|---|
| `pandocpy`, `python-api` | 80 each | 4 each: the joined document (`md-blocks`, `md-notes`, `md-break`), panflute's `Infinity` |
| `node` | 80 | 15: the TypeScript port's standalone ones (YAML, CSV, `csv-kwargs`) and the joined document's three |
| `pandocrs-wasm`, `pandocpy-wasm`, `pandocjl-wasm`, `node-wasm` | 80 each | 21 each: the Rust port's standalone ones, the joined document's three, `outside` (a wasm filter sees the working directory only; since libpandoc 1.14, given `../outside` too, it fails on the symbolic link), `md-switch` (YAML 1.2) |
| `pandocrs-native` | 56 (not the 24 with markdown: no pandoc to call) | 16: the Rust port's standalone ones |
| `pandocjl` | 80 | 19: the Julia port's standalone ones, the joined document's three, `md-switch` |

Tabs: a read with a conversion keeps them, the original's `pandoc`
command expands them; the ports ask for `"preserve-tabs": false` (every
binding takes it), and `md-tabs` is the original's everywhere.

The host gaps pantable found (`HOSTS.md`): H16, a Julia script's
packages come only from `pandocjl`'s load path (`run.py` stacks this
repository's project with `JULIA_LOAD_PATH`; now documented); H17,
`pandocjl` cut a Lua filter's writes to a pipe at 65,536 bytes (three
cases in its two modes, whose documents `capture.lua` writes; fixed in
LibPandoc.jl#8, and the six runs are the original's); H18, `read_many`'s
errors number a text by its place in the batch, from 0 in Python and
Rust, from 1 in Julia (now documented). And H15 measured with a batch
(above).

In the directory sandbox (`run.py host --sandbox`, only `cases/files`
listed, read-only), 474 case runs in six modes: 387 the original's, 87
known (the same rows as outside it), none failing. The canaries outside
the directory (`outside`, and `outside-absolute` by an absolute name): the
Python script and functions and the Julia script read all three (`..`,
the symbolic link, the absolute name), by design (`HOSTS.md`); the Rust
port as wasm none, failing (it sees the mounts only).

Since libpandoc 1.14 (2026-11-18, `HOSTS.md`) a wasm filter sees no
directory but those `LIBPANDOC_WASM_DIRS` gives, here the working
directory (`wasm-dirs: .` in the ORIGIN): given none, the 9 cases that
include a file and are otherwise the original's differ. `outside` gives
`../outside` too (`outside.env`): with the sandbox or without, the port
reads the file through `..` (and, in `outside-absolute`, the absolute
name) and fails on the symbolic link, refused as "Operation not
permitted" wherever it leads, as a file the original may not read fails
it. The counts above are unchanged.
