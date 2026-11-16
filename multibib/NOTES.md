# multibib

pandoc-ext/multibib: the metadata's `bibliography` maps topics to files,
and a div `#refs-TOPIC` gets the bibliography of that topic's file, of
every work the document cites. Round 3's filter that calls pandoc for
something other than reading: citeproc, from inside the filter
(`pandoc.utils.citeproc`), on the whole document once (every file
together, so that citations resolve) and on a small document per topic
(the citations, and a `#refs` div for citeproc to fill).

The cases are the original's test (`sample`, its two bibliographies in
`cases/files/test/`) and five of this repository's: three topics named the
three ways (`refs-`, `refs_`, `refs`), a div naming no topic, a document
`#refs`, citations in a note, `lang` and `link-citations` (`topics`); a
bibliography given as one file, or as a list (`single`, `list`: no
topics, as the original reads only a map); inline `references` with
`nocite: '@*'` (`references`); none at all (`no-bibliography`). In HTML
and docx (the original names Word's bibliography style). They run in
`cases/files`.

## How each port gets citeproc

| | citeproc | under plain pandoc |
|---|---|---|
| Python | `libpandoc.convert(json, from_="json", to="json", citeproc=True)`, its warnings silenced (`pandoc.log.silence` in the original) | libpandoc as a library, if installed |
| TypeScript | the `pandoc` command with `--citeproc --quiet` on the JSON, a process a call (as the original does where pandoc's Lua has no citeproc) | the same |
| Rust | built for wasm, `libpandoc::convert` with `"citeproc": true`, the host's | none: it fails, saying to build it for wasm |
| Julia | `LibPandoc.convert(json; from = "json", to = "json", citeproc = true)`, its log silenced (`NullLogger`) | LibPandoc as a library, if in the environment |

No host has a call for citeproc itself; `convert` from JSON to JSON with
the option is the way in each, and does what `pandoc.utils.citeproc`
does on these cases.

## The original's behaviour, kept

- **`orig_bib`**: after resolving the citations the original sets
  `doc.meta.bibliography = orig_bib`, a global it never assigns: the
  bibliography leaves the metadata (every case with one). Kept.
- **The map only**: `pandoc.utils.type` of a map is `table`; of a list
  `List`, of text `string`: a list of files or one file give no topics,
  and citeproc then runs with no bibliography (`single`, `list`: the
  document's citations unresolved). A name indexes a map; Lua would index
  a list or text by it too, to a method for names such as `map` or `sub`
  (not ported: no case names one).
- **The topic's div** takes citeproc's classes and attributes, its own
  dropped (`refs-sources .keep key=value` in `topics`); the document's own
  `#refs` goes (the block citeproc filled is the last one, removed).
- **Each topic's bibliography** is of every citation in the document,
  with the metadata citeproc reads (eleven fields, `nocite` among them).

## By language

- **Python**: `MetaList([...])` is a list holding a list (the constructors
  take their items as arguments, `MetaList(*files)`): a list of files
  became one path, `test/secondary.bib test/primary.bib`, which pandoc
  then couldn't find. `Pandoc(*blocks, meta=...)` likewise.
- **TypeScript**: `Pandoc(blocks, { meta })` builds the small document.
- **Rust**: the passes are three walks in `main`, the first's result
  moved into the third; the citeproc call is a function compiled for wasm
  only.
- **Julia**: the shortest, a function a pass, the metadata pass a method
  of the citations' function.
- **State** (Python, TypeScript, Julia): the citations collected are
  module state, emptied by a first `Pandoc` function: a host running these
  filters in process on many documents (filter functions reused, a Julia
  script loaded once per process) would otherwise collect every
  document's citations, where Lua loads the original anew for each.

Copies: the citations are copied when collected (Lua's are).

## In the hosts

The same in every host, but a wasm filter's (`known.tsv`): its call to
citeproc runs in pandoc's sandbox (the calls of a wasm filter are
`"untrusted"`), so citeproc can't read the bibliography files the
filter itself can read (`PandocResourceNotFound`), and the filter fails
wherever the metadata names one (`sample`, `topics`, `references`), in
the four wasm hosts (`HOSTS.md`).
