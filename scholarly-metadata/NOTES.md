# scholarly-metadata

pandoc/lua-filters' scholarly-metadata: `author` and `institute` become
lists of maps with a `name` and an `id`, an author's institutes become
numbers into `institute`, which collects the top-level institutes and
those given in authors (each with its `index`). All metadata: a `Meta`
function, no walk. The heaviest metadata work of round 1, so the one that
shows what panir's typed metadata costs against Lua's plain values.

## The original's behaviour, kept

Everything follows from how Lua sees pandoc's metadata (checked with
pandoc 3.14): a `MetaMap` is a plain table, a `MetaList` a `List`,
`MetaInlines` `Inlines`, a `MetaString` a Lua string, a `MetaBool` a Lua
boolean; `pandoc.utils.type` tells them apart (`table`, `List`, `Inlines`,
`string`, `boolean`, `Blocks`); and a table goes back as a `MetaMap` (a
`MetaList` if it has a list part; `{}` is an empty `MetaMap`, `List{}` an
empty `MetaList`).

- **Every document gets `author` and `institute`**, empty lists if it has
  none (case `none`).
- **Text with commas is a list of names** (`ensure_list`, case `csv`,
  `csv-authors`): only `MetaInlines`, split at every comma as Lua 5.4's
  `gmatch('([^,]*)')` does (empty names kept), each trimmed of Lua's `%s`
  (ASCII whitespace; not Python's `strip()`, which takes Unicode's), each
  then a string: its name a single `Str` holding the spaces (`Str "Jane
  Doe"`), formatting lost. A `MetaString` (`-M author="A, B"`, case
  `metastring`) is not split.
- **An institute by number works only for a string**: `tonumber` of a
  `MetaString`, or of a comma-split piece. YAML never makes a
  `MetaString` (`institute: 1` is `MetaInlines [Str "1"]`), so a lone
  number is an identifier, then a name: a new institute named "1" (case
  `csv`). Lua's `tonumber` takes `" 2 "`, `"1.0"`, `"1e0"`, `"+1"` (case
  `indices`); the ports read decimals as it does, not its hexadecimal
  (`0x1`), as list-table's ports. The table index of a float with an
  integer value is that integer; `2.5`, `0`, `-1` and out-of-range numbers
  find nothing, then fall back to identifiers.
- **A named object**: `MetaInlines` is the name; a string or a boolean is
  named by Lua's `tostring` (`"true"`: case `bools`); a map with `name` is
  copied, its `id` the text of `id` (or of `name` if `id` is missing or
  false: case `shapes`, `Emmy Noether`); a map of one entry is its key and
  its value: a name (`- Grace Hopper: Yale` names her "Yale"), or a map of
  the rest, named by the key if it has no name (a false one counts as
  none: `Hedy Lamarr`); anything else stops the conversion ("not a named
  object": cases `error-empty`, `error-two`; a boolean value in a
  one-entry map, case `error-bool`, fails as `pairs(true)` does).
- **References resolve against the top-level institutes only** (the merge
  comes after): an author's institute given in place by name is a new one
  even if another author gave the same; the merge then keeps the first and
  adds to it what the later lacks, a false entry counting as missing (Lua's
  `b[k] = b[k] or v`; case `merge`, `open: false` becomes `true`).
- **Two top-level institutes with one identifier stay two**; references
  find the first. When an author refers by number to the second, the merge
  copies what the second has into the first (`address` in case `merge`):
  in Lua both tables then hold one value, converted twice on the way back
  to pandoc; in Python, TypeScript and Julia a copy (`deepcopy`, `copy`),
  without which `CHECK_UNIQUE=1` finds the node in two places (it did,
  before the copy). The README's "Lua's elements are values" did not say
  this: here the values are Lua tables, which are references until pandoc
  converts them back.
- **Bug, a known difference: an author or institute given as blocks**
  (`author: |`, case `blocks`), or as a list in a list: the original names
  it after Lua's `tostring` of it, which for blocks is pandoc's Haskell
  `show` (`[Para [Str "Jane",Space,...]]`) and for a list of maps an
  address (`table: 0x...`, a new one each run). The ports fail ("not a
  named object"). `known.tsv`, every language. For upstream: refuse what
  `to_named_object` can't name.
- The globals `key` and `attribs` the original leaks are harmless.

## By language: what the metadata costs

Lines of each function, its signature included, non-blank, comments
out (`RESULTS.md` counts the files); "Lua" is the original's.

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| a field, missing or false (`if x then`, `x or y`) | built in | `given`, 2 lines | 1 | 3 | 3 (methods) |
| a list, whatever the shape (`ensure_list`, with the comma split) | 22 | 11 | 9 | 18 | 7 |
| a named object, by the value's kind (`to_named_object`) | 25 | 28 | 37 | 55 | 28 |
| institutes in two lists at once | tables are references | dicts are | `Map`s are | an `enum Ref { Known(usize), New(Named) }`, a pass for the identifiers first, a `match` in the merge: 19 lines | `Dict`s are |
| writing the maps back | `meta.author = authors` | `MetaList(*map(MetaMap, authors))` | `MetaList(authors.map(Object.fromEntries))` | `MetaList(… .map(MetaValue::MetaMap))` | `meta["author"] = authors` |
| a name from a key (`MetaInlines{Str(k)}`) | 1 | 1 | 1 | 1 (`text_of`) | 1 |
| text (`stringify`, `tostring`) | 1 call | 1 | 1 | 1 | 1 |

- **Python**: `match` on the constructors reads well (`case MetaMap(entries)
  if "name" in entries:`, `case MetaString() | MetaBool():`); the named
  objects are plain `dict[str, MetaValue]`, `MetaMap`'s own content type,
  written back with `MetaMap`. `Meta` (the alias `dict[str, MetaValue]`)
  types the `on_meta` function. `from_python` makes a `MetaList` of a list
  and a `MetaMap` of a dict but refuses inlines (`[Str("x")]`): a name is
  `MetaInlines(Str(key))` by hand (`GAPS.md` 19). `bool(MetaBool(False))`
  is `True`: a port of `if meta.x then` as `if meta.get("x"):` is wrong for
  `x: false`, silently; `given` is a `TypeGuard` (`GAPS.md` 18).
- **TypeScript**: the named objects are `Map<string, MetaValue>`, not plain
  objects: their keys are the document's (`__proto__` is one), and
  panir's maps are prototype-less for that reason, a filter's own objects
  are not; `Object.fromEntries` makes one prototype-less enough for
  `MetaList`'s conversion. `new Map([["name", x], ["id", y]])` takes its
  value type from the first pair (`MetaInlines`), so a typed helper.
  `switch (obj.t)` narrows each case.
- **Rust**: the named objects are `BTreeMap<Text, MetaValue>`, the
  `MetaMap`'s own type, so nothing to convert back. Lua's aliasing is the
  cost: a resolved institute is either an index into the top-level list
  or one of the author's own (`Ref`), the identifiers are taken before the
  merge moves the new ones, and merging the second of two same-named
  top-level institutes into the first clones it. `ctx.fail` from `meta`
  (which returns `()`) needs `ctx.fail::<()>(e);` (`GAPS.md` 24). Text's
  `Borrow<str>` makes `map.get("name")` and `map["id"]` work.
- **Julia**: a method per kind (`named(::MetaInlines)`,
  `named(::Union{MetaString, MetaBool})`, `named(::MetaMap)`,
  `attributes(_, ::MetaMap)`): the shortest port. Named objects are
  `Dict{String, MetaValue}`, references as Lua's tables: the same copy as
  Python's when merging. Setting `meta["author"] = authors` converts a
  vector of `Dict`s, and `[numbers...]` (strings) to `MetaString`s, as Lua
  does. `Named[...]` keeps an empty comprehension's element type.

Differential runs (pandoc's test documents, eight real documents, eight
documents made of the cases with their metadata reshaped,
`assemble.py`): the same AST in every language. Four of the real
documents have an `author` (pandoc's manual and test suite, the textbook,
War and Peace), none an `institute`; one made document fails in the
original and every port alike (a map of two entries without a name).
