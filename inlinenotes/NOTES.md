# inlinenotes

pandocfilters' `examples/inlinenotes.py`: for `markdown` output, a note
becomes the raw `^[`, its paragraphs' inlines joined by a space, and `]`
(Markdown's inline note); "if an input note had any blocks other than
paragraphs, the note is left as is". Other formats (`html` among the
cases') are left alone.

## What the original does, and what the ports do

The original collects a note's inlines with a second walk (`query`) over
the note's JSON, which appends the value of every node it meets whose
content is a JSON list, then checks that everything appended is a node.
That is the docstring's rule for most notes, and something else for some:

- **Kept: the rule.** A note of paragraphs is joined (case `notes`: one
  paragraph, two, a link, code, a quote, a line break, notes in a row, a
  note in a heading, a note in the metadata's title); a note with code, a
  list, a quote, a heading or a div stays as it is (`blocks`).
- **Kept: what falls out of it at the block level.** A `Plain` is joined
  without a space before it (a space goes before a `Para` only), and a
  horizontal rule is dropped, its note joined (`blocks`, case `plain`,
  from JSON); an empty note gives `^[]`. The ports do the same, each with
  a comment; for upstream, the rule is probably meant to be dropped
  neither silently nor at all.
- **Not kept, a known difference: what the JSON walk does inside a
  paragraph.** pandocfilters' walk doesn't give the query a paragraph's
  own inlines (they are what the query returned, whose fields it walks),
  but it does give it the inlines inside those. So an emphasis in an
  emphasis, or in a link, is appended again (`An *outer _inner_ outer*`
  becomes `^[An *outer *inner* outer* emphasis.inner]`), and a note
  whose emphasis or strong holds a link, code, a span, a quote or math
  (whose JSON content holds attributes or strings) is left as is, while
  the same link or code directly in the paragraph is joined. Case
  `nesting`, in `known.tsv` for `markdown` in every language. Doing the
  same would mean walking panir's AST as pandocfilters walks JSON, which
  no port of a filter should; the ports follow the docstring. For
  upstream: collect `Para` content only, not by a walk.

## By language

All four are the same loop over the note's blocks: a paragraph appends
(after a space if something came before), a plain appends, a rule
nothing, anything else returns, leaving the note. The function decides
before it changes anything, so returning early is safe in every language.

- **Python**: `match` with `case Para(content)` reads as the rule does.
- **Rust**: checks every block first (`blocks.iter().all(...)`), then
  takes them (`std::mem::take`): moving the inlines out of the note needs
  the decision made before anything moves.
- **TypeScript, Julia**: a `switch` and an `if` chain; nothing missing.

Differential runs (pandoc's test documents and eight real documents, which
have notes, few with emphasis around a link; five documents made of every
filter's cases but `nesting`): the same AST in every language.
