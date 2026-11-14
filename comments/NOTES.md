# comments

pandocfilters' `examples/comments.py`: everything between a raw HTML
block holding `<!-- BEGIN COMMENT -->` and one holding `<!-- END COMMENT
-->` is left out, the markers too.

## The original's behaviour, kept

The original is a flag set and cleared as pandocfilters' walk meets the
markers, and, while it is set, a function that deletes whatever it is
given. What "whatever" is comes from pandocfilters' walk: every JSON
object with a `t` in a list, in document order, the metadata first, and
not inside what was deleted. The ports follow it with panir's top-down
walk and functions for blocks, inlines and metadata values:

- **A marker anywhere in a raw HTML block counts** (the original searches
  the text), and a block holding both is a beginning (case `edges`). A
  marker as inline HTML, or in a raw block of another format, does nothing.
- **An end without a beginning** is dropped, nothing else (`edges`).
- **A beginning nested in a block** hides the rest of that block and goes
  on after it; an end is seen only if it is not inside something already
  hidden. A beginning in a list item empties the following items but
  keeps them (`nested`); an end inside a quote or a list met while hiding
  goes with it, unseen, and the comment goes on (`nested`). A beginning
  never closed hides the rest of the document (`edges`).
- **A beginning in a note** hides the rest of the note, then the rest of
  the paragraph the note is in (its inlines), then what follows (`note`):
  so the ports delete inlines too.
- **The metadata comes first**: a beginning in a metadata field hides the
  fields after it in key order, then the body until an end (`meta`).
  There the original deletes what is in a list (a `MetaList`'s items, a
  `MetaInlines`' inlines, a `MetaBlocks`' blocks) and keeps a map's values
  (the field itself, a `MetaMap`'s entries, `MetaBool` and `MetaString`,
  which have no list in them). The ports delete a metadata value only in a
  `MetaList`.
- **Bug, a known difference: a beginning in a table cell.** The original
  then deletes the following cells' alignments (objects with a `t`, in the
  cell's JSON list) and writes JSON pandoc can't read ("expected Array of
  length 5 but got Array of length 4"): the conversion fails. The ports
  keep the cells, emptied, as they keep list items. Case `table`, in
  `known.tsv` for every language. For upstream: delete blocks and inlines
  only.
- The `fmt` the original takes from the raw block shadows its `format`
  argument; harmless.

## By language

- **Python**: three functions, on `Block`, `Inline` and `MetaValue` (the
  base classes), and a module-level flag (`global`, as the original). In
  the metadata, returning `[]` for a value deletes it from a list and,
  in a map, deletes the key: the function checks `isinstance(ctx.parent,
  MetaList)`. Since `GAPS.md` 15 was closed, `[]` for a map's value is an
  error in every language, saying how to delete a key; the check is
  needed.
- **TypeScript**: `Block`, `Inline`, `MetaValue` functions;
  `ctx.parent?.t === "MetaList"`, as the other ports test the parent
  (`ctx.parent` is typed by the function since `GAPS.md` 16 was closed;
  before, `Array.isArray(ctx.container)`).
- **Julia**: a method for `RawBlock`, one for `Union{Block, Inline}`, one
  for `MetaValue` with the context; the flag a `const Ref`. Dispatch reads
  well here.
- **Rust**: `block`, `inline` and `meta_value`, the flag in the filter's
  struct (`self.hiding.then(Vec::new)` deletes); `meta_value` deletes
  only where `ctx.in_list()` (Rust's context has no parent). Before
  panir's `Filter` had `meta_value` (`GAPS.md`, 14), a hidden `MetaList`'s
  items were emptied, not deleted, a known difference (case `meta`).

Differential runs (pandoc's test documents, eight real documents, which
have no markers, and five documents made of every filter's cases, which
have: markers at every depth, unclosed ones, the metadata's): the same AST
in every language.
