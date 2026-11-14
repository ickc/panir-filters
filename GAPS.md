# Gaps in panir's filter API

What porting real filters found missing or awkward in
[panir](https://github.com/ickc/panir), each with a proposed change.
How many of round 1's 21 filters (`SURVEY.md`) would hit each, counted
from their sources, ranks them, then how much it hurts; they are numbered
in the order found. Open, by rank: 12, 13, then 11, 14, 15, 16 (low). Each
filter's
`NOTES.md` has the details.

A gap stays here once closed, marked so, with what closed it: the list is
also the history of what the ports found. Gaps 1 to 10 were found with
`caps`, `metavars` and `list-table`, and closed in panir by
[ickc/panir#14](https://github.com/ickc/panir/pull/14), which those three
ports now use. Gap 11 was found bringing them onto it; 12 to 16 with the
second batch (`deemph`, `comments`, `deflists`, `theorem`, `inlinenotes`,
`gitlab_markdown`, `french-spaces`), which used little of panir#14's API
(`has_class`, `ctx.fail`) as none of them reads attributes or the
metadata.

| # | gap | languages | round 1 filters hitting it | status |
|---|---|---|---|---|
| 1 | attributes by key | TypeScript, Rust, Julia; Python lacks `pop` | 5: list-table, track-changes, auto-lang, amsthm, metavars | closed, panir#14 |
| 2 | building nodes with attributes in Rust | Rust | 5: metavars, list-table, count-para, auto-lang, amsthm | closed, panir#14 |
| 3 | a filter function can't fail in Rust | Rust | 4: list-table, metavars, scholarly-metadata, pandoc-quotes | closed, panir#14 |
| 4 | Lua's `return nil` undoes changes; panir keeps them | all (a porting trap) | 1 seen (list-table); any Lua filter that changes, then declines | closed (documented), panir#14 |
| 5 | Rust's context has no metadata | Rust | 1 directly (metavars); common in the wiki's list | closed, panir#14 (but see 11) |
| 6 | a node's constructor name in Rust | Rust | 3: list-table, first-line-indent, amsthm | closed, panir#14 |
| 7 | the walks of Rust and Julia have no depth limit | Rust, Julia | 1 (metavars, on a self-referring field), but any document can trigger it | closed, panir#14 |
| 8 | a value put in a second place must be copied | Python, TypeScript, Julia | 2: metavars, author-info-blocks | closed: documented in panir#14; `check_unique` to call in tests, panir#15 |
| 9 | "the inlines of this block" | Rust | 1: list-table | closed, panir#14 |
| 10 | `ctx.doc` is optional in TypeScript's types | TypeScript | every filter reading the document from a function | closed, panir#14 |
| 11 | inside the metadata, Rust's `ctx.meta()` lacks the field being walked | Rust | 1: metavars (only a field naming itself, an error anyway) | closed, panir#15: said in the README (walking a copy costs more than it gives) |
| 12 | a sub-walk from a Rust filter function: the conversion and the error by hand | Rust | 6: deemph, pagebreak, abstract-section, cito, track-changes, amsthm | closed, panir#15: `ctx.apply(x, f)` |
| 13 | what a top-down function's replacement gets walked is not said | all (documentation) | 6: deemph, theorem, metavars, comments, amsthm, auto-lang | closed, panir#15: said in each README, pinned by a scenario |
| 14 | Rust's `Filter` has no function for metadata values | Rust | 1: comments (a `MetaList`'s items) | closed, panir#15: `meta_value`, `ctx.in_list()` |
| 15 | `[]` for a metadata value in a map: Python deletes the key, TypeScript and Julia refuse | Python, TypeScript, Julia | 1: comments | closed, panir#15: an error in every language |
| 16 | TypeScript's `ctx.parent` is typed `ASTNode`: `ctx.parent?.t` doesn't type-check | TypeScript | 1: comments | closed, panir#15: typed by the function |

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

## 8. A value put in a second place must be copied (closed: documented, a check to call)

**Documented** in panir#14: each README's "Coming from Lua" says Lua's
elements are values and names the language's copy (`copy.deepcopy`,
`copy`, `deepcopy`, `.clone()`). **A check to call** in panir#15:
`check_unique(x)` (Python, Julia), `checkUnique(x)` (TypeScript) raises
if the same node object is reachable from two places, naming both paths.
Nothing calls it: a check on every write cost 45 to 124% of the write
time, and a filter's correctness is its author's. Here `CHECK_UNIQUE=1
python3 run.py test` calls it on every port's output; no port fails it.

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

## 11. Inside the metadata, Rust's `ctx.meta()` lacks the field being walked (closed: documented)

**Closed** in panir#15 by saying it in the Rust README. Walking a copy of the field to give the whole metadata was measured: 3.8 ms of copies against a 0.9 ms walk with 1,000 references in the metadata; not built.

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

## 12. A sub-walk from a Rust filter function: the conversion and the error by hand (closed)

**Closed** in panir#15: `ctx.apply(x, f) -> Option<()>` walks with the walk's conversion and depth and records a failure as `ctx.fail` does, so `ctx.apply(content, &mut Caps)?;`. The error is the inner function's, with its path inside the fragment, as Python and TypeScript report it.

Found with deemph.

**The original**: deemph walks an emphasis's content with a second
filter (`walk(val, caps, fmt, meta)`), from inside its function; Lua's
`el:walk{...}`, panflute's `elem.walk(...)` are the same.

**panir**: Python (`emph.content.walk(caps)`), TypeScript
(`applyFilter(emph.content, caps)`) and Julia (`walk!(caps,
emph.content)`) take one line, which returns the list, and an error goes
up as any exception. Rust takes six: `panir::apply_with(&mut content, &mut
Caps, ctx.conversion())`, whose `Result` the function turns into
`ctx.fail(e)` with a `match`, as a filter function returns an `Option`
and can't use `?` on it. Forgetting the conversion (`apply(..., None)`)
gives the sub-walk no format, silently.

**Proposed**: `Ctx::apply<T: Walkable, F: Filter>(&mut self, x: &mut T,
f: &mut F) -> Option<()>`: walks with the context's conversion and depth;
on failure records the error as `fail` does (the inner path kept in the
message) and returns `None`, so that a function writes
`ctx.apply(&mut content, &mut Caps)?;`.

How common: every filter with `subwalk` in `SURVEY.md`: 6 of 21.

## 13. What a top-down function's replacement gets walked is not said (closed)

**Closed** in panir#15: each README says it, and the scenario `replaced` pins it. pandoc's Lua does the same: what a function returns top-down is given to no function, and its children are walked.

Found with deemph and theorem.

**The original**: pandocfilters calls the action on a node, then walks
the fields of what it returned, not the returned node itself; of a
returned list, neither are its elements given to the action, only their
fields. deemph and theorem depend on it: an emphasis directly in another,
spliced in, stays an emphasis; in LaTeX, a theorem directly in a theorem
stays a div (and the HTML branch, returning a div of the same class, would
otherwise number itself without end).

**panir**: does exactly this in every language (tested in Python: a
spliced emphasis's directly nested emphasis is not given to the function,
one in a strong is), so the ports match. But each README says only that
a node's children "are walked in the node's replacement too", which can
be read either way; a port had to test it.

**Proposed**: say it in each README's `topdown` paragraph: "what a
function returns is not given to the function again (nor, if a list, its
elements); their children are", and a corpus scenario that pins it
(pandoc's Lua should agree: check it when writing the scenario).

How common: the top-down filters of round 1, 6 of 21.

## 14. Rust's `Filter` has no function for metadata values (closed)

**Closed** in panir#15: Rust's `Filter::meta_value`, as the other three languages have (pandoc's Lua has none; panflute has), and `ctx.in_list()` to tell a `MetaList`'s item from a map's value. The scenario `metavalues` pins the four.

Found with comments.

**The original**: comments deletes, while hiding, every node in a list,
so the items of a `MetaList` after a beginning in the metadata.

**panir**: Python, TypeScript and Julia have functions on `MetaValue`
(by type, or for each constructor). Rust's `Filter` has `inline`,
`block`, `inlines`, `blocks`, `meta` and `pandoc` only, as pandoc's Lua
(which calls no function for a metadata value: checked with pandoc 3.14).
The Rust port empties those items instead of deleting them: a known
difference (case `meta`).

**Proposed**: `fn meta_value(&mut self, x: &mut MetaValue, ctx: &mut Ctx)
-> Option<Vec<MetaValue>>`, called as the other languages call theirs;
spliced in a list, one value only in a map (gap 15). Or leave it, as Lua
does, and say in the Rust README which functions the other languages'
filters have that `Filter` doesn't. Low priority.

How common: 1 of 21, and only on a beginning marker in the metadata.

## 15. `[]` for a metadata value in a map: Python deletes the key, TypeScript and Julia refuse (closed)

**Closed** in panir#15: an error in every language, saying to delete the key in the function of the `Meta` or `MetaMap` holding it (scenario `metadelete`).

Found with comments.

**panir**: a `MetaValue` function returning `[]` for a value in a list
deletes it, in every language. For a value in a map (a field of the
metadata, an entry of a `MetaMap`), Python deletes the key, silently;
TypeScript and Julia raise ("expected one MetaValue (this position holds
one), got []"). pandoc's Lua has no such functions to follow (gap 14).
The comments port in Python had to check `ctx.parent` to delete only in a
list, as the original does; in TypeScript and Julia the same check avoids
an error.

**Proposed**: one rule in every language; the TypeScript and Julia one
(a map's position holds one value: `[]` is an error there, return
`nothing` to keep it, and delete a key from the `Meta` or `MetaMap`
function). A corpus scenario to pin it.

How common: 1 of 21.

## 16. TypeScript's `ctx.parent` is typed `ASTNode` (closed)

**Closed** in panir#15: `ctx.parent` is typed by the function it is in (the nodes that can hold that node), so `ctx.parent?.t === "MetaList"` type-checks and narrows.

Found with comments.

**panir**: `ctx.next` and `ctx.prev` are typed by the function's node
(`Inline` for an inline's), but `ctx.parent` is `ASTNode`, which includes
the products without `t` (`Attr`, `Cell`...), so `ctx.parent?.t ===
"MetaList"` doesn't type-check in a `MetaValue` function. The port uses
`Array.isArray(ctx.container)` ("in a list"), which does and says what
the original means.

**Proposed**: type `parent` as the nodes that can hold the function's
node where the schema says (a `MetaValue`'s parent is a `MetaList`, a
`MetaMap` or none), or at least a union of the `t`-tagged ones. Low
priority.

How common: 1 of 21.

