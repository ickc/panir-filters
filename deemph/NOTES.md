# deemph

pandocfilters' `examples/deemph.py`: an emphasis is replaced by its
content, upper-cased by pandocfilters' `caps` (imported from `caps.py`
beside it, so `upstream/` holds both).

## The original's behaviour, kept

- **A filter used inside another.** The original walks the emphasis's
  content with `caps`, and returns the list, spliced in the emphasis's
  place. Each port does the same with panir's walk of a fragment:
  `emph.content.walk(caps)`, `applyFilter(emph.content, caps)`,
  `ctx.apply(content, &mut Caps)`, `walk!(caps,
  emph.content)`; each returns the list it walked. Each port defines its
  `caps` (three lines) rather than importing the caps port: a port is one
  file.
- **Top-down, and an emphasis directly in another stays one.**
  pandocfilters calls the function on a node, then walks the fields of
  what it returned, not the returned nodes themselves. So in `*outer
  _nested_ outer*` the inner emphasis, spliced in, is never given to the
  function: it stays an emphasis (its text upper-cased by `caps`), while
  one in a strong in an emphasis is removed (case `edges`). panir's
  top-down order does exactly this (a function's result is not given to
  the function again; its children are; the READMEs don't quite say so:
  `GAPS.md`, 13), so the ports ask for it. The
  default (typewise, bottom-up) order would remove both. For upstream: a
  quirk, probably unintended.
- **The metadata is walked too**: an emphasis in the title goes (case
  `edges`).
- An empty emphasis is removed, as the original splices in an empty list
  (case `empty-emph`, from JSON: the markdown reader makes none).

## By language

- **Julia**: `uppercase` maps `ß` to `ẞ`, not `SS` (as in `caps`): case
  `unicode`, in `known.tsv`. The shortest port, two methods.
- **Rust**: `ctx.apply(content, &mut Caps)?` walks the content with the
  walk's conversion and fails with the inner function's error, then the
  content is taken out (`std::mem::take`) and returned: two lines. It
  took six before (`panir::apply_with` given the conversion by hand, its
  `Result` matched into `ctx.fail(e)`; `GAPS.md`, 12).
- **Python, TypeScript**: one line each; the walk of a list returns the
  list, ready to return.

Differential runs (pandoc's test documents, eight real documents, five
documents made of every filter's cases): the same AST in every language,
but Julia on `ß` as above.
