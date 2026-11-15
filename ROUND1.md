# Round 1, in a page

Round 1 took real pandoc filters that need only the AST (no file read, no
call into pandoc, no program run), from permissively licensed originals:
21, chosen in [`SURVEY.md`](SURVEY.md) to cover what a filter API is asked
for (28 features: replacing and splicing, top-down walks, state, siblings,
metadata, `header-includes`, tables, formats, raw output, attributes,
sub-walks, lists of filters, citations, Unicode...): nine of pandocfilters'
examples and panflute's french-spaces (Python), ten Lua filters of
pandoc-ext, pandoc/lua-filters and cysouw, and the owner's two, auto-lang
and amsthm. Each was ported to the four languages of
[panir](https://github.com/ickc/panir), 84 ports, as a user of each
language and of panir's documented API would write them.

## How they were checked

- **Cases**: 843 runs of an original (269 documents, each in the formats
  its filter cares about; 82 where the original fails, and the port must
  fail too), kept with what the original made under pandoc 3.14: 3,372
  port runs, 3,194 the same AST. The 178 that differ are each explained
  (`known.tsv`, `NOTES.md`): the original's bugs that a port can't or
  shouldn't copy (JSON pandoc can't read, Lua's `tostring` of blocks,
  pandocfilters walking a note's JSON), Julia's `uppercase("ß")`, pandoc's
  Lua making an empty list item `[Plain []]`, amsthm's notes with markup
  (round 3, below), and the Rust amsthm run without the writer's options a
  JSON filter is not given (the same in process, by `cargo test`).
- **The originals' own samples and tests** are cases, for 20 of the 21
  (one has none): their sample documents and test inputs, verbatim, and
  amsthm's whole suite (its cases, golden test and documentation's
  examples, with the writer's options its specs give them).
- **`CHECK_UNIQUE=1`**: panir's check that no node is in two places, on
  every port's output (Python, TypeScript, Julia; Rust's ownership needs
  none). It found every missing copy (scholarly-metadata's merge,
  pagebreak's page break, amsthm's held environments).
- **Differential runs** on documents the filters were not written for:
  pandoc's test documents, eight real ones (the pandoc manual and
  changelog, GitLab's GraphQL reference, a Wikipedia article from HTML,
  Python's `stdtypes` from RST, a 774-page textbook from LaTeX, a Chinese
  page of code, War and Peace from EPUB), eight documents made of every
  filter's cases at random depths with metadata of their own, and, for the
  last two filters, the real documents with an `auto-lang` map and the
  textbooks with an `amsthm` map: 4,372 port runs in the last round of
  them, 4,358 the same, 14 known (the cases' kinds, on real text). Over
  the five batches they found two bugs in ports the cases had passed (a
  missing copy in scholarly-metadata, a missing failure in
  first-line-indent), besides the originals' bugs each `NOTES.md` lists.

## The gaps, by theme

Porting found 40 gaps in panir's filter API (`GAPS.md`, in the order
found, each with the filters it touches counted from their sources); panir
closed 34 of them in four pull requests (#14 to #17), 3 in part, as it
declined the rest; 6 are open. 18 of the 21 filters hit at least one;
caps, deflists and gitlab_markdown hit none.

| theme | gaps | filters touched | what changed |
|---|---|---|---|
| attributes | 1, 2 | 6 | by key in every language (`attributes["k"]`, `getAttribute`, `attr.get`); `Attr::new` in Rust |
| metadata as Lua has it | 5, 11, 14, 15, 17, 18, 19, 21, 22, 23, 26–29, 32, 33, 39 | 14 | Lua's truth (`bool`, `truthy`, `given`), `lookup`, `items`, plain values both ways, Rust's `From`s; the metadata's table in each "Coming from Lua"; open: a value as inlines or blocks as Lua converts it (33), `pairs` (39) |
| errors | 3, 24, 31 | 9 | Rust's `ctx.fail`, every function an `Option`, `ctx.ok(result)`; every language names the function and the node's path |
| building nodes in Rust | 2, 6, 9, 23, 25, 30 | 15 | `Attr::new`, `tag()`, `inlines_mut`, `Target::new`, `Block::raw`, `Inline::raw`, metadata from plain values |
| documentation | 4, 8, 13, 17, 34, 35, 39 | 13 | "Coming from Lua" in each README: `return nil`, elements and tables as values, top-down replacements, `.t`, `gsub`'s `%`, strings as bytes, `PANDOC_READER_OPTIONS`; 39 open |
| walk semantics | 12, 13, 14, 20, 36 | 12 | `ctx.apply` for sub-walks, `meta_value`, lists of filters in every language; open: Rust's in-place list functions declining (36) |
| safety | 7, 8, 15, 35 | 5 | a depth limit on every walk; `check_unique`; `[]` for a map's value an error |
| the conversion and hosts | 10, 16, 34, 37, 38 | 7 | typed `ctx.doc` and `ctx.parent` in TypeScript; open: the writer's options (38), scripts of more than one file run in process (37) |
| pandoc's helpers | 40 | 1 | open: `to_roman_numeral` |

## Each language, against pandoc's Lua, for a filter author

Lines of code (non-blank, non-comment, without docstrings, the generated
tables and Rust's test), all 21: the originals 3,047; Python 2,293 (75%),
TypeScript 1,883 (62%), Rust 3,824 (125%), Julia 1,756 (58%). Time, as
pandoc runs a JSON filter (a process each, reading and writing the JSON):
on the 774-page textbook, the 21 filters take 12.9 s as originals (a Lua
filter in pandoc pays the conversion of the document to Lua and back,
0.4 to 1.5 s each), 6.3 s in Python, 5.4 s in TypeScript, 0.9 s in Rust,
20.3 s in Julia, which compiles in every new process. Where a filter does
real work for every node, the gap widens: auto-lang on War and Peace with
every script mapped, 5.1 s in Lua, 1.0 in Python, 0.6 in TypeScript, 0.17
in Rust; amsthm numbering the textbook's 738 environments in HTML, 1.4,
0.4, 0.3, 0.05.

- **Python**: as easy as Lua for everything round 1 asked: a function by
  type, `match` on nodes, Lua's truth as Python's (`if meta.get("x"):`),
  `lookup`, `items`, plain values both ways, `main([f, g])`. Easier: an
  error at the assignment that makes it, with the path; regular
  expressions for Lua's patterns; dataclasses for options. Harder: a node
  in two places is the author's to copy (Lua copies), which only
  `check_unique` finds; `ruff format` spreads calls.
- **TypeScript**: about as easy, and the shortest after Julia: plain
  objects narrowed by `t`, `given(...) ?? d` for Lua's `or`, `runFilter([f,
  g])`. Harder: the types of what is untyped in pandoc (the conversion's
  options, a plain value going back into a constructor: casts); a class
  can't take parameter properties under `erasableSyntaxOnly`.
- **Rust**: everything can be said, at twice Python's length (rustfmt's
  layout, a struct and `impl Filter` per walk, `std::mem::take` to move
  out of `&mut`); since panir#17 no helper is written for Lua's idioms
  (`Block::raw`, `ctx.ok`, `given`, `into_items`). Harder: list functions
  change in place, so declining means deciding first (36); a filter is a
  program, so it is never given what only a host knows (38). Easier:
  ownership makes copies explicit (nothing to check), and it is 10 to 30
  times faster than anything else here.
- **Julia**: the shortest: a method per kind of node reads as Lua's table
  of functions, conversion on assignment as Lua's, parametric callable
  structs for a family of filters (amsthm's fonts). Harder: compiling (a
  new process pays 0.3 to 1.5 s; a host keeping Julia running doesn't),
  `if` takes a `Bool` only (`truthy`), and a script run in process needs
  a module with `include` (37).

## What round 1 could not test

- **Reading files** (round 2): include-files, include-code-files,
  pantable; auto-lang's table is compiled in, where the original `dofile`s
  it, so `PANDOC_SCRIPT_FILE` and paths beside a filter went untested.
- **Calling pandoc** (round 3): `pandoc.read` (amsthm's notes, which the
  ports read as plain text only; short-captions), `pandoc.write`
  (list-table's error message), citeproc (multibib, section-refs). Python
  has libpandoc for it; the other languages' bindings were not used.
- **Running programs**: diagram, graphviz, plantuml and the like, not
  planned.
- **The writer's options in Rust**, and in any language under pandoc
  itself: a JSON filter is not given them (38); checked here in process
  only.
- **Hosts**: the Python, TypeScript and Julia ports were run in process
  only by this repository's hosts (`host/`, `julia-cases.jl`), not by
  libpandoc's, which is where 37 bites.

## Open

- Gaps 38 (the writer's options: pandoc to pass them to JSON filters,
  libpandoc to set them, panir to read them, and "Coming from Lua" to say
  so), 37 (running a filter of several files in process: panir's
  hand-off documented or wrapped, libpandoc-python's runner putting the
  script's folder on `sys.path`), 33 (a metadata value as inlines or
  blocks, as Lua converts it), 36, 39, 40 (low).
- For the owner: auto-lang's and amsthm's lists of what looks unintended
  (`auto-lang/NOTES.md`, `amsthm/NOTES.md`); the other originals' bugs
  are in their `NOTES.md`, and two (pandoc-quotes' `.t`, track-changes'
  `trackChanges`) are pandoc 2's API under pandoc 3.
- Julia's cases take minutes now (a module per case compiles each
  script anew: amsthm's 292 about 1.2 s each); a host that reuses a
  script's module per port would not.
