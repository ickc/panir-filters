# theorem

pandocfilters' `examples/theorem.py`: a div with the class `theorem`
becomes, for `latex`, a LaTeX `theorem` environment (labelled with the
div's identifier, if any); for `html` and `html5`, the same div holding a
numbered `<dt>Theorem n</dt><dd>`...`</dd></dl>` in raw HTML. Other
formats (`markdown` among the cases') are left alone.

## The original's behaviour, kept

- **The format by name**: only `latex`, `html` and `html5`; `beamer`,
  `html4` or `context` get nothing. The cases are run for `html`,
  `html5`, `latex` and `markdown`.
- **Numbered in document order**, the metadata first (a theorem in the
  abstract is 1: case `meta`); a theorem in a theorem after its own
  (`edges`). pandocfilters walks top-down, and so do the ports.
- **In LaTeX, a theorem directly in another is not converted** (`edges`):
  the original returns a list, spliced in, and pandocfilters walks the
  spliced blocks' fields, not the blocks themselves. One in a div in a
  theorem is. panir's top-down order does the same (which its READMEs
  don't quite say: `GAPS.md`, 13), so the ports splice as the original
  does. In HTML the original returns a new div, whose
  content is walked, so nested ones are numbered.
- **The HTML is unbalanced**: `</dl>` closes a `<dl>` never opened. For
  upstream.
- A span with the class is left alone; so is a div of another class
  (`edges`). An empty theorem gets the markers alone.

## By language

- **HTML, changed in place**: the original builds a new div with the same
  attributes; the ports add the raw blocks to the div's content and return
  nothing, which panir's top-down walk then walks into, as pandocfilters
  walks the new div. Equivalent, and shorter.
- **The counter**: Python `itertools.count` (no `global`), TypeScript a
  module `let`, Rust a field of the filter's struct, Julia a `const Ref`.
- **Rust**: `div.attr.has_class("theorem")` (panir#14) where the others
  test the classes' list; a `raw(format, text)` helper, as building a
  `RawBlock` is four lines in rustfmt; `div.content.splice(0..0, ...)` to
  prepend; `blocks.append(&mut div.content)` moves the content into the
  spliced list. The longest port (49 lines), from rustfmt's layout.
- **Python**: `div.content[:0] = [...]` prepends, checked.

Differential runs (pandoc's test documents, eight real documents, which
have no theorem divs, and five documents made of every filter's cases,
which do, at every depth): the same AST in every language and format.
