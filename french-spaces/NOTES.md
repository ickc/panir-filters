# french-spaces

chdemko/pandoc-latex-french-spaces, a panflute filter: for `latex` and
`beamer`, a space after `«`, `“` or `‹`, or before `;`, `?`, `!`, `»`,
`”` or `›`, becomes `\thinspace{}`, and one before `:` becomes `~`, as raw
TeX.

## The original's behaviour, kept

- **Only a `Space`, between `Str`s of the same list**: a soft break, a
  non-breaking space (part of a `Str`), a space next to emphasis or code
  are left (case `edges`). The opening marks win over what follows (`«
  ;`). A `Str` is tested by its last or first character, so `«*after*`
  and `a «inside» word` are left too.
- **Everywhere**: in emphasis, links, headings, table cells, and the
  metadata's title (panflute walks the metadata; `edges`).
- `\thinspace{}` and `~` as `RawInline` of format `tex`; empty `Str`s
  next to a space (from JSON; `empty-str`) are not punctuation.
- Its tests' inputs are case `tests`.

## By language

The filter needs a space's neighbours, panflute's `elem.prev` and
`elem.next`.

- **Python, TypeScript, Julia**: `ctx.prev` and `ctx.next` are exactly
  that. panflute's see the list as it was before the walk; panir's as it
  is now, earlier spaces perhaps replaced already; only `Str`s are tested,
  so it makes no difference. TypeScript's are typed `Inline`, so `prev?.t
  === "Str"` narrows to read `.text`.
- **Rust**: a function's context has no siblings (as the README says: the
  function holds its node `&mut`), so the port is an `inlines` function,
  a loop over each list of inlines looking at `xs[i - 1]` and `xs[i + 1]`.
  As short as the others in substance, and the README's design held up;
  `first-line-indent` (the previous block) will want the same with
  `blocks`.
- **Python**: `str.endswith` and `startswith` take a tuple of marks.

The original's `pyproject.toml` says Python 3.10 to 3.13; it runs under
3.15 with panflute 2.3.1 here, and panflute's reading and writing of the
JSON changes nothing on the documents tried (an `html` run, where the
filter does nothing, gives its input back).

Differential runs (pandoc's test documents; eight real documents, three
of them with a space before `:`, `;`, `?` or `!`, 22 in all, but none
with French quotation marks; five documents made of every filter's
cases), for `latex`, `beamer` and `html`: the same AST in every language.
