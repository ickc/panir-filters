# Gaps in panir's filter API

What porting real filters found missing or awkward in
[panir](https://github.com/ickc/panir), each with a proposed change.
How many of round 1's 21 filters (`SURVEY.md`) would hit each, counted
from their sources, ranks them, then how much it hurts; they are numbered
in the order found. Gaps 1 to 35 are closed (20, 25 and 32 in part, as
panir declined the rest) but 33, left until a second filter needed it,
which amsthm is. Open, by rank: 38, 37, 33, 36, 39, 40. Each filter's
`NOTES.md` has the details.

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
the same). 26 to 28 were found bringing them onto it, 29 to 35 with the
fourth batch (`pagebreak`, `track-changes`, `first-line-indent`,
`count-para`), which reads nested options and the environment, writes
`header-includes`, and is two programs of several filters: what it found
of panir#16's API in use is in "The API of panir#16, used", after 25.
26 to 35 were closed by [ickc/panir#17](https://github.com/ickc/panir/pull/17),
which the nineteen ports now use (twelve Rust ports and three TypeScript
ones changed; `run.py test` and `CHECK_UNIQUE=1` the same). 36 to 40 were
found with the fifth batch, the owner's two filters (`auto-lang`,
`amsthm`), the last of round 1: what they found of panir#17's API in use
is in "The API of panir#17, used", after 35. Rounds 2 and 3 (filters
that read files and call pandoc) found 41 to 44, at the end; what they
found in the hosts is `HOSTS.md`.

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
| 26 | TypeScript's `truthy` is not a type guard: the value needs `!` after it | TypeScript | 2 seen: pandoc-quotes, pagebreak (of the 9 that test Lua's truth) | closed, panir#17: `given(v)` |
| 27 | Rust's `truthy` takes an `Option`: a value in hand is `truthy(Some(v))` | Rust | 3: pandoc-quotes, first-line-indent, count-para | closed, panir#17: `truthy(v)`, and `given` |
| 28 | Rust's `items` borrows: a value taken out of the metadata is copied to be one or a list | Rust | 2: scholarly-metadata, author-info-blocks | closed, panir#17: `into_items` |
| 29 | Rust's `lookup` starts from a `Meta`, not from a value | Rust | 3: abstract-section, pagebreak, amsthm | closed, panir#17: from a `Meta`, a value, or an `Option` of one |
| 30 | Rust has no constructor for raw blocks and inlines | Rust | 10: comments, theorem, inlinenotes, french-spaces, author-info-blocks, pagebreak, track-changes, first-line-indent, count-para, amsthm | closed, panir#17: `Block::raw`, `Inline::raw` |
| 31 | Rust: a `Result` into `ctx.fail` is a four-line `match` | Rust | 7: list-table, scholarly-metadata, author-info-blocks, pandoc-quotes, abstract-section, track-changes, first-line-indent | closed, panir#17: `ctx.ok(result)` |
| 32 | TypeScript's `toPlain` returns `unknown`: a cast to give it to a constructor | TypeScript | 1: first-line-indent | closed in part, panir#17: a typed `PlainValue`; the port's cast stays (Lua's failure, not a type) |
| 33 | a metadata value as blocks, as Lua converts it where a block goes: Rust and Julia by hand | Rust, Julia (Python, TypeScript but for a map) | 2: first-line-indent; amsthm (as inlines) | closed, panir#18: `inlines_of`, `blocks_of`, as Lua converts each shape |
| 34 | "Coming from Lua" misses what this batch's originals did: the `Meta` function's `return nil`, `.t` since pandoc 2.17, `PANDOC_READER_OPTIONS`, `gsub`'s replacement `%`, strings as bytes | all (documentation) | 4: first-line-indent, track-changes, count-para, pandoc-quotes | closed, panir#17: a bullet each, with an example that runs |
| 35 | a node returned by two calls is in two places; "what a function returns are values" reads otherwise | Python, TypeScript, Julia (documentation) | 2: pagebreak, (first-line-indent, had its port kept the original's one node) | closed, panir#17: said, the copy shown; the scenarios `check_unique` |
| 36 | Rust's list functions change their list in place: one that builds a new list and may decline (Lua's `Inlines` returning `nil`) decides first, a second look | Rust | 2: auto-lang, amsthm (its `\qedhere`) | closed, panir#18: said in the Rust README, with the idiom |
| 37 | a filter script that loads a file beside it, run in process: Python's `runpy.run_path` puts no folder on `sys.path` (libpandoc-python's host, this repository's) | Python (Julia's hosts, fixed here; LibPandoc.jl's gives `include`) | 1: auto-lang (its table); any filter of more than one file | closed, libpandoc-python#6: the script's folder is on the import path while it runs |
| 38 | the writer's options (Lua's `PANDOC_WRITER_OPTIONS`: -N, `--top-level-division`, `--number-offset`, `-V`): a JSON filter is given none; panir has them in process only, as an untyped map; no "Coming from Lua" names them | all (Rust most: no hand-off, so never) | 1: amsthm (its own tests use them) | closed in part, panir#18: `writer_options` on the conversion, read from `PANDOC_WRITER_OPTIONS`; libpandoc does not set it yet, and plain pandoc gives a JSON filter none |
| 39 | Lua's `pairs` over metadata: a map's entries, a list's (or inlines') indices, an error on text or a boolean; "Coming from Lua" does not say | all (documentation) | 2: auto-lang, amsthm | closed, panir#18: a bullet in each "Coming from Lua" (a map's order is random in Lua) |
| 40 | `pandoc.utils.to_roman_numeral`: not in panir | all | 1: amsthm (a part's number) | closed, panir#18: `to_roman_numeral` |

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
in use: "The API of panir#16, used", after gap 25.

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

## The API of panir#16, used (the fourth batch)

The fourth batch was chosen to test it: three of its four filters write
`header-includes`, two are lists of filters (`first-line-indent`: two,
`count-para`: four), and all read their options from the metadata.

- **Lua's truth**: right in practice, in every language. Python's
  `bool()` is the best of the four: the Lua lines port as written
  (`if lookup(config, "break-on", "form-feed"):`, pagebreak's `config.x
  and stringify(config.x) or os.getenv(...)` as `stringify(v) if v else
  os.environ.get(...)`), and Lua's other test, `x ~= nil`, stays `is not
  None`, as distinct as in Lua. TypeScript's `truthy` needs a `!` on the
  value after it (26: a `given(v)`, the value or `undefined`, would be
  Lua's `v and ...` and, with `??`, Lua's `v or d`); Rust's needs
  `Some(...)` around a value in hand (27). No port of this batch wrote a
  helper for it; the third batch's sixteen are gone.
- **`lookup`**: read every nested option in one call (pagebreak's
  `break-on`, first-line-indent's six options), from the metadata or from
  a value taken out of it, in Python, TypeScript and Julia. Where Lua
  fails on `true` on the way, `lookup` returns nothing, as documented:
  pagebreak, first-line-indent and abstract-section each test for `true`
  themselves, one line; right, as a filter may well not want Lua's
  failure. Rust's starts from a `Meta` only (29).
- **`items`**: fit one of the three filters writing `header-includes`
  exactly (first-line-indent: a list's items or the value alone), after
  Lua's `if x then` (`items(x) if x else []`: `items` keeps `false` as a
  value, as Lua's `ensure_list` would; right). track-changes means it but
  can't use it (its test for a list is pandoc 2's, so the original nests a
  list in a list), and count-para appends in place and fails on anything
  but a list. So "`header-includes` whatever shape" was, in practice,
  three shapes of bug; `items` is what a correct filter would use. Rust's
  borrows (28).
- **Plain values**: Python's `to_python` with the `MetaBlocks`
  constructor reproduced Lua's conversion of metadata where blocks go
  (text a `Plain`, blocks, lists and booleans an error) in one
  expression; TypeScript's `toPlain` too, with a cast (32); Rust and
  Julia have no plain values, so a small function by kind (33). Setting
  `indent` converts `True`/`true` in Python, Rust (`.into()`) and Julia;
  TypeScript's metadata fields don't convert (`MetaBool(true)`), as its
  README says. Rust's `From` covered every metadata value these ports
  made but a raw block, `Block::from(RawBlock { .. }).into()`, two
  conversions (30).
- **A list of filters**: Python's `main([...])`, TypeScript's and
  Julia's `run_filter([...])` read as the originals, the state between
  them module-level as Lua's upvalues; nothing surprising. Rust's
  `apply` after `apply`, as its README shows, was the clearest of the
  five in count-para: each filter borrows what the ones before made
  (`&settings.0`, `&numbering.numbers`). `unique/check.py` (now `host/python.py`) and
  `julia-cases.jl` needed nothing more (the hand-off carries the list).
- **Rust's `Option<()>` functions**: `ctx.apply(doc, &mut f)` as the last
  expression of `pandoc` (pagebreak), `ctx.apply(...)?` in `pandoc` and
  `inline`, `return ctx.fail(...)` anywhere, and `self.numbers.get(name)?`
  ending an `inline` early (count-para) all read well. What remains is
  the helper returning a `Result`, matched into `ctx.fail` (31).
- **`Target::new`**: used (count-para's link). `Attr::new`: used for a
  span. `equal`/`==` on nodes: first-line-indent compares the first
  inline with `\indent ` as Lua's `utils.equals`, in every language.
- **The metadata part of "Coming from Lua"** answered the questions of
  truth, nested options (and that Lua would fail on `true`), one or a
  list, and copies. It did not answer what these originals also did (34):
  first-line-indent's `Meta` function returning `nil` (so its changes to
  the metadata are lost): the README's trap speaks of "a function gets a
  copy of its element", and the metadata is not an element there;
  track-changes' `meta['header-includes'].t` (nil since pandoc 2.17: the
  README's table gives `pandoc.utils.type`, but not that `.t` is gone,
  which pandoc-quotes and track-changes test) and its
  `PANDOC_READER_OPTIONS.trackChanges` (pandoc 2's name; no README maps
  `PANDOC_READER_OPTIONS` to `ctx.conversion`'s reader options, which are
  pandoc's JSON, `"track-changes": "all-changes"`); first-line-indent's
  `gsub` with a replacement from the document (`%` is special there: the
  patterns line speaks of patterns only); count-para's `len` and `sub`,
  which count bytes.
- **Copies** (`CHECK_UNIQUE=1`): one needed, pagebreak's page break,
  made once and returned for every marker as the original does; the
  check found it when the copy was left out (35). The others make every
  node anew.

## 26. TypeScript's `truthy` is not a type guard (closed)

**Closed** in panir#17: `given(v)`, the value if `truthy`, else
`undefined` (`given<T extends MetaValue>(v: T | null | undefined): T |
undefined`): `const marks = given(meta["quot-marks"]); if (marks) ...`
narrows, and `given(meta["quot-lang"]) ?? given(meta.lang)` is the Lua
line (pandoc-quotes). pagebreak's `!` went without it: `stringify` takes
any value, so `stringify(meta[key]!)` never needed it.

Found bringing pandoc-quotes onto panir#16; again with pagebreak.

**The original**: `if meta['quot-marks'] then ... meta['quot-marks']`,
`config['html-class'] and stringify(config['html-class']) or ...`: Lua's
truth, then the value.

**panir**: `truthy(v): boolean`, deliberately not `v is MetaValue` (its
false branch would say `undefined`, wrongly: `MetaBool(false)` is false
too). So after `if (truthy(meta.x))` the value is still `MetaValue |
undefined`: `stringify(meta.x!)` (pagebreak), `quotationMarks(marks!)`
(pandoc-quotes). The ports' own `given` was a type guard, wrong in the
same way but convenient.

**Proposed**: also `given(v): MetaValue | undefined`, the value if
`truthy`, else `undefined`: `const c = given(meta.x); if (c)
stringify(c)` narrows, and `given(meta.x) ?? d` is Lua's `meta.x or d`
exactly. Low.

How common: 2 seen of the 9 filters that test Lua's truth.

## 27. Rust's `truthy` takes an `Option` (closed)

**Closed** in panir#17: `truthy` and `items` take `impl
Into<Option<&MetaValue>>`, so `truthy(v)` of a value in hand
(count-para's two options), and `given(meta.get("x"))`, the value if
`truthy`, which replaced each `.filter(|v| truthy(Some(v)))` and
`.find(|v| truthy(*v)).flatten()` (first-line-indent, pandoc-quotes,
pagebreak's `option`, now `given(..).map(stringify).or_else(..)`, Lua's
`a and f(a) or c`).

Found bringing pandoc-quotes onto panir#16; again with first-line-indent
and count-para.

**panir**: `truthy(value: Option<&MetaValue>)`, as `meta.get("x")` gives;
a value in hand (an `if let Some(v)`, a `filter` closure) is
`truthy(Some(v))`: `meta.get("x").filter(|v| truthy(Some(v)))`,
`if let Some(v) = meta.get(k) { truthy(Some(v)) }`.

**Proposed**: `truthy(value: impl Into<Option<&MetaValue>>)` (a
`&MetaValue` converts), or a method `MetaValue::truthy(&self)` beside it.
Not breaking. Low.

How common: 3 (pandoc-quotes, first-line-indent, count-para).

## 28. Rust's `items` borrows (closed)

**Closed** in panir#17: `into_items(v)`, a `Vec` moved out of what
`meta.remove` gives (or of a value). scholarly-metadata's `as_list` and
author-info-blocks' `maps` take the value out and move its items; the
`.to_vec()` and the clone per author are gone.

Found bringing scholarly-metadata and author-info-blocks onto panir#16.

**The original**: `ensure_list(meta.author)`, then the authors are
changed and written back.

**panir**: `items(Option<&MetaValue>) -> &[MetaValue]`, borrowed from the
document. A port that took the value out (`meta.remove("author")`) to
change and move its items now reads it borrowed and copies them
(`items(meta.get("author")).to_vec()`, `map.clone()` per author), or
writes its own `match` to keep them.

**Proposed**: also `MetaValue::into_items(self) -> Vec<MetaValue>` (a
list's items, else the value alone), and `Option<MetaValue>` the same by
a function, so that `items_of(meta.remove("author"))` moves. Low: a copy
of a few authors costs nothing measurable.

How common: 2 (scholarly-metadata, author-info-blocks).

## 29. Rust's `lookup` starts from a `Meta` (closed)

**Closed** in panir#17: `lookup(&from, path)` from a `Meta`, a
`MetaValue`, or an `Option` of either. pagebreak and abstract-section
now take their configuration out first (`let config =
doc.meta.remove("pagebreak")`) and read it (`lookup(&config,
&["break-on", "form-feed"])`), as the original and the other languages
do.

Found with pagebreak (and abstract-section, brought onto panir#16).

**The original**: `local config = doc.meta.pagebreak or {}`, then
`config['break-on']['form-feed']`, the configuration then removed.

**panir**: Python, TypeScript and Julia take the metadata, a `MetaMap`
or nothing: `lookup(config, "break-on", "form-feed")` on the value taken
out. Rust's `lookup(meta: &Meta, path)` takes a map only, so the ports
read `lookup(&doc.meta, &["pagebreak", "break-on", ...])` before
removing `pagebreak`, and a `MetaValue` in hand would need a `match` to
its `MetaMap`.

**Proposed**: a `lookup` generic over what it starts from (a trait for
`Meta`, `MetaValue` and `Option<&MetaValue>`), as the other languages'.
Low.

How common: 3 (abstract-section, pagebreak; amsthm reads its own).

## 30. Rust has no constructor for raw blocks and inlines (closed)

**Closed** in panir#17: `Block::raw(format, text)` and
`Inline::raw(format, text)`, taking `impl Into<Text>`. The eight `raw`
helpers are gone (theorem, pagebreak, track-changes, first-line-indent,
count-para; french-spaces, inlinenotes and author-info-blocks had
struct literals in place), and a raw block as metadata is
`Block::raw(f, t).into()`. With `ctx.ok`, `given` and the rest of
panir#17, the twelve Rust ports it changed are 94 lines shorter, 1 to 14%
(`RESULTS.md`).

Found with pagebreak; the four ports of this batch all wrote a helper.

**The original**: `pandoc.RawBlock('html', s)`, `pandoc.RawInline('tex',
s)`: the most common node a filter makes.

**panir**: Python, TypeScript and Julia: `RawBlock(format, text)`. Rust:
`RawBlock { format: "html".into(), text: s.into() }.into()`, four lines
under rustfmt; as metadata, `Block::from(RawBlock { .. }).into()`. Each
port has a `raw(format, text)` helper, eight so far.

**Proposed**: `Block::raw(format, text)` and `Inline::raw(format, text)`
taking `impl Into<Text>`, returning the variant (as `Attr::new` and
`Target::new` did for theirs); perhaps `RawBlock::new` too.

How common: 10 of 21 (comments, theorem, inlinenotes, french-spaces,
author-info-blocks, pagebreak, track-changes, first-line-indent,
count-para, amsthm).

## 31. Rust: a `Result` into `ctx.fail` is a four-line `match` (closed)

**Closed** in panir#17: `ctx.ok(result)`, the value or `None` having
failed with the error. Each of the seven `match`es is one line:
`ctx.ok(self.configure(meta))` as the last expression of `meta` or
`pandoc`, `let id = ctx.ok(self.author(span))?;` in `inline`,
`self.marks = ctx.ok(configure(meta))?;`.

Found with first-line-indent; seven ports have it.

**The original**: fails anywhere in its helpers (`error`, or Lua
failing).

**panir**: a Rust port's helpers return `Result<T, String>` (with `?`
inside), and the filter function turns it into the walk's failure:
`match self.configure(meta) { Ok(()) => Some(()), Err(e) => ctx.fail(e)
}`, or `let x = match ... { Ok(x) => x, Err(e) => return ctx.fail(e) };`.

**Proposed**: `Ctx::ok<T, E: Into<Box<dyn Error + Send + Sync>>>(&mut
self, r: Result<T, E>) -> Option<T>`, recording the error as `fail`
does: `ctx.ok(self.configure(meta))`, `let x = ctx.ok(helper())?;`.

How common: 7 (list-table, scholarly-metadata, author-info-blocks,
pandoc-quotes, abstract-section, track-changes, first-line-indent).

## 32. TypeScript's `toPlain` returns `unknown` (closed in part)

**Closed in part** in panir#17: `toPlain` returns a `PlainValue`, by
the kind of value given. first-line-indent's `as Blocks` stays: its
values are any metadata value (a `MetaValue`, so the union), which Lua
converts where blocks go or fails on; the cast says "as Lua would",
which no type can (panir's commit says the same).

Found with first-line-indent.

**panir**: `toPlain(v): unknown`; the port gives what it returns to
`MetaBlocks([...])`, which converts plain values as Lua does, and needs
`as Blocks` for the type checker.

**Proposed**: a `Plain` type (`string | boolean | Inline[] | Block[] |
Plain[] | { [k: string]: Plain }`) as `toPlain`'s result, and the
constructors' init types accepting it where it converts. Low.

How common: 1 (first-line-indent).

## 33. A metadata value as blocks, as Lua converts it (open again)

**Left** as it is in panir#17, as proposed: one filter of 21 needed
it, and a `blocks_of(value)` was to come when a second did. **amsthm is
the second**, as inlines: its `qed_symbol` is `pandoc.Inlines(meta.x)`,
which Lua converts as it does where inlines go (checked with pandoc
3.14: inlines as they are, text its words, a list of strings a `Str`
each, unsplit, a map none, a list of anything else, blocks and booleans
an error). Every port wrote it, by kind: Python a `match` of 5 arms,
TypeScript 5 lines, Rust a `match` of 5, Julia 5 methods; and Python's
`inlines()` takes text only, so it could not be given the value.

**Proposed now**: `inlines_of(value)` and `blocks_of(value)` in each
language (Python and TypeScript perhaps through `to_python`/`toPlain`
and the constructors, as first-line-indent's `as_block`, but these fail
on a map, where Lua gives none), converting as Lua does, failing where
Lua fails, with the message naming the kind. Low: 2 of 21.

Found with first-line-indent.

**The original**: `pandoc.MetaBlocks{block, meta['header-includes']}`:
Lua converts each value where a block goes (text a `Plain`, a string a
`Plain` of its words, a table without items an empty `Plain`; blocks, a
list, a boolean fail).

**panir**: Python and TypeScript do it with their plain values and
constructors, but for a map (an error there); Rust and Julia have no plain
values: a function per kind (6 and 4 lines).

**Proposed**: nothing for now (one filter of 21, and a quirk of Lua's
conversion); if more need it, a `blocks_of(value)` in each language.

How common: 1 (first-line-indent).

## 34. "Coming from Lua" misses what this batch's originals did (closed)

**Closed** in panir#17: a bullet each in every "Coming from Lua" (the
first two in its "Metadata" part), each checked with pandoc 3.14 and with
an example that runs.

Found with first-line-indent, track-changes and count-para.

**panir**: each README's "Coming from Lua", with its metadata part,
answered truth, nested options, one or a list, copies. Not:

- the `return nil` trap holds for the `Meta` function too: its changes to
  the metadata count only if it returns it (first-line-indent's do not
  when `indent` isn't set). The README says "a function gets a copy of
  its element";
- a metadata value's `.t`, which pandoc 2 had, is `nil` since pandoc
  2.17, so an original testing it takes the other branch (pandoc-quotes
  fails, track-changes nests `header-includes`);
- `PANDOC_READER_OPTIONS` is `ctx.conversion`'s reader options, as
  pandoc's JSON (`"track-changes": "all-changes"`), not Lua's field names
  (`track_changes`), and pandoc 2's (`trackChanges`, `AllChanges`) are
  gone;
- in `gsub`'s replacement `%` is special (`%1`, `%%`, an error
  otherwise), so a replacement from the document can fail
  (first-line-indent's `size: 5%`);
- `#s`, `s:len()`, `s:sub(i, j)` count bytes (count-para's enclosure).

**Proposed**: a bullet each in "Coming from Lua" (the first two in its
metadata part).

How common: 4 (first-line-indent, track-changes, count-para,
pandoc-quotes).

## 35. A node returned by two calls is in two places (closed)

**Closed** in panir#17: Python's, TypeScript's and Julia's READMEs say
it ("not across calls"), with the copy for each return shown and
`check_unique` called; panir's own scenarios now `check_unique` what they
make (which found one in its `newdoc`).

Found with pagebreak.

**The original**: makes one page-break element and returns it for every
marker: in Lua, each return is a copy.

**panir**: Python's, TypeScript's and Julia's READMEs say "As in Lua,
what a function returns are values: `[el, el]` puts two nodes in the
document, the second a copy". A port returning its one node from each
call puts one node in many places (`CHECK_UNIQUE=1` finds it). The next
sentence ("Elsewhere a node belongs in one place") covers it, read
closely.

**Proposed**: say it: "within one return; a node returned by two calls
is in two places: return a copy". Low.

How common: 2 (pagebreak; first-line-indent's original keeps its raw
nodes in a table the same way, though its ports make them anew).

## The API of panir#17, used (the fifth batch)

The fifth batch is the owner's two filters: auto-lang (a top-down
`Inlines` function regrouping every list, a table of Unicode scripts) and
amsthm, the largest of round 1 (1,033 lines of Lua: two passes, three walks
within a function, counters, metadata of every shape, LaTeX and the other
formats apart, cross-references). Both were written against panir#17.

- **`Block::raw` and `Inline::raw`** (Rust): thirteen in amsthm's port
  (`\begin{...}`, `[`, `]`, `\label`, `\end`, the references, the
  `header-includes`), each reading as Lua's `pandoc.RawInline('latex',
  ...)`; as metadata, `MetaValue::from(Block::raw(..))`. Right: no helper
  was written, and none was missed.
- **`ctx.ok`** (Rust): once, the options' `Result` in the `pandoc`
  function (`let options = ctx.ok(read_options(..))?;`), as Lua fails in
  its `Pandoc` function on bad metadata. Right, and the only `Result` the
  port has: the rest of amsthm warns and goes on.
- **`given`** (TypeScript, Rust): four times in each amsthm port, each
  Lua's `meta.x and f(meta.x)` (`documentclass`, `counter_depth`,
  `secnumdepth`, `counter_ignore_headings`): `const d = given(...); d ?
  stringify(d) : ...` narrows, and `items(given(x))` is Lua's `if x then
  for ... in ipairs(meta_list(x))`, one call. Right. Python's `if
  opt.get("counter_depth"):` and Julia's `truthy(x) ? ... : ...` read as
  well, as their READMEs say.
- **`truthy`, `items` from a value; `into_items`** (Rust):
  `into_items(meta.remove("header-includes"))` moves what is there into
  the new list, Lua's `add_header_include` in four lines; `items(v)` of a
  value in hand where the original's `meta_list` takes one. Right.
- **`lookup`**: not needed: amsthm reads one level of its map.
- **"Coming from Lua"'s new bullets**: the `gsub` replacement (amsthm
  doubles `%` in its symbol for `gsub`: the ports put the text in as it
  is, `re.subn` with a function, `replace` with a function or a plain
  string, as the bullet says); strings as bytes (auto-lang walks
  `utf8.codes`: each port iterates code points, which the bullet's
  `utf8.len` points to); `.t` (amsthm uses `pandoc.utils.type`: the
  bullet's table had it right). What they did not cover: Lua's `pairs`
  over a metadata value (39), and `PANDOC_WRITER_OPTIONS` (38).
- **Copies** (`CHECK_UNIQUE=1`): amsthm's held environments are put back
  wherever a div has the attribute; Lua returns a copy each time, a port
  returning its one div put it in two places, numbered twice (the case
  `bodies` showed it before `CHECK_UNIQUE` did): 35 again, which the
  README now says.
- **Walks within a function**, three in amsthm (the body held out, set in
  its font, put back; `\qedhere`), one a fragment's typewise walk after a
  top-down one: `div.content.walk(f)`, `applyFilter(div.content, f)`,
  `ctx.apply(&mut div.content, &mut f)?`, `walk!(f, div.content)`, each as
  Lua's `pandoc.Blocks(div.content):walk{...}`, in place. A walk is of
  what is in a node, not the node itself, in panir as in Lua: the proof's
  note, walked as a `Para`, has its own merge called after, as the
  original does. pandoc's typewise order (every inline, then every block)
  and its top-down order (a list's function before its elements, a
  replacement's children walked) were what both originals assume, and
  what panir does, in every language.

## 36. Rust's list functions change their list in place

Found with auto-lang.

**The original**: an `Inlines` function that builds a new list from the
old one (splitting each `Str` into pieces), and returns it only if it
made a span: `return changed and out or nil`, the list as it was
otherwise.

**panir**: Python, TypeScript and Julia return the new list or nothing,
as Lua. Rust's `inlines(&mut self, xs: &mut Vec<Inline>, ..)` changes the
list in place; a port that takes the list apart to build the new one
(`std::mem::take(xs)`) has lost the old one by the time it knows, so it
decides first (auto-lang's `has_run`, a second look at every `Str`), or
clones the list. amsthm's `\qedhere` does the same, cheaply
(`xs.iter().any(...)`).

**Proposed**: no API change (an `Option<Vec<_>>` return would cost a new
list where most list functions change one item); a line in the Rust
README's "Coming from Lua": "an `Inlines` or `Blocks` function returning
`nil` keeps the list as it was: decide before taking it apart" (gap 4's
rule, for lists). Low.

How common: 2 of 21 (auto-lang, amsthm).

## 37. A filter script that loads a file beside it, run in process

Found with auto-lang.

**The original**: `dofile`s its table, `script-ranges.lua`, from the
folder of `PANDOC_SCRIPT_FILE`.

**panir**: each port has its table as a module beside it (`from
script_ranges import ...`, `import "./script-ranges.ts"`, `mod
script_ranges;`, `include("script-ranges.jl")`). Run as a program, each
finds it. Run in process by a host, as panir's hand-off is for: Python's
`runpy.run_path` puts nothing on `sys.path` (running `python script.py`
puts the script's folder first), so the import fails, in this
repository's host (`host/python.py`, fixed) and in libpandoc-python's
(`_scripts._in_process`, which runs a script with `runpy.run_path`; not
changed here); Julia's `Base.include(Module(), script)` gives the script
no `include` (this repository's `julia-cases.jl`, fixed; LibPandoc.jl's
`pandocjl` defines one, as `julia script.jl` has it). TypeScript and
Rust are not run that way.

**Proposed**: in panir, the hand-off's documentation (Python's
`panir.filter.handoff`, Julia's `Panir.handoff`) says what a host gives a
script: its folder on `sys.path`, a module with `include`, as running
it would; better, one function per language that runs a filter script
in process as a host should (`panir.run_script(path)`, `Panir.run_script
(path)`), which libpandoc's hosts and this repository's would call; and
libpandoc-python's runner puts the folder on `sys.path` while the script
loads. Medium: any filter of more than one file (round 2's
include-files, pantable).

How common: 1 of 21 (auto-lang).

## 38. The writer's options

Found with amsthm.

**The original**: reads `PANDOC_WRITER_OPTIONS`: `number_sections`
(-N), `top_level_division`, `number_offset`, and the variable
`secnumdepth` (-V), to number theorems within sections as LaTeX does; its
own tests run with them (`-N`, `--top-level-division=chapter`,
`--number-offset=4`, `-V secnumdepth=0`).

**panir**: a JSON filter is not given them: pandoc 3.14 passes
`PANDOC_READER_OPTIONS` and `PANDOC_VERSION` only (checked with a filter
printing its environment). panir's `ctx.conversion.options` has the
conversion's options, in defaults-file keys, where its host knows them
(libpandoc, in process), as an untyped map (`dict`, `Record<string,
unknown>`, `serde_json::Value`, `AbstractDict`): reading
`variables.secnumdepth` takes a cast in TypeScript, a value that may be
text or a number in all four. No README's "Coming from Lua" names
`PANDOC_WRITER_OPTIONS`. Under pandoc the ports number as the original
does without -N; to check them with the options, `run.py` grew a case's
options (`NAME.opts`), given to the original with `pandoc -d` and to the
Python, TypeScript and Julia ports in process through the hand-off
(`host/`). A Rust filter is a program: it is never run in process, so it
never has them (a `cargo test` in the port checks them in process).

**Proposed**: (a) upstream, pandoc to pass the writer's options to a
JSON filter as it does the reader's (`PANDOC_WRITER_OPTIONS`, JSON), as
jgm/pandoc#11016 proposes for the input format, and libpandoc to set it
when it runs a JSON filter; then each `Conversion.from_environment` reads
it into `options`, for every language, Rust's programs included; (b) a
"Coming from Lua" bullet: `PANDOC_WRITER_OPTIONS` is `ctx.conversion
.options`, in defaults-file keys (`number-sections`, `top-level-division`,
`number-offset`, `variables`), known only where the host knows them;
(c) perhaps typed accessors for the few options filters read (numbering,
the top-level division, variables). Medium: in round 1 only amsthm, but
any filter that numbers or lays out by section (pandoc-crossref's kind,
round 4) needs them.

How common: 1 of 21 (amsthm).

**Since** (round 2): libpandoc 1.10 sets `$PANDOC_WRITER_OPTIONS` for
every filter it runs, and a conversion's `writer_options` in process
(panir#18 reads it), with Lua's values under pandoc's JSON keys; amsthm's
ports read `options` where a host knows them, else `writer_options`, and
are the same in every host and as a JSON filter run by libpandoc (the
Rust port's 36 known rows gone; `run.py test` gives every port
`$PANDOC_WRITER_OPTIONS` as libpandoc sets it). Left open: (a) upstream
(plain pandoc still gives a JSON filter none); (b) each README's "Coming
from Lua" still says "which nothing sets yet (proposed to pandoc and
libpandoc)": libpandoc does.

## 39. Lua's `pairs` over metadata

Found with auto-lang and amsthm.

**The original**: `for script, tag in pairs(meta["auto-lang"] or {})`,
`for k, v in pairs(opt.name_to_text)`: a map's entries. On a list, or
inlines (`auto-lang: Greek`), `pairs` gives the indices, 1 to n, which
name nothing: auto-lang warns `no Unicode script named 1`; on text
(`-M auto-lang=x`) or a boolean it fails (`bad argument #1 to 'for
iterator'`). Both filters' metadata of the wrong shape do this.

**panir**: each port wrote it by kind (Python a `match`, TypeScript `if`s,
Rust a `match`, Julia methods), from what Lua was found to do with
pandoc 3.14; no README says it.

**Proposed**: a line in each "Coming from Lua" metadata part: `pairs(v)`
is a map's entries (`v.content.items()`, ...); on a list or inlines, its
indices; on text or a boolean, an error. Low.

How common: 2 of 21 (auto-lang, amsthm).

## 40. `pandoc.utils.to_roman_numeral`

Found with amsthm.

**The original**: a part's number in a theorem's (`I.1`, with
`--top-level-division=part`) is `pandoc.utils.to_roman_numeral(n)`.

**panir**: has `stringify` of `pandoc.utils`, not this: each port wrote
the ten lines.

**Proposed**: nothing for now (one filter); if more of `pandoc.utils`
turns up in later rounds (`to_roman_numeral`, `normalize_date`,
`citeproc`'s helpers are pandoc's), a small module of them. Low.

How common: 1 of 21 (amsthm).

## Round 2 and 3: what the filters that read files and call pandoc found in panir

Round 2's and 3's filters (include-files, include-code-files, multibib,
amsthm's notes, until its ports followed the proposed v4.0.0 on
2026-11-18, and pantable: CSV and YAML in a code block, markdown
cells read with pandoc, an included file) were mostly about the hosts:
those gaps are in `HOSTS.md`. What they found in panir itself (45 and 46
pantable's; its options are YAML in the code block, not metadata, so
nothing of panir's metadata; its widths count code points as Python's
`len`, which each language has, so no text width was needed):

| # | gap | languages | filters hitting it (of 24) | status |
|---|---|---|---|---|
| 41 | pandoc.path's functions (`normalize`, `join`, `directory`, `is_relative`, `filename`), whose `normalize` no language's own path functions match | all | 1: include-files (every port has the five, 25 lines) | open |
| 42 | Lua's `tonumber` (and the integer `string.sub` takes) | all | 4: amsthm, list-table, include-files, include-code-files (each port has `lua_number`) | closed, panir#19 |
| 43 | a fragment's walk from a filter function doesn't know the conversion unless given it, where Lua's `PANDOC_READER_OPTIONS` and `FORMAT` are globals | Python, TypeScript, Julia (Rust's `ctx.apply` passes it) | 1: include-files (a nested include read with the document's reader options) | closed, panir#19 |
| 44 | Python's `MetaList([...])` and `Pandoc(meta, blocks)` take their items as arguments: a list passed is one item (a list in a list), silently | Python | 1: multibib | closed, panir#19 |
| 45 | a table from a grid of cells: panir has `Table`'s constructors only; a filter making one from rows with spans and row blocks writes the grid (covered cells, anchors) and the split into head, bodies and foot | all | 2: pantable (each port about 40 lines), list-table | open |
| 46 | pandoc's attribute syntax (`{#id .class key=value}`) has no parser: a filter reads `[]{...}` with pandoc's reader and takes the span's attributes, a call into pandoc, and none where there is no reader | all | 1: pantable (attributes of cells, rows and row blocks) | open |
| 47 | Python's `Meta`, the name a metadata function is annotated with, was `dict[str, MetaValue]` to a type checker: `meta["indent"] = True` ran and did not check | Python | 3: count-para, first-line-indent, scholarly-metadata (6 of the 11 errors of pyright's first real run) | closed, panir#20 |
| 48 | no public name for the lists a function is given (`NodeList[Block, ...]`, `NodeList[Inline, ...]`): annotated `list[Block]`, `blocks.walk(f)` doesn't check | Python | 1: include-files | open |

## 41. pandoc.path's functions

Found with include-files.

**The original**: `path.normalize(path.join({include_path, link.target}))`,
`path.directory(line)`, `path.is_relative`, `path.filename`: pandoc's,
Haskell's `System.FilePath` on POSIX. `normalize` drops `.` and doubled
slashes but keeps `..` (`nest/two/../up.md`); `join` gives the second
when it is absolute; `directory("file.md")` is `.`.

**panir**: nothing; each language's own path functions differ:
`os.path.normpath`, `node:path`'s `normalize` and Julia's `normpath`
resolve `..`, Rust's `Path::components` drops a trailing slash. Each port
has the five functions, about 25 lines, checked by the cases (links and
images of every shape, `nested`).

**Proposed**: `panir.path` (`pandoc.path`'s functions, as pandoc has
them, POSIX and Windows), in each language, beside `stringify`; or a
"Coming from Lua" bullet with the five lines that matter. Low until more
filters that rewrite paths come (pandoc-ext's include-files, diagram's
outputs, a typst or LaTeX includer).

How common: 1 of 24.

## 42. Lua's `tonumber`

Found with list-table and amsthm (round 1), again with include-files and
include-code-files.

**The original**: `tonumber(attr)` (`shift-heading-level-by`,
`startLine`, `counter_depth`): blanks around, hexadecimal, exponents;
`nil` for anything else, which the filter then treats as no value or
fails on. `string.sub(s, "4")` converts its argument the same way and
wants an integer value (`2.0` passes, `1.5` fails).

**panir**: nothing: amsthm, include-files and include-code-files each
have the same `lua_number` in every language (list-table a decimal-only
one), 8 lines each.

**Proposed**: `lua_number` (or `tonumber`) beside `to_roman_numeral`, as
a helper for ports, the same in the four languages. Low.

How common: 4 of 24.

**Closed** in panir#19: `lua_number` (TypeScript `luaNumber`) in the four
languages; the ports' copies are gone.

## 43. A fragment's walk doesn't know the conversion

Found with include-files.

**The original**: an included file's own includes are read inside a walk
started from the filter function (`pandoc.walk_block(pandoc.Div(...),
{...})`), whose functions read `PANDOC_READER_OPTIONS`, a global.

**panir**: `contents.walk(f)` (Python), `applyFilter(contents, f)`
(TypeScript) and `walk!(f, contents)` (Julia) start a walk whose
`ctx.conversion` knows nothing but what it is given; the port passes
`conversion=ctx.conversion` (Python), `format = ctx.conversion` (Julia).
Rust's `ctx.apply(x, f)` passes the conversion itself. Easy to forget:
the nested include then reads with no reader options and no error.

**Proposed**: a walk started from a filter function inherits the running
walk's conversion (Python and Julia: a context variable set while a
filter runs; TypeScript: the same, or `ctx.apply` as Rust has). Low.

How common: 1 of 24.

**Closed** in panir#19: a walk started inside a filter function knows the
conversion of the walk it is in, unless given one.

## 44. Python's constructors take their items as arguments

Found with multibib.

**The original**: `pandoc.MetaList{}` and `pandoc.Pandoc(blocks, meta)`
take a table and the blocks first.

**panir**: `MetaList(*items)`, `Pandoc(*blocks, meta=...)`: given a list,
`MetaList([a, b])` is a list holding a list (plain values convert), and
`Pandoc(meta, [blocks])` fails only later (`ASTTypeError`). The port
built its bibliography list as `MetaList(files)`: pandoc then looked for
one file named `test/secondary.bib test/primary.bib`.

**Proposed**: a "Coming from Lua" line (Lua's table arguments are
arguments here); perhaps a warning when the only argument of a list
constructor is a list. Low.

How common: 1 of 24.

**Closed** in panir#19: `MetaList([a, b])` is an error (and a type
checker's), with "Coming from Lua"'s line.

## 45. A table from a grid of cells

Found with pantable (list-table met it first, smaller).

**The original**: pantable's `TableArray` (a numpy array of cells, each
covered cell holding its span's text and anchor) and `PanTable`'s
`to_panflute_ast`: rows of the anchored cells only, split into the head,
each body's head and rows, and the foot by counts (`ms`, numpy's
`split`), every block with its attributes; panflute's constructors below.

**panir**: `Table`, `TableHead`, `TableBody`, `TableFoot`, `Row`, `Cell`
and their attributes, as pandoc's types; nothing that takes a grid. Each
port has a `Grid` (the texts, the spans, `put`, `shape`, `is_at`: 25 to
35 lines) and the row blocks by counts (8 to 12 lines), then builds the
nodes; Rust's struct literals take the most. list-table builds its table
from lists the same way, its spans from attributes.

**Proposed**: a helper beside the constructors, `table_of(rows,
head=1, bodies=[(head_rows, rows, row_head_columns)], foot=0)` (each
language's spelling), taking rows of cells (spans included, covered
positions absent) and the blocks' sizes, defaults for what isn't given;
or a "Building a table" example in each README with a grid of spans.
Low: two filters, and the arithmetic is the filter's own.

How common: 2 of 25 (pantable, list-table: the two `table` filters of
`SURVEY.md`).

## 46. pandoc's attribute syntax has no parser

Found with pantable.

**The original**: a markdown cell's first line may hold its attributes,
`{#id .class key=value}`, and the fancy table's first column the rows'
and blocks'; pantable reads each as the markdown `[]{...}` with pandoc
and takes the empty span's attributes (`Ica.from_panflute_ast`).

**panir**: no parser of the syntax; the ports do as the original, the
texts in the same `read_many` as the cells. Without a reader (a Rust
JSON filter, the TypeScript port under plain pandoc a process a text)
attributes cost what a cell does.

**Proposed**: `parse_attr(text)` (or `Attr.parse`) in each language, as
pandoc's markdown reads `{...}` (identifier, classes, quoted values,
`key=value`, `-` for unnumbered), checked against pandoc on a corpus;
or leave it to the reader and say so. Low.

How common: 1 of 25 (pantable).

## 47. Python: `Meta` didn't take plain values for a type checker (closed)

Found when pyright first ran on the ports: `pyrightconfig.json`'s
`include` (`*/python`) matched no file, so `npm run check` and CI had
checked no Python port since the first round (it is `**/python` now, with
libpandoc-python's sources beside for the ports that call pandoc). Of the
11 errors of its first real run, 6 were this.

**panir**: metadata converts plain values put into it (`doc.meta["draft"]
= True`, the README's), and `doc.meta` is typed so. But `Meta`, the name
a function is annotated with (`def read(meta: Meta)`, the README's too),
was `dict[str, MetaValue]`: in count-para (`meta["header-includes"] =
new`, a list), first-line-indent (`meta["indent"] = True`) and
scholarly-metadata (`meta["author"] = authors`, a list of maps) the
assignment ran and did not check.

**Closed** in panir#20: to a type checker `Meta` is the map the function
is given (`NodeDict[MetaValue, ...]`); a plain dict annotated `Meta` no
longer checks (multibib built two with `Meta(...)`: plain dicts now, which
is what they were).

How common: 3 of 25.

## 48. Python: no name for the lists a function is given

Found by the same run (include-files).

**panir**: a function for every list of blocks or inlines (`on_blocks`,
`on_inlines`), `doc.blocks` and a node's `content` are a `NodeList[Block,
...]` or `NodeList[Inline, ...]`: a list that checks and converts what is
put into it, with `walk`. Its second parameter (what converts) is
private, so a port annotates `list[Block]`, which is true and lets a
function be registered, but `blocks.walk(update)` then doesn't check
(include-files; it calls `update(blocks)` now, the same walk).

**Proposed**: public names, as `Meta` is for the metadata: `Blocks` and
`Inlines`. One thing to settle first: in Lua `pandoc.Inlines("text")` is
a constructor, and a name that is a plain `list` when running would make
`Inlines("text")` a list of characters; so either the names construct
(as Lua's do) or they are for annotations only and say so. Low.

How common: 1 of 25 seen; any typed Python filter that walks a list.
