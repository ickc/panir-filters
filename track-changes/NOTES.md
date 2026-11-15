# track-changes

pandoc/lua-filters' track-changes: Word's tracked changes, as pandoc's
docx reader makes them with `--track-changes=all` (spans of class
`insertion`, `deletion`, `comment-start`, `comment-end`, with `author`
and `date`), accepted, rejected, or shown in HTML (`ins`, `del`, `mark`
as raw HTML) and LaTeX (the `changes` package's commands, its preamble
and a colour per author in `header-includes`); in Word and ODT kept, a
comment's `¶` a line break. A `Pandoc` function chooses the filter at run
time and walks the body with it.

## Input

The changes come from a reader option: pandoc's docx reader makes the
spans only when reading with `--track-changes=all`. The cases are
markdown with those spans written out, as the docx reader makes them
(the repository's own `sample.md` does the same), and three go through
docx: `docx.md`, `docx-reject.md` and `docx-accept.md` are written as
Word by pandoc and read back with `--track-changes=all` or `reject`
(`NAME.via` holds `docx`, `NAME.args` the reader's options: `run.py`).
No Word document is kept; pandoc writes the same one each time
(`SOURCE_DATE_EPOCH=0`), and only the JSON read from it is in
`expected.jsonl`. pandoc's docx writer keeps no custom metadata, so their
`trackChanges` is given with `-M`. The tracked changes survive the round
trip: insertions, deletions and a comment, with their authors and dates
(a comment's identifier becomes a `comment-id` attribute).

## The original's behaviour, kept

- **The reader's option is never read.** The original takes
  `PANDOC_READER_OPTIONS.trackChanges` first, as pandoc 2 named it, with
  pandoc 2's values (`AllChanges`); since pandoc 3 the field is
  `track_changes` (`all-changes`), so `trackChanges` is `nil` and the
  metadata's `trackChanges` alone decides (`accept` when missing or not a
  mode). So do the ports: `ctx.conversion`'s reader options are there in
  every language, but reading them would differ from the original (case
  `docx-reject`: read with `reject`, shown with `all`; `docx-accept`:
  read with `all`, accepted). For upstream: read `track_changes` and its
  values.
- **`header-includes` is always wrapped in a list**: the original tests
  `meta['header-includes'].t == 'MetaList'`, which no metadata value has
  had in Lua since pandoc 2.17 (pandoc-quotes failed on the same), so a
  list becomes a list in a list (case `header-includes`), text a list of
  it. For upstream: `pandoc.utils.type(...) == 'List'`. The ports keep the
  nesting: panir's `items`, which is what the original means, would
  differ.
- **The mode, as Lua reads it**: a string as it is, a list, map or text
  as its `stringify` (`trackChanges: {mode: all}` is `all`, case
  `mode-map`), a boolean is no mode; anything but `all` and `reject`
  accepts (`mode-true`, `mode-unknown`: `All`).
- **The first class decides** what a change is shown as (`.note
  .insertion` is no change, case `all`); accepting and rejecting test the
  classes in the original's order: a span of both is an insertion (cases
  `both`, `both-reject`).
- **Initials**: a name with a space is its words' first letters, ASCII
  only (Lua's `%w`: `Émile Zola` is `MZ`, the `É` not being a letter in
  Lua), upper-cased, titles (`Dr`, `Mr`, `Prof`, ...) left out; a name of
  one word as it is. Authors' colours are given in the order of their
  initials, by byte; two authors of the same initials are one (the later
  name kept: `Zed Y` and `Zed Young`, case `header-includes`), and a
  seventh gets `auth7`, a colour the preamble doesn't define (case
  `all`). For upstream: more colours, or reuse them.
- **HTML**: attributes in their order, each but `date` as `data-...`,
  their values not escaped; a comment always gets `data-id`, empty when it
  has no identifier (Lua's `""` is true). The comment's text is its
  `title`, a `¶` as `&#10;`.
- **A change without an author fails** where it is shown (`all`, HTML and
  LaTeX: case `error-author`), not where it is accepted or rejected
  (`no-author`).
- Only the body is walked (`pandoc.walk_block(pandoc.Div(blocks), ...)`):
  changes in the metadata stay (case `metadata`).
- `all` in any other format changes nothing but removes `trackChanges`.

## By language

The filter chosen at run time is one function chosen in every language
(Python registers it on a new `Filter`, TypeScript gives `applyFilter` an
object of it, Rust an enum of modes in one struct, Julia a function
value), and the walk is of `doc.blocks`.

- **Python**: `pilcrows(line)` makes the small filter for a comment's
  `¶`; `span.walk(...)` returns the span, so `stringify(span.walk(...))`
  reads as Lua's `stringify(pandoc.walk_inline(elem, ...))`.
  `stringify(doc.meta.pop("trackChanges", MetaString("accept")))` is the
  mode in one line. `doc.meta["header-includes"] = [...]` converts a
  list.
- **TypeScript**: `String.raw` keeps the LaTeX preamble as the original
  has it; `getAttribute` is `undefined` without an author, so the port
  throws where Lua fails on `nil`.
- **Rust**: `ctx.apply(&mut span.content, &mut Pilcrows(...))?` from an
  `inline` function; the authors a `BTreeMap`, so in order. The raw
  inlines and blocks are `Inline::raw` and `Block::raw` (panir#17), a raw
  block as metadata `MetaValue::from(Block::raw(..))`; before, struct
  literals in a `raw` helper, and `Block::from(RawBlock { .. }).into()`,
  two conversions (gap 30). `author()` returns a `Result`, which its
  callers give the walk with `ctx.ok(self.author(span))?` (panir#17; a
  `match` into `ctx.fail` before, gap 31).
- **Julia**: a Julia `"""` string drops a newline right after the
  quotes, so the preamble (which starts with one) begins with two; the
  comment's filter is an anonymous function on `Str`.

Nothing here needed a copy: every node made is new, and the changes'
content is moved into their place.

`ctx.conversion`'s reader options, which a fixed original would read, are
`reader_options` (Python, Rust, Julia) and `readerOptions` (TypeScript),
pandoc's JSON (`"track-changes": "all-changes"`), not Lua's names
(`track_changes`); no "Coming from Lua" maps `PANDOC_READER_OPTIONS` to
them (`GAPS.md` 34).
