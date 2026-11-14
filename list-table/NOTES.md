# list-table

pandoc-ext/list-table: a div of class `list-table` holding a bullet list
of bullet lists becomes a table. The richest of the three: it builds every
part of a table, reads and removes attributes, and fails on malformed
input.

## The original's behaviour, kept

- **Returning `nil` undoes what the function changed.** The original
  removes the div's first class, then returns `nil` for an empty div or a
  caption alone; pandoc's Lua then keeps the element as it was (a
  function's changes count only when it returns the element). panir keeps
  changes made in place whatever the function returns, so every port
  decides first and changes the div after (case `edges`: the empty div
  keeps its class). Easy to get wrong when porting from Lua (`GAPS.md`).
- **Its quirks**, all in case `edges` or `sample`: a table in an item
  contributes its body rows only (its head is dropped); a table as the
  first item doesn't start a new body; `header-cols` applies to the first
  body, even when that body ends up empty and is dropped; the column count
  is the first row's, colspans added; `widths` divides by the sum of all
  widths given, columns or not; an attribute span's following space stays
  in the cell (`[]{#id} Name` gives a cell starting with a space);
  `Caption`'s short caption is an empty list, not none.
- **Numbers as Lua reads them** (`tonumber`): `header-rows=abc` is the
  default, `1.5` counts as 1. Each port has a small `number` for that,
  decimal only; Lua's hexadecimal (`0x10`) is not ported, nor widths that
  add up to 0 (Lua divides by zero into `nan`, so do the others but
  Python, whose `/` raises). `colspan="2.0"` passes in Lua (a float with
  an integer value) and fails in the ports.
- **Errors.** The original stops with a message and the offending blocks
  written as markdown (`pandoc.write`). The ports stop with the message
  only: writing markdown needs pandoc (libpandoc), which a JSON filter
  doesn't have (round 3's need). The cases check that a port fails where
  the original does (`error-*`), not the message. Where the original's
  message comes from Lua indexing a generic `.content`, the Rust and
  Julia ports, which match on block types, say another of the original's
  messages (`error-two-blocks`).
- `data-pos` divs (pandoc's `sourcepos` extension) are unwrapped as the
  original does; no case has one (markdown's reader doesn't make them).

## By language

- **Attributes by key** are most of the work: the filter reads and removes
  seven (`colspan`, `rowspan`, `align`, `aligns`, `widths`,
  `header-rows`, `header-cols`). Python has `attributes["k"]`, `.get`,
  `del`, but no `pop(key, default)`, so each port has a `pop` helper;
  TypeScript, Rust and Julia have only the list of pairs (`findIndex` and
  `splice`; `iter().position` and `remove`; `findfirst` and `popat!`).
  The biggest gap found here.
- **"The inlines of this block, if any"**: Lua and Python take
  `block.content` and look at its first element; TypeScript can (`"content"
  in block`, then tell an inline from a list of blocks); Rust and Julia
  match `Plain`, `Para`, `Header` (`inlines_of`), which is also what the
  original means.
- **The name of a block's constructor** for messages: `.t` in TypeScript,
  `type(b).__name__` in Python, `nameof(typeof(b))` in Julia; panir's Rust
  has none, the port takes it from `Debug`'s output.
- **Rust**: a filter function can't fail, so the port's logic returns a
  `Result` and the `Filter` method prints the error and exits the process.
  Taking a table apart (`let Table { attr, caption, bodies, .. } =
  *table`) and the one-element checks (`<[Block; 1]>::try_from(item)`) are
  pleasant; the port is the longest of the four because rustfmt spreads
  it out, and because of `inlines_of`, `tag` and the error plumbing.
- **TypeScript**: the types follow narrowing well (`list.t !==
  "BulletList"`), except that a `shift()` loses what `content[0]?.t ===
  "Para"` had narrowed: take the element first. `Cell(content, {
  alignment, rowSpan, colSpan, attr })` and `Table(caption, specs, head,
  bodies, foot, { attr })` read well.
- **Julia**: `skip_data_pos` and `inlines_of` by dispatch are idiomatic
  and short. Constructors copy the lists they are given, so a body made
  with `TableBody(Row[])` is filled through `bodies[end].body`, not through
  the vector passed in.
- **Python**: closest to the original; the `Alignment` enum (not its
  names as strings) for pyright to accept `ALIGNMENTS`' values.

Differential runs (pandoc's test documents, eight real documents, which
have no list tables): the same AST in every language; the cases are what
exercises the filter.
