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
  it was before the walk (pandocfilters passes the original). The Python,
  TypeScript and Julia ports read `ctx.doc.meta`, which the top-down walk
  has already changed by then (the metadata comes first); it makes no
  difference to the result, as a field's replacement is the same whether
  it is made in the metadata or again in the body (case `kinds`). The Rust
  port clones the metadata first (below).
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
  Python's `RecursionError`. So do the Python port (panir's depth error,
  with the path) and the TypeScript one (at once, 0.1 s). **The Rust and
  Julia ports don't stop cleanly**: neither walk bounds how deep it goes
  (`GAPS.md`). With the usual 8 MB stack, Rust aborts on a stack overflow
  (no path, no message of panir's) and Julia was still running after 60 s
  (480 MB); with no stack limit (`ulimit -s unlimited`, as on the machine
  this was found on) Rust took 59 GB in two minutes and Julia 5.7 GB in
  30 s before they were killed. Not in `cases/`, so that CI doesn't run
  it; to see it:

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
- **TypeScript**: `ctx.doc?.meta[field]`: `doc` is optional in the types
  (it is `undefined` for a fragment), so a filter that only ever runs on
  documents still writes `?.`; a missing field has to be thrown by hand
  (`undefined`, not an error). `copy(value.content)` for the same reason
  as in Python. Attributes given as an object (`{ class: "interpolated",
  field }`), in its keys' order.
- **Rust**: a function's `Ctx` has no document, so the filter holds a
  clone of the metadata, made in `main` before the walk. A field that
  isn't there can only `panic!`: a filter function can't return an error.
  Building the span is the longest part (`Span { attr: Attr { attributes:
  vec![("class".into(), ...)], ..Attr::default() }, content }.into()`).
- **Julia**: the shortest port; `ctx.doc.meta[field]` throws `KeyError`
  as Python does, and `Span(deepcopy(value.content); attributes = [...])`
  takes pairs.

Differential runs (pandoc's test documents, eight real documents): the
same AST in every language, and the same failure on the GitLab reference.
