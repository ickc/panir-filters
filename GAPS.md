# Gaps in panir's filter API

What porting real filters found missing or awkward in
[panir](https://github.com/ickc/panir), each with a proposed change.
How many of round 1's 21 filters (`SURVEY.md`) would hit each, counted
from their sources, ranks them, then how much it hurts; they are numbered
in the order found. Gaps 1 to 25 are closed (20 and 25 in part, as
panir declined the rest); none is open. Each filter's `NOTES.md` has the
details.

A gap stays here once closed, marked so, with what closed it: the list is
also the history of what the ports found. Gaps 1 to 10 were found with
`caps`, `metavars` and `list-table`, and closed in panir by
[ickc/panir#14](https://github.com/ickc/panir/pull/14), which those three
ports now use. Gap 11 was found bringing them onto it; 12 to 16 with the
second batch (`deemph`, `comments`, `deflists`, `theorem`, `inlinenotes`,
`gitlab_markdown`, `french-spaces`), which used little of panir#14's API
(`has_class`, `ctx.fail`) as none of them reads attributes or the
metadata. 17 to 25 with the third, the metadata cluster
(`scholarly-metadata`, `author-info-blocks`, `pandoc-quotes`,
`abstract-section`, `cito`), the first batch of Lua originals: what Lua's
plain metadata values cost in panir's typed ones (below, "Metadata, by
language"), and what each "Coming from Lua" says or doesn't; closed by
[ickc/panir#16](https://github.com/ickc/panir/pull/16), which those five
ports now use (`run.py test`, `CHECK_UNIQUE=1` and the differential runs
the same).

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
| 17 | Lua's metadata, as panir has it, is not said: what each Lua value is here, its fields, `pandoc.utils.type`, tables as references | all (documentation) | 11: every Lua original but list-table | closed, panir#16: a "Metadata" part in each "Coming from Lua" |
| 18 | Lua's `if meta.x then` (set and not false): every port writes it, and a node is always true in Python and TypeScript | Python, TypeScript, Julia (Rust matches) | 9: scholarly-metadata, author-info-blocks, pandoc-quotes, abstract-section, pagebreak, track-changes, first-line-indent, amsthm, auto-lang | closed, panir#16: Python `bool()`, `truthy` in the others |
| 19 | Python's `from_python` refuses inlines and blocks, which Lua, TypeScript and Julia make `MetaInlines` and `MetaBlocks` | Python | 7: scholarly-metadata, author-info-blocks, abstract-section, track-changes, first-line-indent, count-para, amsthm | closed, panir#16 |
| 20 | several filters as one program: Python's `main` and Julia's `run_filter` take one | Python, Julia | 4: pandoc-quotes, count-para, auto-lang, amsthm | closed, panir#16 (Python, Julia); Rust: apply after apply, shown |
| 21 | a nested option: no lookup by path | all | 4: abstract-section, pagebreak, first-line-indent, amsthm | closed, panir#16: `lookup` |
| 22 | one value or a list of them (Lua's `ensure_list`, `header-includes` "whatever shape"): one arm per constructor | all (Rust most) | 7: scholarly-metadata, pandoc-quotes, abstract-section, track-changes, first-line-indent, count-para, amsthm | closed, panir#16: `items` |
| 23 | Rust has no metadata from plain values | Rust | 3: cito, scholarly-metadata, first-line-indent | closed, panir#16 |
| 24 | Rust: `ctx.fail` and `ctx.apply` from a function returning `()` (`meta`, `pandoc`, list functions) | Rust | 6: scholarly-metadata, author-info-blocks, pandoc-quotes, abstract-section, pagebreak, track-changes | closed, panir#16 |
| 25 | Rust's `Link` and `Target` are built field by field | Rust | 2: author-info-blocks, count-para | closed, panir#16: `Target::new`; no `Default` (declined) |

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

## Metadata, by language (the third batch)

In Lua, pandoc's metadata is plain Lua values: a `MetaMap` a table, a
`MetaList` a `List`, `MetaInlines` `Inlines`, `MetaBlocks` `Blocks`, a
`MetaString` a string, a `MetaBool` a boolean, a missing field `nil`
(checked with pandoc 3.14; since pandoc 2.17 none has a `t`:
pandoc-quotes' original fails on that). `pandoc.utils.type(v)` tells them
apart, Lua's truth (`if v then`, `v or default`) is "set and not false",
`ipairs` walks a list, inlines and blocks alike, and a table assigned back
becomes a `MetaMap` (or a `MetaList`). panir has a typed node for each.
What the common operations cost in the five ports (lines of each piece,
its signature included, comments out; "a field, missing or false" counts
the helper each port wrote), as found, then what was proposed and what
the ports are after panir#16 (`run.py test` and the differential runs
the same):

| operation | Lua | Python | TypeScript | Rust | Julia | proposed | after panir#16 |
|---|---|---|---|---|---|---|---|
| a field, with a default if missing | `meta.x or d` | `meta.get("x", d)` | `meta.x ?? d` | `meta.get("x")`, an `Option` | `get(meta, "x", d)` | nothing: the same | the same |
| set and not false (Lua's `if meta.x`) | built in | a 2-line `given` in each of four ports; `if meta.get("x"):` is wrong (`bool(MetaBool(False))` is `True`), silently | 1 line, four times; `if (meta.x)` is wrong, silently | `matches!` (1–3), four times | 3 one-line methods, four times; `if MetaBool(false)` throws | a `truthy(value)` in each language, and a line in "Coming from Lua" (gap 18) | no helper left: Python `if meta.get("x"):` and `or`, as Lua; `truthy(...)` in the others (TypeScript's not a type guard: a `!` after it, once) |
| a nested option (`(meta.a or {}).b`) | 1 | 2 (`isinstance`) to 7 (a `match` that also fails on `true`, as Lua) | 1–3 | 3–5 (`match`) | 2–3 (methods) | `get(meta, "a", "b")`: through maps, `None` if not there (gap 21) | `lookup(meta, "a", "b")`, 1 (Rust `lookup(&meta, &["a", "b"])`, from the metadata only); abstract-section's test for `true` stays |
| one value or a list of them (scholarly-metadata's `ensure_list`, with its comma split) | 22 | 11 | 9 | 18 | 7 | `items(value)`: a list's items, else the value alone, none if missing (gap 22) | 6, 7, 10, 4: the comma split, then `items`; author-info-blocks' `maps` 8, 5, 12, 1 (from 11, 9, 24, 3) |
| a string, a list of strings, or inlines, to texts (pandoc-quotes' marks; abstract-section's identifiers with their configuration) | 19; 8 | 11; 16 | 8; 17 | 10; 19 (an arm per constructor: their items differ in type) | 4; 8 | `items` and `stringify` make most of it one line (each filter's quirk aside: four characters, an identifier per inline) | 11; 13, 8; 16, 10; 17, 4; 8: `items` doesn't fit these (Lua's `ipairs` takes inlines as a list; a non-empty string is no identifier), `lookup` saves the nested option |
| a value's kind (`pandoc.utils.type`) | `type(v) == 'Inlines'` | `match`, `isinstance` | `v.t`, narrowing | `match` | dispatch | nothing: each language's own is better than Lua's strings; say which is which (gap 17) | the same; each README's table says which is which |
| metadata from plain values (cito's map of lists of strings) | `meta.x = t`, 1 | `doc.meta["x"] = by_property`, 1 (but not inlines: gap 19) | `MetaMap(Object.fromEntries(m))`, 1 | 14 (rustfmt's) of nested constructors | `doc.meta["x"] = by_property`, 1 | `From` and `FromIterator` for `MetaValue` (gap 23) | Rust 4 (`collect::<MetaValue>()` inside, `collect()` outside); Python takes inlines and blocks |
| a map worked on, then written back (a named object) | a table | `dict[str, MetaValue]` (`MetaMap`'s content), `MetaMap(d)` | a `Map` (its keys are the document's: `__proto__` too), `Object.fromEntries` | `BTreeMap<Text, MetaValue>`, `MetaMap`'s own: nothing to convert | `Dict{String, MetaValue}`, `MetaMap`'s own; `meta["x"] = [dicts...]` converts | nothing | Python `meta["x"] = [dicts...]` converts too; Rust `collect()` of the maps |
| one value in two maps (Lua copies when it converts back) | built in | `deepcopy` (found by `check_unique`) | `copy` (same) | `clone`, and an `enum Ref { Known, New }` with a pass for the identifiers: 19 lines | `deepcopy` | a line in "Coming from Lua" (gap 17) | the same, and said in each README |
| metadata to text (`pandoc.utils.stringify`, `tostring` of a string or a boolean) | 1 | 1 | 1 | 1 | 1 | nothing: `stringify` is exact in every language | the same |
| metadata to plain values | (it is) | `to_python` | `toPlain` | none | none | nothing: both turn inlines into text, so no port could use them (a name keeps its emphasis); Python's, TypeScript's and Julia's maps and lists are already plain containers of nodes | `to_python` and `toPlain` keep inlines and blocks as lists of them (gap 19); still no port needs them |

So, as found: the metadata costs least in Julia (dispatch on the
constructors, conversion on assignment, `Dict`s that are `MetaMap`'s own:
scholarly-metadata is 77 lines to Lua's 129) and TypeScript (narrowing,
conversion in the constructors), more in Python (`match` is good, but
`ruff format` spreads it, and there is no conversion of inlines), most in
Rust, where plain values have no conversion (cito's one Lua line is 14)
and Lua's aliasing of tables needs indices (scholarly-metadata's `Ref`).
In every language the one trap is Lua's truth, which no README mentions
and every port reimplemented (18); the one missing piece of plain reading
is the nested option (21) and "one or a list" (22).

After panir#16 the four ports it touched lost 3 to 12% of their lines in
Python, 1 to 8% in TypeScript, 2 to 10% in Rust (cito's too), 5 to 12% in
Julia (`RESULTS.md`), all
sixteen `given`s and the four `items`-like helpers; scholarly-metadata in
Julia is 71 lines. Rust's cost is now the aliasing alone. What the fourth
batch, which reads options and writes `header-includes`, found of this API
in use will be in the section after this one.

## The API of panir#14 and #15, used here

- **Attributes by key**: not used: these filters read no attribute
  (abstract-section reads a heading's identifier).
- **Rust's `ctx.fail`**: used by four ports, all from `meta` or `pandoc`,
  which return `()`: `ctx.fail::<()>(e);`, a turbofish and a `return`
  after; the README's "`return ctx.fail(...)`" holds only in `inline`,
  `block` and `meta_value` (gap 24).
- **Rust's `ctx.apply`**: cito's `inline` function, `ctx.apply(&mut
  cite.content, &mut Rewrite { .. })?`, reads as Lua's
  `cite.content:walk{...}`; abstract-section's, from `pandoc`, can't use
  `?` (gap 24).
- **Rust's `ctx.meta()`, `meta_value`, `ctx.in_list()`**: not needed:
  these filters read the metadata in their `meta` or `pandoc` function.
- **`Attr::new`**: not used (`Attr::default()` for a link's).
- **`tag()`**: author-info-blocks names a wrong kind in its messages with
  `MetaValue::tag`, as Lua's `pandoc.utils.type`.
- **TypeScript's typed `ctx.parent`, `ctx.doc`**: not used: the `Pandoc`
  and `Meta` functions get the document or the metadata. `ctx.format` is
  `string | undefined`, so `ctx.format?.includes("latex")` where Lua's
  `FORMAT:match` needs nothing.
- **`check_unique`** (`CHECK_UNIQUE=1`): found one place a copy was
  missing (scholarly-metadata's merge, below in 17), and agrees with
  author-info-blocks' copies.

## 17. Lua's metadata, as panir has it, is not said (closed)

**Closed** in panir#16: each README's "Coming from Lua" has a
"Metadata" part: a table of what a field is in the document, in Lua
(`pandoc.utils.type`) and here (the node and its field), Lua's truth,
the nested option, one value or a list, writing metadata, Lua's tables
as references; and lists the field names that are not Lua's (Python's
"as in pandoc's Lua API" is gone), `FORMAT`, a list of filters, Lua's
string patterns.


Found with scholarly-metadata, author-info-blocks and pandoc-quotes.

**The original**: reads the metadata as the plain values above, and their
types with `pandoc.utils.type` (scholarly-metadata) or `.t`
(pandoc-quotes, broken since pandoc 2.17).

**panir**: each language's "Coming from Lua" says one thing about
metadata: "pandoc's Lua has no function for a metadata value (its values
are plain Lua values, reached through the `Meta` function or
`doc.meta`)". Nothing says what those plain values are here: which node
is a table, a `List`, `Inlines`, a string; what their fields are called
(`MetaMap.content`, `MetaList.content`, `MetaInlines.content`,
`MetaString.text`, `MetaBool.value`: found in Python's `_types.py` and
Julia's `generated.jl`, as pandoc's Lua, which the READMEs give as the
names' source, has no such nodes); that Lua's truth is not the
language's (18); that a Lua table is a reference until pandoc converts it
back, so a value in two tables is two values after (scholarly-metadata's
merge: Python, TypeScript and Julia shared one node until a copy,
found by `check_unique`; "Lua's elements are values" says it of
elements, not of the tables holding metadata). Also missing for a Lua
port: Lua's `FORMAT` is `ctx.format` (which may be `None`, `undefined`,
`nothing`); a Lua filter file returning a list of filters (20); Lua's
string patterns are not regular expressions (cito and pandoc-quotes use
them, unescaped: their bugs). And one sentence misleads: Python's "Fields
have pandoc-types' names and order, as in pandoc's Lua API": Lua's
`Quoted` has `quotetype`, Python's `quote_type` (TypeScript's
`quoteType`).

**Proposed**: in each "Coming from Lua", a paragraph on metadata: a
six-row table (Lua's value, `pandoc.utils.type`, panir's node and its
field, how to test for it in the language); Lua's truth and its helper
(18); tables as references; `FORMAT`; and the field names of the
`MetaValue` constructors in "The AST".

How common: every Lua original that reads or writes the metadata, 11 of
21 (all but list-table).

## 18. Lua's `if meta.x then`: set and not false (closed)

**Closed** in panir#16, not as proposed in Python: a metadata node's
`bool()` is Lua's truth (`MetaValue.__bool__` true, `MetaBool`'s its
value), so `if meta.get("x"):` and `meta.get("x") or d` are the Lua
lines as written; `truthy(v)` in TypeScript (a `boolean`, not a type
guard), Rust (`truthy(Option<&MetaValue>)`) and Julia. The scenario
`truthy` pins the four. All sixteen `given`s are gone.


Found with scholarly-metadata.

**The original**: `if meta['quot-marks'] then`, `named.id or named.name`,
`b[k] = b[k] or v`, `author.equal_contributor and ...`: a field counts if
it is there and not `false`; `""`, an empty list, the text `no` count.

**panir**: no help, and the obvious port is wrong in two languages:
Python's nodes are always true (`bool(MetaBool(False))` is `True`), so
`if meta.get("x"):` treats `x: false` as set, silently; TypeScript's too
(`if (meta.x)`); Julia's `if` throws on a node, which at least is loud;
Rust has to `match`. Every port wrote the same helper (`given`): 2 lines
in Python (a `TypeGuard`), 1 in TypeScript, 3 one-line methods in Julia,
`matches!` in Rust; sixteen copies in four filters.

**Proposed**: a function in each language, `truthy(value)` (Python
`panir.truthy`, TypeScript `truthy`, Rust `MetaValue::truthy(Option<&MetaValue>)`
or a function, Julia `Panir.truthy`): `false` for a missing value and a
`MetaBool(false)`, `true` otherwise, as Lua; and the line in "Coming
from Lua" (17). Not `__bool__` on `MetaBool`: a node's truth would then
depend on its kind, and `MetaList` already has a length.

How common: 9 of 21: the four Lua originals here that read metadata, and
pagebreak (`config['html-class'] and ...`), track-changes, first-line-
indent, amsthm, auto-lang (`meta["auto-lang"] or {}`).

## 19. Python's `from_python` refuses inlines and blocks (closed)

**Closed** in panir#16: `from_python` makes `MetaInlines` of an inline
or a list of them and `MetaBlocks` of blocks; `to_python` and `toPlain`
give inlines and blocks as lists of them (not their text).


Found with scholarly-metadata.

**The original**: Lua makes `MetaInlines` of a list of inlines and
`MetaBlocks` of a list of blocks when it converts a value back (checked:
`meta.x = {pandoc.Str 'a'}` is `MetaInlines`).

**panir**: TypeScript's constructors and `fromPlain`, and Julia's
`convert`, do the same; Python's `from_python` (which setting a field
uses) makes a `MetaList` of any list and refuses its inlines: `doc.meta[
"x"] = [Str("a")]` is "expected MetaValue, got Str". So
`MetaInlines(Str(key))` by hand. Cheap, but a difference between the
languages and from Lua.

**Proposed**: `from_python` converts an inline, or a list of inlines, to
`MetaInlines`, and a block, or a list of blocks, to `MetaBlocks`, as the
other languages and Lua; a list of other values stays a `MetaList`.

How common: 7 of 21 put inlines or blocks into the metadata (three here,
and the four that add to `header-includes`); low.

## 20. Several filters as one program: Python's `main` and Julia's `run_filter` take one (closed: Python, Julia)

**Closed** in panir#16 for Python (`main([f, g])`, each filter handed
over in turn to a host) and Julia (`run_filter([f, g])`, `walk!([f, g],
doc)`, each a function or `f => traverse`); pandoc-quotes uses both.
Rust: **not closed**, declined: a list type (a slice of `&mut dyn
Filter`) could not share the earlier filter's state with the later one
without a `RefCell`; the README shows `apply` after `apply`, `?` between,
the state a field given to the second. (Rust was not among this gap's
languages: its port did that already.)


Found with pandoc-quotes.

**The original**: `return {{Meta = configure}, {Quoted = insert}}`: two
filters in turn, the second using what the first read. Needed: in one
typewise walk the metadata comes after the inlines, and a top-down one
would leave a quotation directly in a quotation unrewritten.

**panir**: TypeScript's `runFilter([configure, quotes])` is the Lua; Rust
applies one filter then the other in `main`. Python's `Filter.main` runs
one filter (`run(doc, [f, g])` exists, but no `main` for it), Julia's
`run_filter(f; traverse)` one function; the ports use a `Pandoc` function
that reads the metadata, then walks the document with a second filter:
correct, but not the original's shape, and the hosts that run a script's
filter (libpandoc's `pandocjl`, this repository's `check.py` and
`julia-cases.jl`) take one.

**Proposed**: Python `panir.main([f, g])` (or `f.main(then=[g])`), Julia
`run_filter(f, g; traverse)` (or pairs of a function and its order), and
the hand-off (`handoff`, `hand_over`) carrying a list.

How common: 4 of 21 (pandoc-quotes, count-para, auto-lang, amsthm: the
`multi` filters).

## 21. A nested option: no lookup by path (closed)

**Closed** in panir#16: `lookup(meta, "a", "b")` (Python, TypeScript,
Julia: from the metadata, a `MetaMap` or nothing), Rust `lookup(&meta,
&["a", "b"])` (from a `Meta`, the metadata or a `MetaMap`'s content);
`None` where a key is missing or a value on the way is not a map.


Found with abstract-section.

**The original**: `(doc.meta['abstract-section'] or {})['section-identifiers']`,
`config['break-on']['form-feed']` (pagebreak).

**panir**: each level is a test that the value is a map, then its
`content`: Python 2 lines (`isinstance`), TypeScript 1 (`a?.t ===
"MetaMap" ? a.content.b : undefined`), Rust a `match` of 3 to 5, Julia a
method per kind.

**Proposed**: a lookup through maps, `None` where a level is missing or
not a map: Python `panir.get(meta, "abstract-section",
"section-identifiers")`, TypeScript `getMeta(meta, ...)`, Rust
`panir::meta_get(&meta, &["abstract-section", "section-identifiers"])`
(`Meta` is a type alias: no method), Julia `Panir.lookup(meta, keys...)`.

How common: 4 of 21 (abstract-section, pagebreak, first-line-indent,
amsthm).

## 22. One value or a list of them (closed)

**Closed** in panir#16: `items(v)` in the four languages, a new list
(Rust a slice borrowed from the document).


Found with scholarly-metadata and abstract-section.

**The original**: scholarly-metadata's `ensure_list` (a list as it is,
anything else alone, nothing for `nil`); `header-includes` "whatever shape
it has" (absent, one value, a list); abstract-section and pandoc-quotes
walk a list, inlines or blocks alike with `ipairs`.

**panir**: one arm per constructor: Python `match` (one arm for three
with `|`), TypeScript a `switch` whose `map` on the union of three array
types needs `(x: unknown) => ...`, Rust three arms (their items are
different types), Julia one method on a `Union`. scholarly-metadata's
`ensure_list` with its comma split: Lua 22 lines, Python 11, TypeScript
9, Rust 18, Julia 7.

**Proposed**: `items(value)`: a `MetaList`'s items, nothing for a missing
value, else the value alone (Python `panir.items`, TypeScript
`metaItems`, Rust `MetaValue::items(Option<MetaValue>) -> Vec<MetaValue>`,
Julia `Panir.items`), the `header-includes` pattern in one call. (Lua's
`ipairs` over inlines is each filter's quirk, not a helper's.)

How common: 7 of 21: three here, and the four that add to
`header-includes`.

## 23. Rust has no metadata from plain values (closed)

**Closed** in panir#16: `From` for strings, `bool`, inlines and blocks
(one or a `Vec`), `Vec<MetaValue>` and `Meta`, and `FromIterator` of
values (`MetaList`) and of pairs (`MetaMap`). cito's 14 lines are 4.


Found with cito.

**The original**: `doc.meta.cito_cites = citations_by_property`: a table
of lists of strings becomes a `MetaMap` of `MetaList`s of `MetaString`s.

**panir**: Python (`from_python` on assignment), TypeScript (`MetaMap`,
`fromPlain`) and Julia (`convert` on assignment) take the same in one
line. Rust has no `From` for `MetaValue`: cito's line is 14 (rustfmt's)
of `MetaValue::MetaMap(... MetaValue::MetaList(... MetaValue::MetaString(
k.into()) ...))`.

**Proposed**: `From<&str>`, `From<String>`, `From<Text>` (`MetaString`),
`From<bool>`, `From<Vec<Inline>>` (`MetaInlines`), `From<Vec<Block>>`
(`MetaBlocks`), and `FromIterator` of `MetaValue`s (`MetaList`) and of
pairs (`MetaMap`), so that cito's is `by_property.into_iter().map(|(p,
ks)| (p.into(), ks.into_iter().map(MetaValue::from).collect())).collect()`.

How common: 3 of 21 (cito, scholarly-metadata, first-line-indent's
`meta.indent = true`).

## 24. Rust: `ctx.fail` and `ctx.apply` from a function returning `()` (closed)

**Closed** in panir#16 as proposed: `inlines`, `blocks`, `meta` and
`pandoc` return `Option<()>`, so `ctx.apply(...)?` and `return
ctx.fail(e)` in every function.


Found with scholarly-metadata and abstract-section.

**The original**: fails (`error`, or Lua failing on bad metadata) and
walks (`doc.blocks:walk{...}`) from its `Meta` or `Pandoc` function: the
usual place for both.

**panir**: `meta`, `pandoc`, `inlines` and `blocks` return `()`, so
`ctx.fail(e)` (generic in its `Option<T>`) needs `ctx.fail::<()>(e);` and
a `return` after, and `ctx.apply(x, f)` can't take `?`: `if
ctx.apply(...).is_none() { return; }`. The README's "`error(...)`:
`return ctx.fail(...)`" and "`ctx.apply(&mut el, &mut filter)?`" hold
only in `inline`, `block` and `meta_value`.

**Proposed**: those four return `Option<()>` (default `Some(())`), so that
`ctx.fail(e)?` and `ctx.apply(x, f)?` read the same in every function; a
breaking change, as panir may make.

How common: 6 of 21 (four here; pagebreak and track-changes walk from
their `Pandoc` function).

## 25. Rust's `Link` and `Target` are built field by field (closed in part)

**Closed** in part by panir#16: `Target::new(url, title)`, as
`Attr::new`, in the order of Lua's `pandoc.Link(content, target,
title)`. **Not** `Default` for `Target` (declined: its url has no default
in pandoc's Lua), nor `Link::new`: a link is still a struct literal with
`attr: Attr::default()`.


Found with author-info-blocks.

**The original**: `pandoc.Link(content, 'mailto:' .. email)`.

**panir**: Python, TypeScript and Julia take `url=` (with `title` and the
attributes defaulted); Rust has neither a constructor nor `Default` for
`Target` and `Link`: `Link { attr: Attr::default(), content, target:
Target { url, title: "".into() } }`.

**Proposed**: `Default` for `Target` (pandoc's Lua defaults the title),
or `Target::new(url)`; perhaps `Link::new(content, url)`, as `Attr::new`
(gap 2). Low.

How common: 2 of 21 (author-info-blocks, count-para).
