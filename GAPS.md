# Gaps in panir's filter API

What porting real filters found missing or awkward in
[panir](https://github.com/ickc/panir), each with a proposed change.
Ranked by how many of round 1's 21 filters (`SURVEY.md`) would hit it,
counted from their sources; then by how much it hurts. Found so far with
`caps`, `metavars` and `list-table`; each filter's `NOTES.md` has the
details.

| # | gap | languages | round 1 filters hitting it |
|---|---|---|---|
| 1 | attributes by key | TypeScript, Rust, Julia; Python lacks `pop` | 5: list-table, track-changes, auto-lang, amsthm, metavars |
| 2 | building nodes with attributes in Rust | Rust | 5: metavars, list-table, count-para, auto-lang, amsthm |
| 3 | a filter function can't fail in Rust | Rust | 4: list-table, metavars, scholarly-metadata, pandoc-quotes |
| 4 | Lua's `return nil` undoes changes; panir keeps them | all (a porting trap) | 1 seen (list-table); any Lua filter that changes, then declines |
| 5 | Rust's context has no metadata | Rust | 1 directly (metavars); common in the wiki's list |
| 6 | a node's constructor name in Rust | Rust | 3: list-table, first-line-indent, amsthm |
| 7 | the walks of Rust and Julia have no depth limit | Rust, Julia | 1 (metavars, on a self-referring field), but any document can trigger it |
| 8 | a value put in a second place must be copied | Python, TypeScript, Julia | 2: metavars, author-info-blocks |
| 9 | "the inlines of this block" | Rust | 1: list-table |
| 10 | `ctx.doc` is optional in TypeScript's types | TypeScript | every filter reading the document from a function |

Not panir's, but met: Julia has no full Unicode case mapping (`caps`); an
error that shows the offending blocks as markdown needs pandoc's writer,
which is libpandoc's (round 3).

## 1. Attributes by key

**The original**: Lua reads, sets and removes attributes by key
(`el.attributes.colspan`, `el.attributes.colspan = nil`); list-table reads
and removes seven, track-changes reads `author` and others, auto-lang
`lang`, amsthm `info` and its own; metavars sets two (and pandocfilters'
latexdivs, not taken, tests `["latex", "true"] in kvs`).

**panir**: Python has `attributes["k"]`, `.get`, `del`, `in`, but no
`pop(key, default)`, so read-and-remove is three lines (list-table's
ports all grew a `pop` helper). TypeScript (`[string, string][]`), Rust
(`Vec<(Text, Text)>`) and Julia (`Vector{Tuple{String,String}}`) have the
pairs only: `findIndex`/`splice`, `iter().position`/`remove`,
`findfirst`/`popat!`, written in every filter. Constructors already take a
mapping in all four (`attributes: { k: "v" }`, `"k" => "v"`), so only
reading and changing are missing.

**Proposed**, keeping pairs (order, repeated keys) as the data:

- Python: `Attributes.pop(key, default)` (and `setdefault`), as `dict`.
- TypeScript (nodes are plain objects, so functions): `getAttribute(attr,
  key)`, `setAttribute(attr, key, value)`, `removeAttribute(attr, key)`
  returning the value; also `hasClass(attr, c)`.
- Rust: `Attr::get(&self, key) -> Option<&str>`, `Attr::set(&mut self, key,
  value)`, `Attr::remove(&mut self, key) -> Option<Text>`,
  `Attr::has_class`.
- Julia: `attributes` as a small vector type of pairs with `getindex` by
  key, `get`, `haskey`, `setindex!`, `delete!`, `pop!(a, key, default)`
  (Python's semantics: the first pair with the key).

How common: 5 of 21 in round 1, and in most filters of later rounds
(include-files, diagram, pantable read options from attributes).

## 2. Building nodes with attributes in Rust

**The original**: `pandoc.Span(content, {class = ..., field = ...})`.

**panir**: Python, TypeScript and Julia constructors take `identifier=`,
`classes=`, `attributes=` (a mapping). Rust has struct literals only:
metavars' span is `Span { attr: Attr { attributes: vec![("class".into(),
"interpolated".into()), ("field".into(), field.into())],
..Attr::default() }, content }.into()`, the longest line of the port.

**Proposed**: `Attr::new(identifier, classes, attributes)` taking
`impl IntoIterator` of `impl Into<Text>` (pairs for attributes), and
builder methods (`Attr::default().with_class("x").with_attribute("k",
"v")`); perhaps `Span::new(attr, content)` for the boxed constructors.

How common: every filter that makes a span, div or cell with attributes,
5 of 21.

## 3. A filter function can't fail in Rust

**The original**: Lua `error(...)`/`assert`, Python's exceptions: the
conversion stops with a message (list-table on malformed tables, metavars
on a missing field, scholarly-metadata and pandoc-quotes on bad metadata).

**panir**: Python, TypeScript and Julia let the exception through, and
add where it happened (the function and the node's path). Rust's methods
return `Option<Vec<_>>`: the ports `panic!` (metavars) or print and
`std::process::exit(1)` (list-table), with no path.

**Proposed**: `ctx.fail(error)` (or methods returning `Result`) that stops
the walk; `apply` returning `Result<(), panir::FilterError>` with the
node's path, as the other languages report; `panir::filter` printing it
and exiting 1.

How common: 4 of 21.

## 4. Lua's `return nil` undoes changes; panir keeps them

**The original**: list-table removes the div's first class, then returns
`nil` for an empty div. In pandoc's Lua a function's changes to its
element count only if it returns the element: `nil` keeps the original.

**panir**: in every language, a function returning nothing keeps the node
"changed in place or not", as documented. A port that follows the Lua
line by line removes the class for good (the case `edges` catches it).

**Proposed**: no API change; a "coming from Lua" paragraph in each
README: return nothing only when nothing was changed; decide first, change
after. (A strict mode that checks a node is unchanged when nothing is
returned would cost a copy per call: not worth it.)

How common: hard to count from sources; one of the twelve Lua filters so
far.

## 5. Rust's context has no metadata

**The original**: metavars reads the metadata in every `Str` function;
pandocfilters passes `meta` to every call, panflute `doc`, Lua filters
read a global set in an earlier `Meta` function.

**panir**: Python, TypeScript and Julia have `ctx.doc`. Rust's `Ctx` has
the conversion only (a function holds its node `&mut`), so the port
clones the whole metadata before the walk, in `main`.

**Proposed**: `Ctx::meta() -> Option<&Meta>`, `None` while the metadata
itself is walked: the driver can lend `&doc.meta` while it walks
`&mut doc.blocks`, as they are separate fields.

How common: 1 in round 1 directly (the others read the metadata in a
`Pandoc` or `Meta` function first); a common kind in the wiki's list (placeholders, abbreviations, acronyms, mustache).

## 6. A node's constructor name in Rust

**The original**: `el.t` (or `el.tag`): in messages (list-table's
"expected bullet list, found Para"), and as data: first-line-indent's
metadata lists block types by name (`remove_indent_after: [BlockQuote,
...]`), amsthm looks types up in a table by name.

**panir**: Python `type(x).__name__`, TypeScript `x.t`, Julia
`nameof(typeof(x))`. Rust has none: the port cuts it out of `Debug`'s
output.

**Proposed**: generated `tag(&self) -> &'static str` on `Block`, `Inline`
and `MetaValue` (and the reverse not needed).

How common: 3 of 21.

## 7. The walks of Rust and Julia have no depth limit

**The original**: metavars with a field naming itself walks its own
output for ever; Python stops with `RecursionError`.

**panir**: Python stops with its depth error and the path, TypeScript at
once too. Rust overflows the stack and aborts (8 MB stack), or, with no
stack limit, took 59 GB in two minutes; Julia was still running after
60 s (8 MB stack), or took 5.7 GB in 30 s. A document chooses its
metadata, so a document can do this to a Rust or Julia filter.

**Proposed**: count the levels the walk is in (as reading does: Rust's
`MAX_DEPTH`, 256) and stop with an error naming the path, in every order;
in Rust through the error of gap 3.

How common: any top-down filter whose replacement contains what it
matches; rare in intent, but a hostile document finds it.

## 8. A value put in a second place must be copied

**The original**: Lua and pandocfilters copy implicitly (Lua's elements
are values; pandocfilters builds new JSON). metavars puts a metadata
field's inlines into the body; author-info-blocks puts the authors'
names (metadata) into the body and keeps them in the metadata too.

**panir**: a node belongs in one place; to put it in a second, copy it,
"or whatever changes one later changes both". Python uses
`copy.deepcopy`, TypeScript `copy`, Julia `deepcopy`, Rust `clone`.
Forgetting is silent: the output is right until something changes one of
the two.

**Proposed**: make the mistake visible rather than the copy implicit: a
check (`check(doc)`, or under an environment variable in `main`) that
reports a node reachable from two places, with both paths; in Python, a
`panir.copy` for symmetry with TypeScript.

How common: 2 of 21.

## 9. "The inlines of this block"

**The original**: list-table looks at `block.content[1]` whatever the
block, to find an empty span opening it.

**panir**: Python and TypeScript can do the same (TypeScript then tells
an inline from a list of blocks); Julia by dispatch on `Union{Plain, Para,
Header}`, short. Rust needs a `match` on the three variants.

**Proposed**: `Block::inlines_mut(&mut self) -> Option<&mut Vec<Inline>>`
(Plain, Para, Header). Low priority.

## 10. `ctx.doc` is optional in TypeScript's types

**panir**: `ctx.doc` is `Pandoc | undefined` (a fragment has none), so a
filter that only runs on documents writes `ctx.doc?.meta`, and handles an
`undefined` that can't happen. Python's is untyped (`Any`), Julia's
`nothing` for a fragment.

**Proposed**: `runFilter`'s functions get a context whose `doc` is a
`Pandoc` (it always runs on a document); `applyFilter` on a fragment
keeps the optional one. Low priority.
