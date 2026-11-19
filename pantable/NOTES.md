# pantable

ickc/pantable's `pantable` filter (0.15.0, on panflute 2.3.1, with the
include attribute of pull request ickc/pantable#78 at `578353a`, not
merged): a code block of class `table` holds CSV, with options in a YAML
block (between `---` and `---` or `...`), and becomes a pandoc `Table`.
The options: `caption` and `short-caption`, `alignment` (a letter a
column), `alignment-cells` (a line of letters a row), `width` (a number or
a fraction a column, or `D`), `table-width` (the columns without a width
get one from the length of their text), `ms` (the rows of the head, of
each body's head and rows, of the foot), `ns-head` (each body's row-head
columns), `markdown` (cells, captions and attributes read as markdown,
**a call into pandoc**), `fancy-table` (with markdown: the first column
marks the head, bodies and foot, with attributes; a cell's first line its
span and attributes), `csv-kwargs` (Python's `csv.reader`'s arguments),
`format` (`csv`, the only one), `header` (read, never used: below), and
the file to read the CSV from: the `include` attribute
(`{.table include="file.csv"}`, appended to the block's text, as
[panclude](https://github.com/ickc/panclude) does) or the older `include`
key (`include-encoding` its encoding). The owner's own filter; its licence
BSD-3-Clause (`LICENSE`, its repository's; panflute's is BSD-3-Clause
too). The original is copied whole (`upstream/pantable/`, the package;
the filter is `cli/pantable.py`), and run as it is installed from that
commit (`oracle-requirements.txt`).

**Ported**: `pantable` (CSV to `Table`), every option, in the four
languages, **after panclude**: the ports read no file (below,
"Includes"), and their options are the block's YAML header **as pandoc
reads it**, over the document's metadata under `pantable` (below,
"Options"). **Left out**: `pantable2csv` and `pantable2csvx` (a `Table` to
a CSV code block): a conversion tool more than a filter (run once on a
document, not in each build), and it would need again what the reading
direction needed and more: cells written back as markdown (the original's
joined-document trick in reverse), PyYAML's dumper's exact layout, and
Python's `csv` writer's quoting, which the owner ruled out (below). And
`csv-kwargs` in TypeScript, Rust and Julia (below), the `include` key and
`include-encoding` (below); numpy, `tabulate`, `map_parallel` (the
original's optional speed-ups) have no part in what it makes.

The cases (95, in `html`): the original's own tests at `578353a`
(`tests/golden`, as `golden-NAME`, but the include key's three:
`include_key`, `include_key_invalid_path`, `encoding`; their files in
`cases/files`, which they run in, as upstream runs each case from its own
directory), its documentation's examples (`example-NAME`, with
`fruits.csv`) and README (`readme`, with `comparison.csv`), and this
repository's: each option (`caption`, `alignment`, `alignment-cells`,
`width`, `table-width`, `header`, `ms`, `attributes`, `format`), wrong
values (`caption-odd`, `caption-markup`, `width-odd`, `width-zero`,
`width-inf`, `table-width-inf`, `ms-text`, `format-unknown`), YAML
(`yaml-blocks`: what starts a header; `yaml-blocks-later`: YAML not
starting the block; `yaml-keys-both`, `yaml-duplicate`; `yaml-values`:
YAML's numbers and booleans; `yaml-list`, `yaml-scalar`, `yaml-tag`,
`yaml-scanner-error`, `yaml-parser-error`), the document's defaults
(`defaults`), CSV (`csv-quoting`, `csv-unterminated`, `ragged`, `empty`,
and `csv-kwargs*`, `csv-kwargs-tab*` for a tab), the include attribute
(`include`, `include-encoding`, `include-invalid-utf8`,
`include-directory`, and `outside`, with canaries in `cases/outside`),
text widths (`unicode-width`: East Asian wide characters, characters
outside the BMP, combining marks, Python's line breaks), and markdown
cells (`md-*`: inlines, blocks, notes and links, attributes and spans,
fancy tables, widths, tabs, failures), big tables (`big-text`: 300 rows by
8; `big-markdown`: 120 by 6). `NAME.needs` marks the 24 cases whose ports
read markdown cells (in a host one call a table; standalone a `pandoc`
process a text): `run.py test` leaves them out, `run.py host` runs them.

## Includes: panclude's (2026-11-19)

The owner's decision: including a file is panclude's job. The ports read
no file: a block names its file with the `include` attribute, and
panclude, run before the port, appends the file's text to the block's
(an empty block's text becomes the file's; else its own text, `\n`, the
file's), as it does for any code block: `pandoc -L panclude.lua` for stock
pandoc, or libpandoc's own step (libpandoc 1.16: `-F panclude`,
`"panclude"` among a conversion's filters), which reads only inside its
roots (by default the main document's directory). The port then sees an
ordinary block: header, own rows, the file's rows. The original at
`578353a` does the same itself (`include_attribute`, called before
`yaml_filter`; tested against `panclude.lua`), so the two give the same
table, but where a relative path starts: panclude at the document's
directory, the original at the directory pandoc runs in (the cases run
from their own directory, and the document is given on standard input, so
the two are one).

- **A block that still carries `include`** when the port sees it (panclude
  did not run, or could not read the file and left the block as it was):
  left exactly as it is, with one warning, "run panclude first". The
  original at `578353a` leaves a block it can't include as it is too,
  attribute kept (`include`: not found, `include-directory`,
  `include-invalid-utf8`, `include-encoding`'s Latin-1 and UTF-16,
  `golden-include_directory`, `golden-include_not_utf8`,
  `golden-include_external_csv_invalid_path`: all the original's).
- **The `include` key** in a header is no longer read by the ports: a
  warning ("the include key is not read: name the file with the include
  attribute, and run panclude first"), then the table is made as if the
  key were not there (an empty block is deleted, with its own warning).
  A warning rather than silence: the original still reads the key, and
  a table made without its file would otherwise go unnoticed.
  `include-encoding` was the key's: not read, without a warning (the
  attribute's file is UTF-8, as panclude reads it). Known:
  `golden-include_both` (the key's file replaces the rows in the
  original), `example-include` (the documentation's key example).
- **Files outside the document's directory** (`outside`, and, in the
  sandbox run, `outside-absolute` by an absolute name): `panclude.lua`
  reads whatever pandoc's Lua can, so in `run.py test` the ports' tables
  hold the canaries' text, as the original's do; libpandoc's step reads
  only inside its roots, so in every host (`run.py host`, sandbox or not)
  the blocks are left as they are, with panclude's warning (`outside the
  roots`, `a symbolic link leads outside the roots`): known in the hosts
  (`*@host`), and `run.py` fails a host run of a filter after panclude
  whose output holds a canary's marker. The port reads nothing itself, so
  its wasm build is given no directory (no `wasm-dirs`) and imports no
  `path_open`.

What a user writes differently: `{.table include="sales.csv"}` instead
of `include: sales.csv` in the header, a path from the document's
directory, the file in UTF-8; and panclude run first.

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
call** (`read_many`), as markdown (the captions are the header's, as
pandoc read it: "Options", below): Python `libpandoc.read_many(texts,
ctx.conversion, {"from": "markdown", "preserve-tabs": False})`, TypeScript
the instance it runs in (`pandoc.readMany(texts, conversion, {from:
"markdown", "preserve-tabs": false})`, through its `hosted(pandoc)` for
panir's `libpandocFilter`), Rust `libpandoc::read_many` with
`read_options(conversion, Some("markdown"))` and `preserve-tabs` false
(built for wasm: calls to the host), Julia `LibPandoc.read_many(texts,
ctx.conversion; from = "markdown", preserve_tabs = false)`. The texts are
the original's (the same list, the same `[]{...}` for attributes, but the
captions), empty ones not sent. Tabs are expanded, as the original's `pandoc` command does
(`md-tabs`; a read with a conversion keeps them, as Lua's `pandoc.read`).
The reader options are the conversion's, where the original's command
has its defaults: no case differs (the cases' conversions have the
defaults), and a document read with `--tab-stop=8` would have its cells
read so too, as the brief asked (`ctx.conversion`). Standalone (a JSON
filter under plain pandoc), since the headers too are pandoc's to read
(2026-11-19), every port has one reader for both, which falls back to the
`pandoc` command, a process a text: Python with libpandoc as a library if
installed, else the command; TypeScript the command; Rust, built natively,
the command (built for wasm, the host); Julia LibPandoc if in the
environment, else the command. `run.py test` runs the cases without
markdown cells so (the headers: a process each), CI with pandoc's release
binary; the 24 with markdown are `run.py host`'s (a process a cell
would make `big-markdown` a minute a port), and `pandocrs-native` still
skips them.

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
and a quote never closed; the csv crate skips blank lines (and drops a
UTF-8 byte-order mark, which the include step now drops first);
DelimitedFiles refuses text after a closing quote
(`"c"d`) and a quote never closed, and skips blank lines.

The libraries: Python's `csv`; TypeScript csv-parse 7 (`relax_column_count`);
Rust's csv crate 1.4 (`has_headers(false)`, `flexible(true)`); Julia's
DelimitedFiles 1.9 (`readdlm(io, ',', String)`), the standard library's
(CSV.jl is the well-known one, larger; the standard library first, as
the brief asks).

## Options: the header as pandoc reads it (2026-11-19)

The owner's decision: no YAML library. panir splits the block
(`yaml_block`: a header only where one starts the block, by pandoc's rule
for a metadata block), pandoc reads each header as a document's metadata
block (all of a document's headers in one call, through panir's
`yaml_filter`, panflute's shape: the filter gives it the reader), and
`Options` puts the header over the document's metadata under `pantable`,
a key of the header replacing a default's whole value. PyYAML, the `yaml`
package, yaml-rust2 and YAML.jl are gone, and with them the rows where the
four libraries disagreed with PyYAML and each other: the four ports now
read the same values, and agree with each other on every case but where
their CSV libraries (and `csv-kwargs`) differ, and Julia's `uppercase`
(checked: each port run on each case, the markdown ones too, and the
outputs compared).

**The format the headers are read in: `markdown-smart`.** Over the cases
it gives the fewest differences: `-f markdown`'s smart typography turns a
value's `'` into `’` (`csv-kwargs`' `quotechar: "'"`, in `csv-kwargs` and
the documentation's `example-include`, then reads no quotes), `--` into
a dash and `...` into an ellipsis; no case has a caption whose smart
quotes would show. And it is defensible: options are values, which
smart typography alters; markdown's other extensions stay on, so a
markdown caption keeps its emphasis, maths, raw TeX and HTML. Not the
document's own reader: a header is YAML, which only the markdown readers
take, and the document's extensions (or a reader that is not markdown)
would change what a value is from one document to another. What
remains of markdown in a value: escapes, entities and raw HTML are read
(`caption-markup`); a value that is only a tab is lost; `\t` is raw TeX.

Each option's reading, the same in the four ports (keys as written or
with `_` for `-`, as the original's README spells `fancy_table` and
`ns_head`; given both, the hyphenated one: pandoc's metadata is a map,
with no order for the original's "the later wins"; a value of another
shape than the option's is its default, with a warning):

| option | the ports, from pandoc's reading | the original (PyYAML 1.1, then Python) |
|---|---|---|
| `caption`, `short-caption` | with `markdown`, what pandoc read (inlines; a caption of blocks, from a `\|` scalar or paragraphs, its blocks; a short caption of blocks none, with a warning); without, its text, one `Str`. A list, a map, a boolean: none | `str()` of the value, read as markdown with `markdown` |
| `alignment` | its text | `str()` |
| `alignment-cells` | its text's words, a row each: a `\|` scalar's lines arrive as one paragraph, its line breaks spaces | `str()`'s lines |
| `width` | a list; an item a number as Lua reads one (panir's `lua_number`: `0.25`, `3`, `1e-1`) or two with `/` (`1/3`); `D`, a negative or anything else the default; a zero denominator fails, as the original; not a list: the default | `list()` of the value, `float(Fraction(x))` each |
| `table-width` | a number or a fraction as `width`'s | `float(Fraction(x))`, a float as it is |
| `markdown`, `fancy-table` | a YAML boolean (`true`, `True`, `false`); anything else the default, false | `bool()`: `"false"` and `yes` are true |
| `ms`, `ns-head` | a list of whole numbers (`3`, `3.0`) | `list()`, `int()` each |
| `csv-kwargs` (Python) | a map: booleans as they are, text a whole number an integer (`quoting: 3`), `\t` a tab, other text as it is | the map as PyYAML made it |
| `format` | text; not `csv` fails, as the original | `str()` |
| `header` | not read: the original reads it and never uses it | read, unused |
| `include`, `include-encoding` | not read (above) | the key's file |

**A tab for `csv-kwargs`** (the Python port; the others don't take
`csv-kwargs`): YAML's `"\t"` is a tab, a value that is only a tab, which
pandoc reads as nothing (`csv-kwargs-tab`: the delimiter is empty and
Python's `csv` fails); the two characters `\t` (`'\t'`, or `"\\t"`)
arrive as raw TeX, `RawInline (Format "tex") "\\t"`, which `stringify`
(and panir's `to_data`) drop, so the port takes the raw text of a
value's inlines and reads `\t` as a tab (`csv-kwargs-tab-escaped`; the
original passes the two characters to `csv.reader`, which fails).

**The document's defaults**: the metadata's `pantable` map gives every
table its options, a block's header replacing a key's whole value
(`defaults`; the original has no such thing):

```yaml
---
pantable:
  table-width: 1
  markdown: true
---
```

**Headers pandoc reads otherwise than PyYAML** (known, `*`: the four
ports alike):

- YAML 1.2, the numbers as text: `010` is 10, `1:30` and `1_000` text,
  `1e3` 1000, `on` and `Off` text, `yes` text (`yaml-values`, the boolean
  `caption`s of `caption-odd`); `.inf` is `+inf`, no number
  (`width-inf`, `table-width-inf`: the original fails on both); null is
  empty text (`caption-odd`'s `None`);
- only a boolean is one (`md-switch`: `markdown: "false"`, `yes`);
- a header only starts the block: YAML after the data, data before it, a
  header opened by `-----`, several headers are data
  (`yaml-blocks-later`, `golden-testing_wrong_type`); and so a data line
  `---` opens nothing (`md-fancy-dashes`: the original's bug 5, below, gone);
- a header that is a list or a word is no metadata: the whole block is
  data (`yaml-list`, `yaml-scalar`: the original reads a list of pairs, and
  fails on a word); a tag is read as text (`yaml-tag`: the original
  refuses it);
- YAML pandoc can't read fails the read, and the port, as pandoc fails a
  document so (`yaml-scanner-error`, `golden-invalid_yaml`,
  `yaml-duplicate`: a key given twice; the original leaves a block it
  can't scan as it is, takes the later of two keys, and fails on a parser
  error, `yaml-parser-error`, where the ports fail too);
- markup in a plain table's caption is read (`caption-markup`); a list,
  map or boolean caption is none (`caption-odd`); a text `ns-head` is
  not a list (`ms-text`); a boolean width, one with underscores, and a
  text width are the default (`width-odd`).

What a user writes differently: booleans as `true` and `false`; `width`,
`ms` and `ns-head` as lists; one header, at the start of the block; for
`csv-kwargs`' tab, `\t` (the two characters); and YAML pandoc reads
(no key twice).

**One call for a document's headers.** `yaml_filter` reads every header
of the document in one call of the reader, before any table is made; a
table with markdown cells reads them in a call of its own, as before. In
a host that is one more call into pandoc a document (none without a
header). Measured on a table of 1,001 rows by 10 columns of CSV with a
header (`caption`, `table-width`, `alignment`), standalone (`run.py
time`, best of 5, the `pandoc` command being pandocrs): Python 0.17 s
before, 0.18 s now; Rust under 0.01 s, 0.01 s now; TypeScript 0.22 s;
the original 0.15 s. The header's `pandoc` process is the difference, not
visible beside the table.

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
- **Errors**: an include that failed leaves the block as it is (a
  warning); an empty table is deleted; anything else fails as the
  original does, with the language's message (`width: 1/0`, an unknown
  `format`), and YAML pandoc can't read fails ("Options", above).

One bug of the ports found while checking them, fixed: the original
sizes a markdown table by `ms` (`PanTableAbstract.m` is `ms.sum()`), so
with `fancy-table` the rows after the last marker are dropped, and their
text doesn't count in the widths; the four ports put them in the foot
(`md-fancy-tail`, added).

## For the owner: bugs and oddities in pantable

Each seen running 0.15.0 under pandoc 3.14 (the case named), most also in
its source, and all still so at `578353a`; where the ports now differ,
since they read the header as pandoc does, the item says so:

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
   The ports read a header only where one starts the block: the rows
   stay (known).
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
   column has a cell of its own. (`.inf` is the text `+inf` to pandoc, no
   number: the ports take the default, known.)
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
    leaves the block (panflute catches only `ScannerError`). The ports
    have pandoc's outcomes: a word or a list is no header, the block all
    data; YAML it can't read fails.
11. **Options of the wrong type are converted as Python converts them**,
    with odd results: `markdown: "false"` turns markdown on
    (`bool("false")`, `md-switch`), a list caption is its Python repr
    (`['one', 2, True]`, `caption-odd`), `caption: yes` is `True` (YAML
    1.1), `width: "12"` is the widths 1 and 2 (`list("12")`). The ports
    take a value of another shape as the default, with a warning.
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
| the header | panir's `yaml_filter`, pandoc's reading | `yamlFilter` | `yaml_filter` | `yaml_filter` |
| pandoc (headers, markdown) | `libpandoc.read_many`; the `pandoc` command without libpandoc | the instance's `readMany`; the `pandoc` command standalone | `libpandoc::read_many` (wasm); the `pandoc` command natively | `LibPandoc.read_many`; the `pandoc` command without LibPandoc |
| lines (code), 2026-11-19 | 510 (356) | 510 (383) | 929 (801) | 450 (365) |
| before (PyYAML, yaml, yaml-rust2, YAML.jl; the key's include) | 496 (362) | 647 (505) | 1,219 (1,059) | 596 (493) |

(Code: lines that are neither blank, comments nor docstrings; the
"before" figures counted again so, where the table had other counts.)
The original: 2,405 lines of Python (1,929 without blank and comment
lines; the package, both directions). Each port keeps Python's
`splitlines`, the original's span grid and row-block arithmetic; the
values' readings are the four ports' one rule ("Options"), on panir's
helpers: `Options.text`, `.boolean`, `.list`, `lua_number` for numbers,
`inlines_of` and `blocks_of` for a markdown caption (copies, so a
default caption used by two tables is two nodes). Gone: Python's
conversions of YAML values (`str`, `bool`, `list`, `Fraction`, `repr` of
floats) in TypeScript, Rust and Julia, the YAML libraries, the encodings
(`TextDecoder`, encoding_rs, StringEncodings), the file reading.

- **Python**: closest to the original (its `csv`); the Python port alone
  takes `csv-kwargs`, so it alone reads a value's raw TeX for `\t`.
- **TypeScript**: `[...s].length` for code points; one reader for headers
  and cells, given the format.
- **Rust**: libpandoc-rs a dependency for wasm only, so the native build
  needs no libpandoc; natively the `pandoc` command; `serde_json` for
  `Options::list`'s plain data.
- **Julia**: `uppercase` (simple case mapping, known); a regular
  expression of Python's line breaks built from the characters (PCRE took
  `\x{2028}` in a class otherwise).

Copies: none (`CHECK_UNIQUE=1` in `run.py test`).

## In the hosts

`run.py host` (`CHECK_UNIQUE=1`; the hosts built from the bindings'
checkouts at libpandoc 1.16, `HOSTS.md`, "Since: pantable after panclude";
the include step libpandoc's, `-F panclude`), 831 case runs: 525 the
original's, 306 known, none failing (before, at libpandoc 1.15 with the
include key and the YAML libraries: 696 runs, 554 and 142). Each mode's
known ones:

| mode | runs | known |
|---|---|---|
| `pandocpy`, `python-api` | 95 each | 28 each: the four ports' (`*`): 21 of the header as pandoc reads it, the include key and the defaults, `outside`, and the joined document's three; the include key's documentation example and the Python port's two tab cases |
| `node` | 95 | 34: the four ports' 25, the include key's example, `csv-kwargs` (six) and csv-parse's two |
| `pandocrs-wasm`, `pandocpy-wasm`, `pandocjl-wasm`, `node-wasm` | 95 each | 37 each: the four ports' 25, the include key's example, `csv-kwargs` (eight) and the csv crate's three |
| `pandocrs-native` | 71 (not the 24 with markdown cells, a `pandoc` process a text there) | 30: the wasm modes' but the seven with markdown cells |
| `pandocjl` | 95 | 38: the four ports' 25, the include key's example, `csv-kwargs` (seven), DelimitedFiles' four, `ß` |

More known than before, all for the reasons above: the cases written for
the readings (`caption-markup`, `width-odd`, `ms-text`,
`yaml-blocks-later`, `yaml-keys-both`, `yaml-duplicate`, `defaults`, the
tab cases), the YAML rows now the four ports' alike (where before each
library had its own), and `outside` in every mode (the include step's
roots); the rows of each YAML library, of the encodings, and of the wasm
filter's directories (`outside` refused on the symbolic link) are gone.

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
listed, read-only), 564 case runs in six modes: 365 the original's, 199
known (the same rows as outside it), none failing (before: 474, 387 and
87). The canaries outside the directory (`outside`, and
`outside-absolute` by an absolute name): read by none, in every mode, the
ports reading no file and the include step only inside its root (before:
the Python script and functions and the Julia script read all three, by
design, `HOSTS.md`; the Rust port as wasm none).
