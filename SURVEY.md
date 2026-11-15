# Survey (round 1)

Where the filters come from, what each demands of a filter API, and why
it was taken or not. Surveyed 2026-11-14; commits as fetched that day.

Round 1 takes filters that need only the AST: a document in, a document
out, deterministic, no file reading, no network, no external programs,
no call back into pandoc; from permissively licensed originals; plus two
of the owner's own (`pandoc-amsthm`, `auto-lang.lua`).

## Licences, checked before reading any source

| source | licence (as checked) | used |
|---|---|---|
| [jgm/pandocfilters](https://github.com/jgm/pandocfilters) `examples/` | BSD-3-Clause (`LICENSE`, © 2013 John MacFarlane) | yes |
| [sergiocorreia/panflute](https://github.com/sergiocorreia/panflute) `examples/` | BSD-3-Clause (`LICENSE`); its `examples/pandocfilters/` and `input/` carry pandocfilters' licence | as a second reading of pandocfilters' examples |
| [pandoc/lua-filters](https://github.com/pandoc/lua-filters) | MIT (`LICENSE`); some files say ISC in their header (scholarly-metadata, author-info-blocks, revealjs-codeblock); `first-line-indent/` and `not-in-format/` MIT with their own `LICENSE`; `pandoc-quotes.lua/` MIT (`LICENSE.txt`) | yes |
| [pandoc-ext](https://github.com/pandoc-ext) | MIT each (`LICENSE`), except `info` (CC-BY-4.0, not code) and `math2svg` (none); `pagebreak.lua`'s header is ISC text under an MIT `LICENSE` | yes |
| [cysouw/count-para](https://github.com/cysouw/count-para) | CC0-1.0 (`LICENSE`); the file's header is ISC text | yes |
| [chdemko/pandoc-latex-french-spaces](https://github.com/chdemko/pandoc-latex-french-spaces) | BSD-3-Clause (`LICENSE`) | yes |
| [ickc/pandoc-amsthm](https://github.com/ickc/pandoc-amsthm) | BSD-3-Clause (`LICENSE`; header says so too) | yes (owner's) |
| [ickc/font](https://github.com/ickc/font) `bin/auto-lang.lua` | BSD-3-Clause (`LICENSE`; the file has no header) | yes (owner's) |
| pandoc's Lua filters manual (jgm/pandoc `doc/lua-filters.md`) | **GPL-2.0-or-later**: pandoc's `COPYRIGHT` releases everything under the GPL "with the exceptions noted below", and none of the exceptions covers `doc/` (they are pandoc-types' modules, some readers and writers, the Lua engine, which is GPL too, and bundled third-party code) | **no**: not read, not ported |
| pandoc wiki, [Pandoc Filters](https://github.com/jgm/pandoc/wiki/Pandoc-Filters) | a list of links (read); each linked project checked on its own | see below |

Both of the owner's filters have a licence (BSD-3-Clause); nothing is
missing there.

Where a header and the repository's `LICENSE` differ (ISC text in an MIT
repository; ISC text in a CC0 one), both are permissive; a port keeps the
file's own header and the repository's `LICENSE`.

From the wiki, by licence (GitHub's API, before opening anything):
GPL, so out of round 1 and not read: pandoc-crossref (GPL-2.0),
pandoc-fignos, -eqnos, -tablenos (GPL-3.0), pandoc-subfigs (GPL-2.0),
pandoc-placetable (GPL-2.0). No licence found, not read:
pandoc-abbreviations; pandoc-unicode-math ("NOASSERTION": to check by
hand before a later round).

## Features

What a filter needs of the API, as abbreviations in the tables and in
`filters.tsv`:

| code | the filter... |
|---|---|
| `replace` | replaces a node by one node |
| `splice` | replaces a node by several, or by none |
| `topdown` | depends on the top-down order (pandocfilters' walk, Lua's `traverse = "topdown"`), or on skipping a node's children |
| `state` | keeps state across elements (a counter, a flag, a table filled in one pass and used in another) |
| `siblings` | needs the previous or next element of the same list |
| `meta-walk` | reads the metadata while walking the body |
| `meta` | reads or writes the metadata (`Meta` function, `doc.meta`) |
| `meta-plain` | builds metadata from plain values (strings, maps, lists) |
| `header-includes` | appends to `header-includes`, whatever shape it has (absent, one value, a list) |
| `table` | builds or reads tables |
| `format` | depends on the output format (`FORMAT`) |
| `raw` | makes raw blocks or inlines, or reads them |
| `attr` | reads or writes attributes (identifier, classes, key-values) |
| `subwalk` | walks a subtree, or the document, with a second filter |
| `query` | walks only to collect, changing nothing |
| `stringify` | needs `stringify` |
| `lists` | `Inlines`/`Blocks` list functions |
| `doc` | a whole-document function (`Pandoc`) |
| `multi` | several filters run one after the other |
| `cite` | citations |
| `math` | math |
| `deflist` | definition lists |
| `note` | notes |
| `span-div` | spans or divs selected by class |
| `equal` | compares nodes for equality |
| `env` | reads the environment or pandoc's reader options |
| `error` | stops the conversion with a message |
| `unicode` | depends on Unicode properties (case, scripts) |

## Chosen for round 1 (21)

| name | source | licence | lang | what it does | there for |
|---|---|---|---|---|---|
| caps | pandocfilters `examples/caps.py` | BSD-3 | Python | upper-cases every `Str` | the trivial inline rewrite; metadata is walked too; Unicode case mapping per language |
| deemph | pandocfilters `examples/deemph.py` | BSD-3 | Python | replaces emphasis by its content upper-cased | `subwalk` (a filter used inside another), `splice` |
| comments | pandocfilters `examples/comments.py` | BSD-3 | Python | drops everything between two HTML comments | `state` in document order, `topdown`, `raw`, deleting blocks |
| deflists | pandocfilters `examples/deflists.py` | BSD-3 | Python | definition lists to bullet lists with strong terms | `deflist`, building blocks from parts |
| metavars | pandocfilters `examples/metavars.py` | BSD-3 | Python | `%{field}` in text becomes the metadata field's value | `meta-walk`, metadata values to inlines, `attr` |
| theorem | pandocfilters `examples/theorem.py` | BSD-3 | Python | divs of class `theorem` to LaTeX environments or numbered HTML | `span-div`, `format`, `raw`, a counter, `splice` of blocks |
| inlinenotes | pandocfilters `examples/inlinenotes.py` | BSD-3 | Python | notes of paragraphs become markdown inline notes | `note`, `query` (collect inlines of a subtree), `format` |
| gitlab_markdown | pandocfilters `examples/gitlab_markdown.py` | BSD-3 | Python | `math` code blocks to display math; strips backticks from inline math | `math`, code block classes |
| french-spaces | chdemko/pandoc-latex-french-spaces | BSD-3 | Python (panflute) | spaces before French punctuation to non-breaking ones in LaTeX | `siblings` (panflute's `prev`/`next`) |
| list-table | pandoc-ext/list-table | MIT | Lua | RST-like list tables (nested bullet lists in a div) to tables | `table`, `attr` (as a map), empty spans as attribute holders, `error` (with the offending markdown) |
| pagebreak | pandoc-ext/pagebreak | MIT (ISC header) | Lua | raw `\newpage`, form feeds, to each format's page break | `doc`, `subwalk`, `meta` (nested options, removed after), `env`, `format`, `raw` |
| abstract-section | pandoc-ext/abstract-section | MIT | Lua | a section headed "Abstract" moves into the metadata | `lists` (`Blocks`) in a `subwalk`, `meta`, `state`, `stringify` |
| cito | pandoc-ext/cito | MIT | Lua | `@prop:key` citations: property into metadata, key cleaned | `cite`, `subwalk` of a cite's content, `meta-plain` (a map of lists) |
| scholarly-metadata | pandoc/lua-filters | MIT (ISC header) | Lua | normalises authors and institutes in the metadata | `meta`, `meta-plain`, metadata types, `stringify`, heavy list and map work |
| author-info-blocks | pandoc/lua-filters | MIT (ISC header) | Lua | affiliations and correspondence as blocks before the body | blocks built from metadata, `format`, `doc`; runs after scholarly-metadata |
| track-changes | pandoc/lua-filters | MIT | Lua | accept, reject or render tracked changes (docx input) | `span-div`, `attr`, `format`, `env` (reader options), filters chosen at run time, `header-includes` |
| pandoc-quotes | pandoc/lua-filters `pandoc-quotes.lua` | MIT | Lua | quotation marks by the document's language | `multi` (configure, then rewrite), `meta`, `Quoted` to inlines |
| first-line-indent | pandoc/lua-filters | MIT | Lua | first-line indents, removed after quotes and lists | `siblings` at block level (the previous block), `equal`, `header-includes`, options from `meta` |
| count-para | cysouw/count-para | CC0-1.0 (ISC header) | Lua | numbers paragraphs; citations of their ids become links | `state` (collect, then resolve), `multi`, `cite`, `header-includes`, `format` |
| amsthm | ickc/pandoc-amsthm | BSD-3 | Lua | amsthm's theorem and proof environments for any format | everything at once: `topdown` with skipped children, nested `subwalk`s, counters, `cite` as references, `meta`, `header-includes`, `format` |
| auto-lang | ickc/font `bin/auto-lang.lua` | BSD-3 | Lua | spans with `lang` (and `dir`) around runs of a mapped script | `lists` (`Inlines`) `topdown`, skipping children, `multi`, `unicode` (scripts), a data table shipped with the filter |

Notes on the owner's two: amsthm reads a theorem's title (`[Title]{...}`
info) with `pandoc.read` (markdown), which is round 3's need, and is 1,323
lines; port it last in round 1, its titles through libpandoc or marked as
round 3 there. auto-lang `dofile`s a generated table (`script-ranges.lua`,
the Unicode scripts' ranges) next to it: the ports need the same table in
each language (generated, as upstream generates its own), not a file read
at run time.

pandoc-quotes keeps its quotation marks as a table in the script (no
file read: round 1), and the ports have the same table. Under pandoc 3 its
original fails on every `quot-marks` and `quot-marks-by-lang` (it tests
`meta.t`, which metadata has not had in Lua since pandoc 2.17): its cases
are made with those two tests fixed (`pandoc-quotes/upstream/ORIGIN`).
author-info-blocks is run after scholarly-metadata, the originals and the
ports alike (`after:` in its `ORIGIN`).

The fourth batch kept its four as chosen. track-changes' `env` turned out
to be pandoc 2's: it reads `PANDOC_READER_OPTIONS.trackChanges`, which
pandoc 3 names `track_changes`, so under pandoc 3.14 only the metadata
decides; the reader's option still makes its input (docx read with
`--track-changes=all`), which its cases go through (`NAME.via`).
pagebreak's `env` is two environment variables (`NAME.env`).

How the features are covered (each at least twice where possible):

| feature | filters |
|---|---|
| `replace` | caps, metavars, gitlab_markdown, french-spaces, count-para |
| `splice` | deemph, comments, theorem, inlinenotes, track-changes, pandoc-quotes |
| `topdown` | comments, metavars, amsthm, auto-lang |
| `state` | comments, theorem, abstract-section, count-para, first-line-indent, amsthm |
| `siblings` | french-spaces, first-line-indent |
| `meta-walk` | metavars, (pagebreak, track-changes, count-para read it first) |
| `meta` | pagebreak, abstract-section, scholarly-metadata, author-info-blocks, pandoc-quotes, first-line-indent, count-para, amsthm, auto-lang |
| `meta-plain` | cito, scholarly-metadata |
| `header-includes` | track-changes, first-line-indent, count-para, amsthm |
| `table` | list-table |
| `format` | theorem, inlinenotes, french-spaces, pagebreak, author-info-blocks, track-changes, first-line-indent, count-para, amsthm |
| `raw` | comments, theorem, inlinenotes, pagebreak, track-changes, count-para |
| `attr` | metavars, list-table, track-changes, count-para, auto-lang |
| `subwalk` | deemph, pagebreak, abstract-section, cito, track-changes, amsthm |
| `query` | inlinenotes, amsthm |
| `stringify` | abstract-section, scholarly-metadata, author-info-blocks, track-changes, count-para |
| `lists` | abstract-section, auto-lang |
| `doc` | pagebreak, author-info-blocks, track-changes, first-line-indent, count-para |
| `multi` | pandoc-quotes, count-para, auto-lang, amsthm |
| `cite` | cito, count-para, amsthm |
| `math` | gitlab_markdown, amsthm |
| `deflist` | deflists |
| `note` | inlinenotes |
| `span-div` | theorem, track-changes, amsthm, auto-lang |
| `equal` | first-line-indent |
| `env` | pagebreak, track-changes |
| `error` | list-table |
| `unicode` | caps, auto-lang |

## Not taken in round 1

| filter | source | why |
|---|---|---|
| abc, blockdiag, ditaa, gabc, graphviz, lilypond, plantuml, tikz | pandocfilters | run programs |
| includes | pandocfilters | reads files (round 2); Python 2 only (`unicode`) |
| latexdivs | pandocfilters | duplicate of theorem |
| myemph | pandocfilters | duplicate (format + splice: theorem, deemph) |
| caps, deemph, ... | panflute `examples/panflute/` | ports of pandocfilters' examples: a second oracle, not new filters |
| headers | panflute | trivial |
| build-table | panflute | needs YAML parsing (`yaml_filter`); tables covered by list-table |
| table, table-better | panflute | broken against panflute 2 (`e.items`; undefined `cell`, `pf`) |
| abstract-to-meta | lua-filters | superseded by pandoc-ext/abstract-section |
| cito, list-table, pagebreak, include-files | lua-filters | older versions of pandoc-ext's |
| latex-hyphen, not-in-format, revealjs-codeblock, minted | lua-filters | duplicates (`format` + `raw` + `attr`) |
| scrlttr2 | lua-filters | duplicate of scholarly-metadata (metadata only) |
| mhchem | lua-filters | an lpeg grammar: porting a parser, not testing the API |
| wordcount | lua-filters | its output is a report and `os.exit`, not a document |
| lua-debug-example | lua-filters | a debugger demo |
| fonts-and-alignment | pandoc-ext | duplicate (`span-div` to raw), 508 lines |
| pretty-urls | pandoc-ext | small and covered (`stringify`, links); a candidate for a later round |
| logging | pandoc-ext | a library |
| pandoc-latex-environment | chdemko | duplicate of theorem |
| the Lua manual's examples | jgm/pandoc `doc/` | GPL-2.0-or-later |
| pandoc-crossref, fignos/eqnos/tablenos, pandoc-subfigs | wiki | GPL (round 4: clean-room only) |

## Later rounds

- **Round 2, read files:** include-files (pandoc-ext, lua-filters),
  include-code-files (lua-filters), pandocfilters' includes, pantable
  (CSV).
- **Round 3, call pandoc:** short-captions and table-short-captions
  (`pandoc.read`), multibib, section-bibliographies,
  multiple-bibliographies, section-refs (citeproc); amsthm's titles
  (`pandoc.read`); list-table's error message (`pandoc.write`, only when
  failing); diagram (`pandoc.read` for captions, besides programs).
- **Not planned, run programs or reach the network:** pandocfilters' abc,
  blockdiag, ditaa, gabc, graphviz, lilypond, plantuml, tikz; diagram,
  diagram-generator, math2svg, lilypond (lua-filters), bibexport
  (bibtex), spellcheck (aspell), doi2cite (network), run-lua (runs code
  from the document).
