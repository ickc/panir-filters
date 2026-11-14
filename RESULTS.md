# Results

Per filter and language: the cases (`run.py test`), the differential runs
against the original (`run.py diff`), lines of code, and time and peak
memory (`run.py time`). Measured 2026-11-14 on Linux (x86-64, 32 cores),
pandoc 3.14, Python 3.15, Node.js 26, Rust 1.97, Julia 1.12; panir at
`09c14e8` for the first three filters' times, at `d02476a` (panir#14's
branch) for everything else.

## Cases and differential runs

The differential runs: pandoc's test documents (`fetch-docs.sh`: 5) and
eight real documents (the pandoc manual and changelog; GitLab's GraphQL
reference, all tables; a Wikipedia article read from HTML; Python's
`stdtypes` from RST; a 774-page textbook from LaTeX; a Chinese page of
code; War and Peace from EPUB); and, from the second batch on, five
documents made of every filter's cases (`assemble.py`), as the real ones
hold little of most filters' syntax: 18 documents, each in each output
format the filter's cases use. A case is a document in one format.

| filter | cases | Python | TypeScript | Rust | Julia | differential (each language) |
|---|---|---|---|---|---|---|
| caps | 3 | 3/3 | 3/3 | 3/3 | 2/3, 1 known | 18/18; Julia 16/18, 2 known |
| metavars | 4 (1 fails in the original) | 4/4 | 4/4 | 4/4 | 4/4 | 18/18 (1 fails in the original and every port) |
| list-table | 6 (4 fail in the original) | 6/6 | 6/6 | 6/6 | 6/6 | 18/18 |
| deemph | 5 | 5/5 | 5/5 | 5/5 | 4/5, 1 known | 18/18 |
| comments | 7 (1 fails in the original) | 6/7, 1 known | 6/7, 1 known | 5/7, 2 known | 6/7, 1 known | 18/18 |
| deflists | 4 | 4/4 | 4/4 | 4/4 | 4/4 | 18/18 |
| theorem | 16 (4 formats) | 16/16 | 16/16 | 16/16 | 16/16 | 72/72 |
| inlinenotes | 10 (2 formats) | 9/10, 1 known | 9/10, 1 known | 9/10, 1 known | 9/10, 1 known | 35/36, 1 known |
| gitlab_markdown | 3 | 3/3 | 3/3 | 3/3 | 3/3 | 18/18 |
| french-spaces | 12 (3 formats) | 12/12 | 12/12 | 12/12 | 12/12 | 54/54 |

In all, 1,152 port runs against the original: 1,146 the same (one of them
a failure of the original and the port alike), 6 known differences
(inlinenotes on the textbook in every language, caps on two documents in
Julia). The whole run takes 9 minutes, mostly Julia's start.

Known differences (`known.tsv`, explained in each `NOTES.md`):

- Julia's `uppercase` maps `ß` to `ẞ` where the others give `SS` (caps,
  deemph);
- comments: the original, on a beginning in a table cell, writes JSON
  pandoc can't read (every language); Rust has no function for metadata
  values, so it empties a hidden `MetaList`'s items where the original
  deletes them (`GAPS.md`, 14);
- inlinenotes: the original's query walks the note's JSON and repeats
  nested emphasis, or leaves a note whose emphasis or citation holds a
  link, code or raw inline (every language; a case, and the textbook).

How long the whole suite (`run.py test`, 70 cases) takes per language:
Python 1.5 s, TypeScript 4.8 s (a Node.js process per case), Rust 0.1 s
(built), Julia 5.5 s in one process (`julia-cases.jl`; 13 cases took
12.6 s a process each, so 70 would take about a minute).

## Lines of code

Non-blank, non-comment lines, docstrings not counted; the Rust ports as
rustfmt leaves them. In brackets, before panir#14 (attributes by key,
`ctx.fail`, `ctx.meta()`, `Attr::new`, `tag`, `inlines_mut`). caps' Rust
was given as 12 at first; it is 13.

| filter | original | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| caps | 6 (Python) | 7 | 6 | 13 | 3 |
| metavars | 17 (Python) | 19 | 18 | 28 (40 before panir#14) | 13 |
| list-table | 209 (Lua; 176 without its old-pandoc polyfill and markdown error context) | 139 (144) | 118 (119) | 218 (239) | 107 (111) |
| deemph | 7 (Python; 13 with `caps.py`) | 11 | 12 | 29 | 4 |
| comments | 18 (Python) | 24 | 20 | 33 | 13 |
| deflists | 8 (Python) | 7 | 5 | 21 | 4 |
| theorem | 25 (Python) | 17 | 19 | 49 | 16 |
| inlinenotes | 19 (Python) | 22 | 25 | 38 | 17 |
| gitlab_markdown | 14 (Python) | 13 | 10 | 28 | 10 |
| french-spaces | 20 (Python, panflute) | 16 | 14 | 40 | 15 |

Rust is two to five times the others on these small filters: rustfmt's
layout (a `let ... else` is three lines, a struct literal one per field),
the `impl Filter` scaffolding, and taking values out of the node
(`std::mem::take`) before moving them into a replacement.

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

Julia's times are mostly compiling (each run is a new process, and
list-table has more methods to compile); in a host that keeps Julia
running (LibPandoc.jl's `pandocjl`) they would not be paid per document.

The seven pandocfilters-sized filters cost what reading and writing the
JSON costs: the filter's own work is lost in it in every language. The
one panflute original, french-spaces, is the slowest original (panflute
builds its own objects, with parents, from the JSON): 2.4 times
pandocfilters' time on the textbook, and 2.5 to 20 times the ports'.
