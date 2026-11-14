# gitlab_markdown

pandocfilters' `examples/gitlab_markdown.py`: a code block whose first
class is `math` becomes a paragraph of display math (GitLab's ```` ```math
````), and inline math loses the backticks at its ends (GitLab's
`` $`x`$ ``).

## The original's behaviour, kept

- **The first class only**: `{.numbered .math}` stays code; `{#eq .math
  .numbered key=value}` becomes math, its identifier, classes and
  attributes dropped (case `edges`). An empty math block gives empty math.
- **All the backticks at either end** go (Python's `strip("`")`): one side
  only, several, none in the middle; display math keeps its backticks; a
  `math` class on inline code does nothing (`edges`). Math in the metadata
  is stripped too.
- **The original writes `{"t": "DisplayMath", "c": []}`**, a constructor
  without fields given an empty list. pandoc reads it as `DisplayMath`, so
  it is the same AST; but its JSON is not pandoc's. The harness now takes a
  Python original's output as pandoc reads it (`pandoc -f json -t json`;
  `run.py`), as it always did a Lua original's; no case had to be marked
  known for it.

## By language

Two functions in each, nothing missing. Rust's `trim_matches('`')`,
Julia's `strip(text, '`')` and TypeScript's ``replace(/^`+|`+$/g, "")``
are Python's `strip`. TypeScript imports panir as a namespace
(`pandoc.Math(...)`), as its README advises: a named `Math` would hide
JavaScript's in the module. Rust moves the code's text into the math
(`std::mem::take`).

Differential runs (pandoc's test documents, which have inline math, eight
real documents, five documents made of every filter's cases): the same AST
in every language.
