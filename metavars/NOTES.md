# metavars

pandocfilters' `examples/metavars.py`: a word that is all `%{field}`
becomes that metadata field's value (inlines in a span, or a string).

## The original's behaviour, kept

- **Top-down, and what it puts in is walked.** pandocfilters calls the
  function on a node, then walks the replacement's children, so a field
  may name another (`place: "%{city}"` gives a span in a span: case
  `kinds`). panir's `topdown` order does exactly that; the ports ask for
  it.
- **The metadata is walked too**, and the function reads the metadata as
  it was before the walk (pandocfilters passes the original). The ports
  read `ctx.doc.meta` (Rust `ctx.meta()`), which the top-down walk has
  already changed by then (the metadata comes first); it makes no
  difference to the result, as a field's replacement is the same whether
  it is made in the metadata or again in the body (case `kinds`).
- **Bug, kept: a missing field stops the filter.** `meta.get(field, {})`,
  then `result['t']`: a `KeyError`. Every port fails too (case `missing`).
  It happens on a real document: GitLab's GraphQL reference has
  `%{linkStart}` and `%{linkEnd}` (its own placeholders) as words, and the
  original fails on it; so do the ports. For upstream: leave unknown
  fields alone.
- **Bug, kept: `class` is an attribute.** pandocfilters' `attributes()`
  takes `classes` (a list), not `class`, so the span gets the key-value
  `class="interpolated"` and no class. HTML shows no difference; other
  writers do. The ports make the same span. For upstream.
- **Bug, kept, not as a case: a field naming itself never ends.**
  `loop: "%{loop}"` and `%{loop}` in the text: the original stops with
  Python's `RecursionError`. So does every port now, at once (0.1 s or
  less, Julia's start aside): Python with panir's depth error, TypeScript
  with its own, Julia with `ASTDepthError` at 256 levels, each with the
  path. Rust stops sooner and says something else: `no metadata field
  loop`, on `meta.loop.content[0]`, because while a metadata value is
  walked `ctx.meta()` is the *other* fields (the value is out of the map
  while it changes): the field the filter is in looks missing. Fields
  naming each other (`a: "%{b}"`, `b: "%{a}"`) the same: `no metadata
  field a`. A true error in a misleading message (`GAPS.md`, 11). Before
  panir#14 (with no depth limit) Rust overflowed its stack and Julia ran
  without end. Not in `cases/`, as its message differs by language; to
  see it:

      printf -- '---\nloop: "%%{loop}"\n---\n\n%%{loop}\n' | pandoc -t json | target/release/metavars html

- The pattern is written `'%\{(.*)\}$'`, an invalid escape in a normal
  string: Python 3.12 and later warn about it on every run. For upstream:
  a raw string.
- `$` in Python's pattern also matches before a final newline, which a
  `Str` from a reader never has; the TypeScript and Julia patterns don't,
  and the Rust port checks the prefix and suffix. Not a difference on any
  document from pandoc.

## By language

- **Python**: `match ctx.doc.meta[field]` with `case MetaInlines(content)`
  reads well. The metadata's inlines go into the body as a copy
  (`copy.deepcopy`): a node belongs in one place, and Python has no copy
  of panir's own to reach for (TypeScript has `copy`).
- **TypeScript**: `ctx.doc.meta[field]`: under `runFilter` `doc` is a
  `Pandoc` (it was `Pandoc | undefined`, so the port wrote `?.`; closed in
  panir#14). A missing field has to be thrown by hand (`undefined`, not an
  error). `copy(value.content)` for the same reason as in Python.
  Attributes given as an object (`{ class: "interpolated", field }`), in
  its keys' order.
- **Rust**: `ctx.meta()` and `return ctx.fail(...)` (panir#14) replaced a
  clone of the metadata held by the filter and a `panic!`: 40 lines to
  28, and the error now has the node's path. `ctx.meta()` is an `Option`
  (`None` in a fragment), so `ctx.meta().and_then(|m| m.get(field))`, one
  `let ... else` for both. The span's attributes are `Attr::new("", &[],
  &[("class", "interpolated"), ("field", field)])`: short, and the empty
  `&[]` needs no annotation. The surprise is above: inside the metadata,
  `ctx.meta()` lacks the field being walked.
- **Julia**: the shortest port; `ctx.doc.meta[field]` throws `KeyError`
  as Python does, and `Span(deepcopy(value.content); attributes = [...])`
  takes pairs.

Differential runs (pandoc's test documents, eight real documents): the
same AST in every language, and the same failure on the GitLab reference.
