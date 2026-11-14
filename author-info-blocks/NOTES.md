# author-info-blocks

pandoc/lua-filters' author-info-blocks: the body starts with a note on
equal contributors, the numbered affiliations and the corresponding
authors (linked), and `author` becomes one line of names with their marks
(in LaTeX, a list of them); `institute` goes. It reads the metadata as
scholarly-metadata leaves it, and runs after it: its cases run
scholarly-metadata first, the originals in one pandoc (`-L
scholarly-metadata.lua -L author-info-blocks.lua`), each language's ports
one after the other (`after: scholarly-metadata` in `ORIGIN`). The one
filter so far that puts the metadata's nodes into the body.

## The original's behaviour, kept

- **Without authors it fails** (but for LaTeX): the last author is taken
  from an empty list, and `List.extend(nil)` stops the conversion. After
  scholarly-metadata every document has an `author` list, so every
  document without authors fails: case `no-author`, and in the
  differential runs eight of the real and made documents (html). The
  ports fail too. For upstream: no author line when there are no authors.
- **Lua's truth** (case `marks`): `equal_contributor: no` is text, so
  true; `false` is false; `correspondence: 1` is true; a corresponding
  author needs `correspondence` and an `email`.
- **The affiliations paragraph is always there**, empty without
  institutes (`Para []`).
- **Two tests of the format**: the marks are LaTeX's when `FORMAT ==
  'latex'`, the author list a list when `FORMAT:match 'latex'`; the ports
  keep both (`format == "latex"`, `"latex" in format`), which differ for no
  format pandoc has.
- **A name or an email must be inlines**: a `MetaString` or `MetaBool` name
  (only from JSON or `-M`) makes `List.clone` fail, an email of either
  makes the concatenation fail; the ports fail on any name or email that
  isn't inlines. Where the original does not fail (a name that is a map,
  dropped; blocks or a list in LaTeX, written as they are) the ports
  differ: no case holds these, and the made documents avoid them.
- An email holding a link makes a link in a link (case `marks`), as the
  original; its `mailto:` is the email's text.

## Nodes in two places

Here the README's "Lua's elements are values" is the whole story, and
`CHECK_UNIQUE=1` agrees with what was copied:

- **an author's name** is in the author line and, for a corresponding
  author, in the link: one copy (`copy.deepcopy`, `copy`, `deepcopy`,
  `.clone()`);
- **the marks** (`✉`, `*`, the LaTeX ones) and the separators (`,`, the
  line break) are made anew each time they are used: a function returning
  new nodes, where the original keeps one table of them (in Lua each use
  is a copy);
- the institutes' names and the emails are moved, not copied: `institute`
  is removed and `author` replaced, so they are in one place.

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| a field, missing or false | built in | `given`, 2 lines | 1 | 3 | 3 (methods) |
| `author`/`institute` as scholarly-metadata leaves them (a list of maps) | read as tables | `items` and `maps`, 11 lines | 9 | 24 | 3 |
| a name or email that must be inlines | (fails by itself) | 4 | 4 | 9 | 2 |
| writing `author` back | `pandoc.MetaInlines(...)` | `MetaInlines(*line)` | `MetaInlines(line)` | `MetaValue::MetaInlines(line)` | `MetaInlines(line)` |

- **Python**: closest to the original; `doc.blocks = [*body, *doc.blocks]`
  (assigning a field copies the list); `ctx.format` is `str | None` for
  pyright, so `"latex" in (ctx.format or "")`. `ruff format` makes it the
  longest but Rust (the imports and the long `Para(...)` calls).
- **TypeScript**: the shortest with Julia; `ctx.format?.includes`
  (`ctx.format` may be `undefined`); `doc.blocks.unshift(...body)`,
  `delete doc.meta.institute`.
- **Rust**: the authors are taken out of the metadata (`remove`), so the
  names could move; they are cloned for both uses, being borrowed while
  the author line is made. `x.tag()` names the kind in the messages.
  `Link` and `Target` have no defaults or constructor (`Link { attr:
  Attr::default(), content, target: Target { url, title: "".into() } }`,
  where Lua writes `pandoc.Link(content, url)`; `GAPS.md` 25). `ctx.fail`
  from `pandoc` needs `ctx.fail::<()>(e)` (`GAPS.md` 24).
- **Julia**: an array literal takes its element type from what is in it:
  `[mark(...)]` was a `Vector{Vector{RawInline}}`, and appending a `Str`
  to it failed; `Inline[...]` where a list mixes kinds.

Differential runs: the same AST in every language. Of the real documents,
pandoc's manual, its test suite, the textbook and War and Peace have
authors (a name each, no institute, so an empty affiliations paragraph);
the others fail in the original and the ports alike (html), as do some of
the made documents.
