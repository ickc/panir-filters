# Results

Per filter and language: the cases (`run.py test`), the differential runs
against the original (`run.py diff`), lines of code, and time and peak
memory (`run.py time`). Measured 2026-11-14 on Linux (x86-64, 32 cores),
pandoc 3.14, Python 3.15, Node.js 26, Rust 1.97, Julia 1.12; panir at
`09c14e8` for the first three filters' times, at `d02476a` (panir#14's
branch) for the second batch's, at `2c097cc` (panir#15 merged) for the
third batch's; at `ec04fa5` (panir#16 merged), 2026-11-15, for the cases,
the differential runs and the lines of the fourth batch; at `54eb1a8`
(panir#17's branch), 2026-11-15, for the fifth batch, and for every case,
differential run and line since.

## Cases and differential runs

The differential runs: pandoc's test documents (`fetch-docs.sh`: 5) and
eight real documents (the pandoc manual and changelog; GitLab's GraphQL
reference, all tables; a Wikipedia article read from HTML; Python's
`stdtypes` from RST; a 774-page textbook from LaTeX; a Chinese page of
code; War and Peace from EPUB); and documents made of every filter's
cases (`assemble.py`), as the real ones hold little of most filters'
syntax: five from the second batch on, eight from the third, each with
metadata of its own (the cases' fields, some missing, some reshaped), as
the real ones hold little metadata: 21 documents, each in each output
format the filter's cases use. A case is a document in one format.

| filter | cases | Python | TypeScript | Rust | Julia | differential (each language) |
|---|---|---|---|---|---|---|
| caps | 3 | 3/3 | 3/3 | 3/3 | 2/3, 1 known | 21/21; Julia 19/21, 2 known |
| metavars | 4 (1 fails in the original) | 4/4 | 4/4 | 4/4 | 4/4 | 21/21 (8 fail in the original and every port: a real document, and seven made ones whose metadata lacks a field their text names) |
| list-table | 6 (4 fail in the original) | 6/6 | 6/6 | 6/6 | 6/6 | 21/21 |
| deemph | 5 | 5/5 | 5/5 | 5/5 | 4/5, 1 known | 21/21 |
| comments | 7 (1 fails in the original) | 6/7, 1 known | 6/7, 1 known | 6/7, 1 known (2 before panir#15) | 6/7, 1 known | 21/21 |
| deflists | 4 | 4/4 | 4/4 | 4/4 | 4/4 | 21/21 |
| theorem | 16 (4 formats) | 16/16 | 16/16 | 16/16 | 16/16 | 84/84 |
| inlinenotes | 10 (2 formats) | 9/10, 1 known | 9/10, 1 known | 9/10, 1 known | 9/10, 1 known | 41/42, 1 known |
| gitlab_markdown | 3 | 3/3 | 3/3 | 3/3 | 3/3 | 21/21 |
| french-spaces | 12 (3 formats) | 12/12 | 12/12 | 12/12 | 12/12 | 63/63 |
| scholarly-metadata | 14 (3 fail in the original) | 13/14, 1 known | 13/14, 1 known | 13/14, 1 known | 13/14, 1 known | 21/21 |
| author-info-blocks (after scholarly-metadata) | 12 (2 formats; 1 fails in the original) | 12/12 | 12/12 | 12/12 | 12/12 | 42/42 |
| pandoc-quotes (the original fixed for pandoc 3) | 20 (6 fail in the original) | 20/20 | 20/20 | 20/20 | 20/20 | 21/21 |
| abstract-section | 13 (2 fail in the original) | 13/13 | 13/13 | 13/13 | 13/13 | 21/21 |
| cito | 5 | 5/5 | 5/5 | 5/5 | 5/5 | 21/21 |
| pagebreak | 143 (13 documents, 11 formats; 22 fail in the original; 2 set the environment) | 143/143 | 143/143 | 143/143 | 143/143 | 231/231 |
| track-changes | 133 (19 documents, 7 formats; 4 fail in the original; 3 read through docx) | 133/133 | 133/133 | 133/133 | 133/133 | 147/147 |
| first-line-indent | 72 (24 documents, 3 formats; 8 fail in the original; the two documents added with the fifth batch are 6 of them, 2 failing) | 72/72 | 72/72 | 72/72 | 72/72 | 63/63 |
| count-para | 48 (16 documents, 3 formats; 12 fail in the original) | 48/48 | 48/48 | 48/48 | 48/48 | 63/63 |
| auto-lang | 21 (1 format; 2 fail in the original) | 21/21 | 21/21 | 21/21 | 21/21 | 84/84; and 40/40 on the real documents with an `auto-lang` map |
| amsthm | 292 (73 documents, 4 formats; 16 fail in the original; 27 documents with the writer's options) | 260/292, 32 known | 260/292, 32 known | 224/292, 68 known (36 the writer's options, which a JSON filter is not given: the same in process, `cargo test`) | 260/292, 32 known | 336/336; and 48/48 on the textbooks with an `amsthm` map |

The fourth batch's differential runs: 504 per language, all the same AST.
The originals fail on some made documents, and the ports with them:
pagebreak on one whose `pagebreak.break-on` is `true` (every format),
track-changes on three holding a change without an author (HTML and
LaTeX), first-line-indent on one whose configuration is `true`,
count-para on two whose `header-includes` is text (HTML, LaTeX). The real
documents hold no tracked changes, no options and no raw page breaks (the
pandoc manual and changelog name `\newpage` only in text and code), so
there the filters number paragraphs and mark indents with their defaults;
the made documents carry the cases' syntax and options.

The fifth batch's differential runs: auto-lang on the 21 documents (none
has an `auto-lang` map but the made ones, which carry its cases' maps),
and on five real documents with one (`assemble.py --lang`: every script
mapped, and every one but Latin): the Wikipedia article (spans of
Cyrillic, Arabic, Han, Devanagari, Myanmar, Bengali, Hebrew, Armenian),
War and Peace, the Wikipedia article's source, the Chinese page of code
(724 Han spans), the pandoc manual (3,260 Latin spans with Latin mapped);
amsthm on the 21 in four formats, and on three textbooks read from LaTeX
with an `amsthm` map declaring their divs' classes (`assemble.py
--amsthm`: the 774-page textbook's 738 theorems, definitions, exercises
and the like numbered, a book of 471 pages' 35, a textbook's 84). Every
port the same AST on every one; as JSON filters, without the writer's
options, which none of these documents is given.

The new made documents (with the fifth batch's cases) found one bug in an
earlier port: first-line-indent did not fail where its original fails
building `header-includes` it then drops (`first-line-indent/NOTES.md`),
fixed, with two cases. And Julia's `uppercase` meets `φῶς`, auto-lang's
Greek, in all eight made documents: `ῶ` is two characters in full upper
case (caps, known).

In all, the fifth batch's run, every filter on the 21 documents: 4,284
port runs, 4,270 the same, 14 known differences (the six before, and
Julia's `φῶς` in caps on the eight made documents); with the documents
with maps, 4,372, 4,358 the same. The run took 32 minutes (the Python
originals' filters again, with their oracle, 12).

In all, with the fourth batch, 3,864 port runs against the original (the
fifteen earlier filters' again on the new made documents): 3,858 the
same, 6 known differences, the same 6 as before; the whole run took 29
minutes. Before it, 1,848 port runs: 1,842 the same (failures of the
original and the port alike among them: author-info-blocks on every
document without authors but in LaTeX, metavars, scholarly-metadata and
pandoc-quotes on some made documents' metadata), 6 known differences
(inlinenotes on the textbook in every language, caps on two documents in
Julia). The whole run takes 14 minutes, mostly Julia's start.

Known differences (`known.tsv`, explained in each `NOTES.md`):

- Julia's `uppercase` maps `ß` to `ẞ` where the others give `SS`, and
  keeps `ῶ` one character (caps, deemph);
- comments: the original, on a beginning in a table cell, writes JSON
  pandoc can't read (every language); (before panir#15, Rust emptied a
  hidden `MetaList`'s items where the original deletes them: `GAPS.md`,
  14);
- scholarly-metadata: an author given as blocks, which the original names
  after Lua's `tostring` of them, where the ports fail (every language);
- inlinenotes: the original's query walks the note's JSON and repeats
  nested emphasis, or leaves a note whose emphasis or citation holds a
  link, code or raw inline (every language; a case, and the textbook);
- amsthm: notes with markup, which the original reads with pandoc's
  markdown reader (round 3), kept as text by the ports (every language,
  28 runs); pandoc's Lua making an empty list item `[Plain []]` once the
  original reads the list's `content` (every language, 4 runs); and Rust,
  run as a JSON filter, without the writer's options the original's
  tests give (36 runs: the same in process, by `cargo test`).

How long the whole suite (`run.py test`) takes per language: with 70
cases, Python 1.5 s, TypeScript 4.8 s (a Node.js process per case), Rust
0.1 s (built), Julia 5.5 s in one process (`julia-cases.jl`; 13 cases
took 12.6 s a process each, so 70 would take about a minute); with the
third batch's, 134 cases, Python 3.2 s, TypeScript 10.1 s, Rust 0.1 s,
Julia 18.8 s (more methods to compile: the regular expressions, the
chained cases' two scripts each); with the fourth's, 524 cases, Python
11.7 s, TypeScript 37.1 s, Rust 0.3 s, Julia 58.3 s (in one process, each
case's script included anew: compiling it is most of that); with the
fifth's, 843 cases, Python 20.7 s, TypeScript 63.6 s, Rust 0.5 s, Julia
372 s, of which amsthm's 292 cases are about 300 (its 600 lines compiled
anew in each case's module, 1.2 s a case: `run.py` now stops
`julia-cases.jl` after `$TIMEOUT` a case, not for them all).

## Lines of code

Non-blank, non-comment lines, docstrings not counted; the Rust ports as
rustfmt leaves them. In brackets, for the first three, before panir#14
(attributes by key, `ctx.fail`, `ctx.meta()`, `Attr::new`, `tag`,
`inlines_mut`); for the third batch, before panir#16 (Lua's truth,
`lookup`, `items`, plain values, a list of filters, Rust's `Option<()>`
functions); for the Rust ports since the fourth batch, the count before
panir#17 (`Block::raw`, `Inline::raw`, `ctx.ok`, `given`, `into_items`,
`lookup` from a value), and for those before it, in brackets alone,
likewise. caps' Rust was given as 12 at first; it is 13. TypeScript's
three ports changed by panir#17 (`given`) kept their counts.

| filter | original | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| caps | 6 (Python) | 7 | 6 | 13 | 3 |
| metavars | 17 (Python) | 19 | 18 | 28 (40 before panir#14) | 13 |
| list-table | 209 (Lua; 176 without its old-pandoc polyfill and markdown error context) | 139 (144) | 118 (119) | 216 (239; 218 before #17) | 107 (111) |
| deemph | 7 (Python; 13 with `caps.py`) | 11 | 12 | 29 | 4 |
| comments | 18 (Python) | 24 | 20 | 33 | 13 |
| deflists | 8 (Python) | 7 | 5 | 21 | 4 |
| theorem | 25 (Python) | 17 | 19 | 42 (49) | 16 |
| inlinenotes | 19 (Python) | 22 | 25 | 34 (38) | 17 |
| gitlab_markdown | 14 (Python) | 13 | 10 | 28 | 10 |
| french-spaces | 20 (Python, panflute) | 16 | 14 | 37 (41) | 15 |
| scholarly-metadata | 129 (Lua) | 92 (100) | 84 (87) | 172 (191; 175 before #17) | 71 (77) |
| author-info-blocks | 134 (Lua) | 108 (117) | 61 (66) | 148 (178; 160 before #17) | 54 (61) |
| pandoc-quotes | 178 (Lua; 104 without its table of 72 languages, 74 lines in every language) | 140 (144; 66 without the table) | 119 (120; 45) | 171 (180; 97; 176 before #17) | 114 (120; 40) |
| abstract-section | 63 (Lua) | 61 (69) | 52 (53) | 75 (84; 78 before #17) | 40 (42) |
| cito | 207 (Lua; 52 without its table of properties, 155 lines) | 126 (46; the table 80) | 91 (39; 52) | 225 (89; 136; 233 before) | 89 (43; 46) |
| pagebreak | 103 (Lua) | 79 | 48 | 91 (101) | 42 |
| track-changes | 221 (Lua; 174 without its LaTeX preamble, a string) | 152 (102 without it) | 127 (80) | 216 (169; 229 before #17) | 131 (80) |
| first-line-indent | 192 (Lua; 173 without its CSS) | 148 (130) | 106 (88) | 212 (194; 229 before #17) | 111 (92) |
| count-para | 130 (Lua; 117 without its CSS) | 138 (125) | 102 (89) | 221 (210; 235 before #17) | 91 (90) |
| auto-lang | 294 (Lua; its table of scripts, 279 lines, generated) | 256 | 227 | 412 | 207 |
| amsthm | 1,033 (Lua) | 718 | 619 | 1,400 (and a test of 41) | 602 |

Rust is two to five times the others on these small filters: rustfmt's
layout (a `let ... else` is three lines, a struct literal one per field),
the `impl Filter` scaffolding, and taking values out of the node
(`std::mem::take`) before moving them into a replacement.

The third batch, the first made mostly of Lua originals, is mostly
metadata work: Julia's ports are the shortest (60% of the Lua on
scholarly-metadata: a method per kind of value, conversion on
assignment), TypeScript's close, Python's longer than its logic as `ruff
format` spreads calls and imports, Rust's the longest (no conversion from
plain values, an index type where Lua's tables alias). What each
operation costs, per language: `GAPS.md`, "Metadata, by language".

The fourth batch, Lua originals that read nested options and write
`header-includes`, is shorter than its originals in TypeScript and Julia
(about half of the Lua, the long strings aside: track-changes 80 lines to
174), close to them in Python, and as long or longer in Rust, where each
port builds raw nodes with struct literals and turns its helpers'
`Result`s into failures by hand (`GAPS.md` 30, 31). Lua's truth,
`lookup`, `items`, the plain values and lists of filters left no helper
for them in any port. panir#17 (`Block::raw`, `Inline::raw`, `ctx.ok`,
`given`, `lookup` from a value) took 6 to 10% off those four Rust ports,
and 1 to 14% off the eight earlier ones it touched; Rust is now 0.9
(pagebreak) to 1.7 (count-para) times their Lua.

The fifth batch, the owner's two: auto-lang's ports are without their
tables (each about 830 lines, made by `auto-lang/tables.py`, 121, from the
original's), as the original is without its; amsthm, the largest of round
1, is 70% of the Lua in Python, 60% in TypeScript and Julia, and 135% in
Rust, rustfmt's layout of its many `match`es and long format strings,
with no helper written for a Lua idiom (`Block::raw`, `ctx.ok`, `given`,
`into_items` from panir#17).

## Time and memory

Each run as pandoc runs a JSON filter, a whole process: the document's
JSON in, the filter, JSON out. The originals: pandocfilters' under the
same Python (panflute's too); the Lua one as `pandoc -f json -L list-table.lua` plus a
filter that writes the JSON and ends pandoc before its writer (so pandoc's
reading and writing of JSON count, as a JSON filter's do). Best of 5
(pandoc manual) and of 3 (textbook); seconds and peak resident MB.

pandoc's manual (1.5 MB of JSON):

| filter | original | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| caps | 0.07 s, 58 MB | 0.07 s, 27 MB | 0.11 s, 113 MB | 0.01 s, 25 MB | 0.33 s, 367 MB |
| metavars | 0.07 s, 59 MB | 0.08 s, 31 MB | 0.10 s, 120 MB | 0.01 s, 24 MB | 0.41 s, 379 MB |
| list-table | 0.13 s, 153 MB | 0.07 s, 32 MB | 0.10 s, 115 MB | 0.01 s, 22 MB | 1.02 s, 418 MB |
| deemph | 0.06 s, 57 MB | 0.07 s, 31 MB | 0.10 s, 117 MB | 0.01 s, 25 MB | 0.42 s, 367 MB |
| comments | 0.06 s, 57 MB | 0.07 s, 31 MB | 0.11 s, 125 MB | 0.01 s, 24 MB | 0.39 s, 368 MB |
| deflists | 0.07 s, 57 MB | 0.07 s, 31 MB | 0.10 s, 114 MB | 0.01 s, 25 MB | 0.45 s, 381 MB |
| theorem | 0.07 s, 57 MB | 0.07 s, 31 MB | 0.10 s, 113 MB | 0.01 s, 23 MB | 0.44 s, 384 MB |
| inlinenotes (markdown) | 0.07 s, 57 MB | 0.07 s, 27 MB | 0.10 s, 113 MB | 0.01 s, 25 MB | 0.45 s, 382 MB |
| gitlab_markdown | 0.07 s, 58 MB | 0.07 s, 31 MB | 0.10 s, 120 MB | 0.01 s, 25 MB | 0.35 s, 367 MB |
| french-spaces (latex) | 0.17 s, 47 MB | 0.08 s, 28 MB | 0.11 s, 124 MB | 0.01 s, 22 MB | 0.37 s, 369 MB |
| scholarly-metadata | 0.09 s, 150 MB | 0.06 s, 32 MB | 0.10 s, 117 MB | 0.01 s, 21 MB | 0.57 s, 387 MB |
| author-info-blocks (after scholarly-metadata) | 0.09 s, 151 MB | 0.13 s, 32 MB | 0.20 s, 117 MB | 0.01 s, 24 MB | 1.44 s, 419 MB |
| pandoc-quotes | 0.14 s, 158 MB | 0.06 s, 30 MB | 0.10 s, 112 MB | 0.01 s, 23 MB | 0.52 s, 375 MB |
| abstract-section | 0.12 s, 153 MB | 0.07 s, 31 MB | 0.11 s, 128 MB | 0.01 s, 23 MB | 0.48 s, 376 MB |
| cito | 0.14 s, 159 MB | 0.07 s, 30 MB | 0.10 s, 115 MB | 0.01 s, 22 MB | 0.59 s, 379 MB |
| pagebreak (asciidoc) | 0.11 s, 153 MB | 0.07 s, 31 MB | 0.10 s, 114 MB | 0.01 s, 23 MB | 0.42 s, 367 MB |
| track-changes (html) | 0.14 s, 158 MB | 0.07 s, 30 MB | 0.10 s, 114 MB | 0.01 s, 23 MB | 0.48 s, 374 MB |
| first-line-indent (html) | 0.14 s, 152 MB | 0.06 s, 28 MB | 0.10 s, 118 MB | 0.01 s, 22 MB | 1.14 s, 425 MB |
| count-para (html) | 0.19 s, 162 MB | 0.07 s, 29 MB | 0.11 s, 124 MB | 0.01 s, 24 MB | 0.65 s, 388 MB |
| auto-lang (html; no map: nothing to do) | 0.15 s, 156 MB | 0.07 s, 30 MB | 0.11 s, 119 MB | 0.01 s, 22 MB | 1.28 s, 418 MB |
| amsthm (latex; no map: the proof only) | 0.16 s, 158 MB | 0.08 s, 33 MB | 0.11 s, 126 MB | 0.01 s, 22 MB | 2.49 s, 513 MB |

The textbook (`quantum-shannon`, 9.8 MB of JSON):

| filter | original | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| caps | 0.44 s, 270 MB | 0.32 s, 89 MB | 0.24 s, 280 MB | 0.05 s, 60 MB | 0.58 s, 456 MB |
| metavars | 0.43 s, 260 MB | 0.35 s, 101 MB | 0.26 s, 298 MB | 0.04 s, 60 MB | 0.75 s, 471 MB |
| list-table | 0.53 s, 343 MB | 0.28 s, 90 MB | 0.25 s, 275 MB | 0.04 s, 68 MB | 1.26 s, 510 MB |
| deemph | 0.42 s, 262 MB | 0.29 s, 105 MB | 0.25 s, 283 MB | 0.04 s, 68 MB | 0.67 s, 455 MB |
| comments | 0.41 s, 261 MB | 0.31 s, 90 MB | 0.26 s, 291 MB | 0.04 s, 61 MB | 0.65 s, 456 MB |
| deflists | 0.41 s, 260 MB | 0.27 s, 89 MB | 0.23 s, 273 MB | 0.04 s, 68 MB | 0.66 s, 456 MB |
| theorem | 0.41 s, 260 MB | 0.27 s, 88 MB | 0.26 s, 278 MB | 0.04 s, 71 MB | 0.66 s, 459 MB |
| inlinenotes (markdown) | 0.42 s, 260 MB | 0.28 s, 87 MB | 0.24 s, 275 MB | 0.04 s, 61 MB | 0.68 s, 461 MB |
| gitlab_markdown | 0.41 s, 260 MB | 0.30 s, 89 MB | 0.25 s, 276 MB | 0.04 s, 68 MB | 0.62 s, 456 MB |
| french-spaces (latex) | 0.99 s, 174 MB | 0.39 s, 89 MB | 0.26 s, 284 MB | 0.05 s, 69 MB | 0.70 s, 456 MB |
| scholarly-metadata | 0.43 s, 319 MB | 0.25 s, 89 MB | 0.23 s, 277 MB | 0.04 s, 61 MB | 0.74 s, 463 MB |
| author-info-blocks (after scholarly-metadata) | 0.43 s, 315 MB | 0.51 s, 90 MB | 0.47 s, 276 MB | 0.09 s, 79 MB | 1.74 s, 496 MB |
| pandoc-quotes | 0.67 s, 373 MB | 0.25 s, 104 MB | 0.23 s, 275 MB | 0.04 s, 61 MB | 0.70 s, 441 MB |
| abstract-section | 0.54 s, 344 MB | 0.28 s, 104 MB | 0.27 s, 305 MB | 0.04 s, 69 MB | 0.71 s, 460 MB |
| cito | 0.69 s, 378 MB | 0.29 s, 104 MB | 0.24 s, 278 MB | 0.05 s, 71 MB | 1.01 s, 456 MB |
| pagebreak (asciidoc) | 0.53 s, 347 MB | 0.28 s, 105 MB | 0.25 s, 273 MB | 0.04 s, 69 MB | 0.65 s, 458 MB |
| track-changes (html) | 0.65 s, 388 MB | 0.28 s, 88 MB | 0.24 s, 278 MB | 0.05 s, 69 MB | 0.71 s, 461 MB |
| first-line-indent (html) | 1.13 s, 337 MB | 0.26 s, 105 MB | 0.25 s, 288 MB | 0.04 s, 61 MB | 1.32 s, 488 MB |
| count-para (html) | 1.45 s, 416 MB | 0.29 s, 92 MB | 0.25 s, 287 MB | 0.05 s, 71 MB | 0.90 s, 463 MB |
| auto-lang (html; no map) | 0.71 s, 369 MB | 0.28 s, 88 MB | 0.24 s, 276 MB | 0.04 s, 70 MB | 1.53 s, 527 MB |
| amsthm (latex; no map) | 0.82 s, 373 MB | 0.30 s, 92 MB | 0.25 s, 281 MB | 0.04 s, 61 MB | 3.08 s, 582 MB |

Julia's times are mostly compiling (each run is a new process, and
list-table has more methods to compile); in a host that keeps Julia
running (LibPandoc.jl's `pandocjl`) they would not be paid per document.

The seven pandocfilters-sized filters cost what reading and writing the
JSON costs: the filter's own work is lost in it in every language. The
one panflute original, french-spaces, is the slowest original (panflute
builds its own objects, with parents, from the JSON): 2.4 times
pandocfilters' time on the textbook, and 2.5 to 20 times the ports'.

The third batch's Lua originals run in pandoc, which reads and writes
the JSON itself, and pays for the conversion of the document to Lua and
back: 0.4 to 0.7 s on the textbook, against 0.23 to 0.29 s for Python and
TypeScript and 0.04 s for Rust. author-info-blocks is timed with
scholarly-metadata before it, as it always runs: the originals in one
pandoc, the ports in two processes (two readings and writings of the
JSON: Python and TypeScript are then slower than the original, Julia
compiles twice). Neither document sets a language, so pandoc-quotes only
reads the metadata; the textbook's 476 citations are cito's work.

The fourth batch's originals are lists of filters in pandoc (first-line-
indent two, count-para four), each a walk over the document converted to
Lua and back: on the textbook 1.1 and 1.5 s, four to six times Python's
and TypeScript's, twenty-five to thirty times Rust's. Julia's
first-line-indent is its slowest yet on the manual (1.1 s), compiling two
filters' methods in a new process.

The fifth batch's two do real work for every node, where they have a map
to work from: auto-lang classifies every character of the text and
regroups every list of inlines; amsthm, given environments, walks every
block twice and each environment's body three times more (held out, set
in its font, put back). On the documents with maps (best of 3):

| filter, document | original | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| auto-lang, War and Peace, every script mapped (25.3 MB of JSON: all its English in spans) | 5.09 s, 638 MB | 0.99 s, 226 MB | 0.62 s, 564 MB | 0.17 s, 154 MB | 2.04 s, 616 MB |
| auto-lang, the Wikipedia article, every script but Latin (3.0 MB) | 0.60 s, 198 MB | 0.27 s, 61 MB | 0.17 s, 166 MB | 0.02 s, 31 MB | 1.94 s, 476 MB |
| auto-lang, the pandoc manual, every script (1.7 MB) | 0.46 s, 151 MB | 0.18 s, 33 MB | 0.15 s, 151 MB | 0.02 s, 26 MB | 1.78 s, 449 MB |
| amsthm, the textbook, 738 environments (11.2 MB), html | 1.39 s, 380 MB | 0.42 s, 94 MB | 0.30 s, 306 MB | 0.05 s, 63 MB | 3.40 s, 606 MB |
| amsthm, the textbook, latex | 0.87 s, 379 MB | 0.31 s, 94 MB | 0.28 s, 293 MB | 0.04 s, 73 MB | 3.12 s, 585 MB |

Here the comparison is of the filters, not of reading JSON. The Lua
original of auto-lang is the slowest of the five on War and Peace, 5 s,
most likely its strings built a character at a time (`run.text ..
char`) besides the conversion of the document to Lua and back; Python takes 1 s, the
same algorithm with its strings joined once; TypeScript 0.6 s, V8's
strings and `Map`s; Rust 0.17 s, 30 times the original. amsthm in HTML,
where it numbers, sets fonts and resolves references, is 1.4 s in Lua,
0.4 and 0.3 s in Python and TypeScript, 0.05 s in Rust. Julia's are
mostly compiling: 1.3 to 2.5 s with nothing to do, a large filter's
methods compiled anew in each process (amsthm's 600 lines are 2.5 s), so
its own work is a second at most, between Python's and TypeScript's
where it shows (War and Peace: 2.0 s, where the textbook with nothing to
do is 1.5). In a
host that keeps Julia running, as LibPandoc.jl's `pandocjl` does, the
compiling is paid once.

