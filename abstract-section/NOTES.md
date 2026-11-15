# abstract-section

pandoc-ext/abstract-section: a section whose heading has the identifier
`abstract` (or one of `abstract-section.section-identifiers`), at the
highest level met so far, moves into the metadata field of that name, as
blocks. A `Pandoc` function: it reads the configuration, walks the body's
lists of blocks with a second filter, then writes the metadata.

## The original's behaviour, kept

- **The body's lists only** (`doc.blocks:walk{Blocks = ...}`): every list
  of blocks, inner ones first, the body's own list too; not the
  metadata's blocks (case `existing`, a `# Abstract` in `note:`). The
  ports walk `doc.blocks` as a fragment, whose list function is called on
  the list itself too, as Lua's.
- **"The highest level met so far"** is one counter for the whole walk,
  in the walk's order (inner lists first, in document order): an abstract
  in a list item at level 1 makes a level-2 one in a later quote stay
  (case `nested`). A heading below that level doesn't end a section: it is
  collected (`## Background`).
- **The last abstract wins**: a second one starts the collection anew
  (case `nested`); a horizontal rule ends a section and is dropped
  (`hrule`, `two-sections`).
- **The section goes even if the field is set**: the metadata's
  `abstract` stays, the body's section is dropped (case `existing`).
  Lua's `not meta[key]` lets a false field be replaced. An empty section
  sets nothing; its heading goes (case `empty`).
- **The configuration, as Lua reads it** (and it is always removed):
  - `abstract-section` a map: its `section-identifiers`; true: Lua
    indexes a boolean and fails (case `error-config`); false or missing:
    the default (`config-false`); anything else (a string, a list): no
    field in Lua, so the default.
  - `section-identifiers`: a list, inlines or blocks, not empty: the text
    of each item. So inlines are one identifier per inline, a space one
    too: `summary abstract` is `summary`, ` ` and `abstract` (case
    `config-inlines`). A non-empty string (from JSON): Lua takes its length,
    then `ipairs` finds no items: no identifiers, nothing moves (case
    `config-string`). True: Lua takes the length of a boolean and fails
    (`error-identifiers`). Anything else: the default (a map, case
    `config-map`).
- An identifier `""` (from `section-identifiers: [""]`) would match every
  heading without one: Lua's `""` is true. The ports test "is there a
  section", not the identifier's truth (`section is not None`).

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| the configuration: a nested option (`(meta['abstract-section'] or {})['section-identifiers']`), then a list, inlines or blocks of identifiers, a string or a boolean | 8 | 16 → 13 | 17 → 16 | 19 → 17 | 8 |
| a field, missing or false | built in | `given`, 2 lines → `not doc.meta.get(key)`, as Lua | 1 → `truthy` | 1 (a `matches!` in place) → `truthy` | 3 → `truthy` |
| the walk of the body's lists, from the `Pandoc` function | `doc.blocks:walk{Blocks = f}` | `doc.blocks.walk(sections)` | `applyFilter(doc.blocks, { Blocks })` | `ctx.apply(&mut doc.blocks, &mut sections)` | `walk!(sections, doc.blocks)` |

Before panir#16 no language read a nested option in one step (`GAPS.md`
21); now `lookup` does, and the configuration is one test (`true` fails,
as in Lua) and a `lookup`. `items` (`GAPS.md` 22) is no help here: the
original takes a list, inlines or blocks alike with `ipairs`, and a
non-empty string as no identifiers, not one; the `match` on the kinds
stays in every language.

- **Python**: one `match` with three constructors in one arm
  (`MetaList(items) | MetaInlines(items) | MetaBlocks(items) if items`) is
  Lua's "anything `ipairs` takes"; the walk's state in the `Pandoc`
  function, its list function a closure (`nonlocal toplevel`).
- **TypeScript**: `switch (ids?.t)` with three cases falling through to
  one `map`; `ids.content.map(stringify)` doesn't type-check on the union
  of three array types, so `(x: unknown) => stringify(x)`.
- **Rust**: the list function of a `Sections` struct (state in its
  fields, `std::mem::take(xs)` then `*xs = body`); a `match` on `(&block,
  &section)` reads as the original's branches. Three arms for the three
  constructors (their items are different types). Before panir#16,
  `pandoc` returned `()`, so `if ctx.apply(...).is_none() { return; }`
  and `ctx.fail::<()>(e); return;` (`GAPS.md` 24); now `ctx.apply(...)?`
  and `Err(e) => return ctx.fail(e)`. `lookup(&doc.meta,
  &["abstract-section", "section-identifiers"])` takes the metadata, not
  a value: the configuration is read from `doc.meta`, then removed.
- **Julia**: the identifiers by dispatch (`identifiers(::Union{MetaList,
  MetaInlines, MetaBlocks})`), the configuration a `lookup` (it was three
  methods of `configured`): the shortest; the list method a closure over
  the counter.

Differential runs: the same AST in every language. No real document has
an abstract section; the made documents hold the cases' (and their
configurations, reshaped: one made `abstract-section` a list, others
dropped it).
