# Gaps in panir's filter API

What porting real filters found missing or awkward in
[panir](https://github.com/ickc/panir), each with a proposed change.
Ranked by how many of round 1's 21 filters (`SURVEY.md`) would hit it,
counted from their sources; then by how much it hurts. Each filter's
`NOTES.md` has the details.

A gap stays here once closed, marked so, with what closed it: the list is
also the history of what the ports found. Gaps 1 to 10 were found with
`caps`, `metavars` and `list-table`, and closed in panir by
[ickc/panir#14](https://github.com/ickc/panir/pull/14), which those three
ports now use.

| # | gap | languages | round 1 filters hitting it | status |
|---|---|---|---|---|
| 1 | attributes by key | TypeScript, Rust, Julia; Python lacks `pop` | 5: list-table, track-changes, auto-lang, amsthm, metavars | closed, panir#14 |
| 2 | building nodes with attributes in Rust | Rust | 5: metavars, list-table, count-para, auto-lang, amsthm | closed, panir#14 |
| 3 | a filter function can't fail in Rust | Rust | 4: list-table, metavars, scholarly-metadata, pandoc-quotes | closed, panir#14 |
| 4 | Lua's `return nil` undoes changes; panir keeps them | all (a porting trap) | 1 seen (list-table); any Lua filter that changes, then declines | closed (documented), panir#14 |
| 5 | Rust's context has no metadata | Rust | 1 directly (metavars); common in the wiki's list | closed, panir#14 (but see 11) |
| 6 | a node's constructor name in Rust | Rust | 3: list-table, first-line-indent, amsthm | closed, panir#14 |
| 7 | the walks of Rust and Julia have no depth limit | Rust, Julia | 1 (metavars, on a self-referring field), but any document can trigger it | closed, panir#14 |
| 8 | a value put in a second place must be copied | Python, TypeScript, Julia | 2: metavars, author-info-blocks | documented, panir#14; the check proposed is not made |
| 9 | "the inlines of this block" | Rust | 1: list-table | closed, panir#14 |
| 10 | `ctx.doc` is optional in TypeScript's types | TypeScript | every filter reading the document from a function | closed, panir#14 |
| 11 | inside the metadata, Rust's `ctx.meta()` lacks the field being walked | Rust | 1: metavars (only a field naming itself, an error anyway) | open; low |

Not panir's, but met: Julia has no full Unicode case mapping (`caps`); an
error that shows the offending blocks as markdown needs pandoc's writer,
which is libpandoc's (round 3).

## 1. Attributes by key (closed)

**Closed** in panir#14: Python `Attributes.pop(key, default)`; TypeScript
`getAttribute`, `setAttribute`, `removeAttribute`, `hasAttribute`; Rust
`Attr::get`, `set`, `remove`, `has_class`; Julia an `Attr` by key
(`attr["k"]`, `get`, `haskey`, `pop!`, `delete!`): the first pair with the
key, as pandoc's Lua. list-table's four `pop` helpers are gone; `hasClass`
in TypeScript was not added (`attr.classes.includes(c)`).

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

## 2. Building nodes with attributes in Rust (closed)

**Closed** in panir#14: `Attr::new(identifier, classes, attributes)`
taking `&str`s (`Attr::new("", &[], &[("field", field)])`); no builder
methods, no `Span::new`. metavars' span went from seven lines to one.

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

## 3. A filter function can't fail in Rust (closed)

**Closed** in panir#14: `ctx.fail(error)` (any error, or a string; returns
`None`, so `return ctx.fail(...)`), `apply` returning `Result<(),
FilterError>` with the method, the node's constructor and its path;
`panir::filter` takes a closure returning that `Result`, prints it and
exits 1.

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

## 4. Lua's `return nil` undoes changes; panir keeps them (closed, documented)

**Documented** in panir#14: a "Coming from Lua" section in each language's
README, this trap first (checked with pandoc 3.14: changed attributes,
children and lists are all undone by `return nil`); no API change, as
proposed.

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

## 5. Rust's context has no metadata (closed)

**Closed** in panir#14: `Ctx::meta() -> Option<&Meta>`, `None` in a
fragment; while a metadata value is walked, the *other* fields (gap 11 is
what that does to metavars).

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

## 6. A node's constructor name in Rust (closed)

**Closed** in panir#14: generated `tag(&self) -> &'static str` on the sum
types.

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

## 7. The walks of Rust and Julia have no depth limit (closed)

**Closed** in panir#14: Rust's walk stops at `MAX_DEPTH` (256) levels with
a `FilterError` (function `None`) and the path; Julia's `walk!` with an
`ASTDepthError` at `Panir.MAX_DEPTH` (256). metavars' self-referring field
now stops at once in every language.

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

## 8. A value put in a second place must be copied (documented)

**Documented** in panir#14, no more: each README's "Coming from Lua"
says Lua's elements are values and names the language's copy
(`copy.deepcopy`, `copy`, `deepcopy`, `.clone()`). The proposed check (a
node reachable from two places) and a Python `panir.copy` were not made;
the gap stays open in that sense.

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

## 9. "The inlines of this block" (closed)

**Closed** in panir#14: `Block::inlines()` and `inlines_mut()`, the
inlines of a `Plain`, `Para` or `Header`.

**The original**: list-table looks at `block.content[1]` whatever the
block, to find an empty span opening it.

**panir**: Python and TypeScript can do the same (TypeScript then tells
an inline from a list of blocks); Julia by dispatch on `Union{Plain, Para,
Header}`, short. Rust needs a `match` on the three variants.

**Proposed**: `Block::inlines_mut(&mut self) -> Option<&mut Vec<Inline>>`
(Plain, Para, Header). Low priority.

## 10. `ctx.doc` is optional in TypeScript's types (closed)

**Closed** in panir#14: under `runFilter` `ctx.doc` is typed `Pandoc`;
`applyFilter` on a fragment keeps the optional one.

**panir**: `ctx.doc` is `Pandoc | undefined` (a fragment has none), so a
filter that only runs on documents writes `ctx.doc?.meta`, and handles an
`undefined` that can't happen. Python's is untyped (`Any`), Julia's
`nothing` for a fragment.

**Proposed**: `runFilter`'s functions get a context whose `doc` is a
`Pandoc` (it always runs on a document); `applyFilter` on a fragment
keeps the optional one. Low priority.

## 11. Inside the metadata, Rust's `ctx.meta()` lacks the field being walked

Found bringing metavars onto panir#14.

**The original**: metavars reads `meta[field]` for every `%{field}`,
the metadata's own included; a field naming itself (`loop: "%{loop}"`)
recurses until Python's `RecursionError`.

**panir**: while a metadata value is walked, `ctx.meta()` is the other
fields (the value is out of the map while it changes: it can't be lent
and changed at once). Documented on `Ctx::meta`. For metavars it gives the
right result everywhere but on that error, which in Rust is now `no
metadata field loop`, on `meta.loop.content[0]`, where the other languages
(and the original) say the walk went too deep. A true error, a misleading
message. Python, TypeScript and Julia read `ctx.doc.meta`, the whole map
as the walk has changed it so far.

**Proposed**: no change to what `ctx.meta()` holds (`None` inside the
metadata would lose metavars' `subtitle: "%{author}"`, which works now);
say it in the Rust README's "Coming from Lua" next to `ctx.meta()`, where
a port looks: "inside the metadata, the field being walked is not in
it". Perhaps also `Ctx::meta_key() -> Option<&str>`, the top-level field
being walked, so a filter can tell "missing" from "the one I am in".
Low priority.

How common: 1 of 21 (metavars), and only on a document that is an error
anyway.

