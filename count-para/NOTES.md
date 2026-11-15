# count-para

cysouw/count-para: each paragraph of the body gets a number (a span
before it, or a LaTeX margin note after its first word), per chapter if
asked; a paragraph opening with `{#name}` can be cited (`[@name]`), and
the citation becomes a link to the number. Four filters in turn, as the
original returns them: `header-includes`, the options, the numbering
(`Pandoc`), the citations (`Cite`), sharing state as Lua's upvalues: the
longest list of filters in round 1.

## The original's behaviour, kept

- **`header-includes` is appended to as a Lua table**: a list grows; a map
  becomes a list of the new entries (a table with items converts to a
  list, its keys dropped: case `header-includes-map`); text, a string or a
  boolean fail in HTML and LaTeX, where something is added (Lua can't
  append to them: cases `header-includes-inlines`, `-string`, `-false`);
  and a document without `header-includes` gets an empty list in any
  other format (`pandoc.MetaList{nil}`). For upstream: append with
  `pandoc.List` of whatever is there.
- **The enclosure is cut by bytes**: `enclosing:sub(1,1)` and `sub(2,2)`
  are the first two bytes, so `«»` makes two halves of `«`, which pandoc
  reads back as U+FFFD each (case `enclosing-bytes`); `len() == 1` is one
  byte. Each port takes the bytes the same way (`byte(text, i)`: an ASCII
  character, else U+FFFD), so the AST is the same. For upstream:
  `utf8.offset`, or take the enclosure as two fields.
- **A name is found anywhere in the first inline's text**, with Lua's
  greedy `{#(.*)}`: `Text{#inside}here:` names the paragraph `inside` and
  drops the whole word; `{#a}{#b}` is the name `a}{#b` (case `basic`);
  emphasis around it counts (its text).
- **Errors**: an empty paragraph, and a paragraph of a name alone, fail
  (Lua indexes the missing first inline: cases `error-empty`,
  `error-alone`); a citation of nothing would too.
- **Only the body's own paragraphs** are numbered (not in a quote, a div,
  a list), and not one opening with an image; chapters are level-one
  headings without `unnumbered` as their first class, and only with
  `resetAtChapter` (case `reset`). Paragraphs before the first chapter
  are chapter 0.
- **Citations are replaced everywhere**, the metadata's too (the title,
  case `basic`), the whole citation by its first id's link, prefix,
  suffix and the other ids dropped; an id that names no paragraph is left.
  A citation of a later paragraph works (the numbers are all made first).
- **Options**: `resetAtChapter` and `addPageNr` by Lua's truth (`"no"` is
  true, case `options-text`); the others as text; `refName: "#"` is `\#`
  in LaTeX (`refname`). A number's text becomes the span's or link's
  inlines as Lua converts a string (its words and spaces).

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| four filters, sharing state | `return {{Meta=..}, {Meta=..}, {Pandoc=..}, {Cite=..}}`, upvalues | `main([formatting, settings, numbering, references])`, a dataclass instance | `runFilter([...])`, module variables | four `apply`s, each filter borrowing what the ones before made (`&settings.0`, `&numbering.numbers`) | `run_filter([...])`, a `const` mutable struct and a `Dict` |
| `header-includes` by its kind | 9 | a `match`, 9 | 4 | a `match`, 13 | 8 |
| an option by Lua's truth | the value kept, tested later | `bool(value)` | `truthy(value)` | `truthy(Some(v))` (gap 27) | `truthy(value)` |
| the name in the first inline (`{#(.*)}`) | `s:match` | `re.search`, `re.DOTALL` | a regular expression, `[\s\S]` | `find` and `rfind`, no regex crate | `match(r"..."s, ...)` |
| a byte of a string, as pandoc reads it back | `s:sub(i, i)` | `.encode()[i:i+1].decode(errors="replace")` | `TextEncoder`, `TextDecoder` | `String::from_utf8_lossy` | `codeunit`, by hand |

- **#16's API, in use**: the list of filters reads as the original in
  Python, TypeScript and Julia (the state module-level, as Lua's
  upvalues); in Rust the four `apply`s in `main` are the README's
  example four times, the state passed by reference from one filter to
  the next: clearer than Lua's globals. `items` doesn't fit
  `header-includes` here: the original appends to the list in place and
  fails on what isn't one; each port matches on the kind. Lua's truth
  (`bool`, `truthy`) for the two boolean options; `Option<()>` lets the
  Rust `inline` function end on `self.numbers.get(name)?` (no number:
  keep the citation).
- **Python**: the CSS has a space at the end of two lines, which an
  editor strips: `\x20` there, in TypeScript too.
- **TypeScript**: a metadata field "present" is `!== undefined` (Lua's
  `~= nil`), not its truth; `if (meta.x)` would be right here only by
  accident.
- **Rust**: `Span { attr: Attr::new(&number, &["paragraph-number"], &[]),
  content: panir::inlines(&shown) }` and a `Link` with `Target::new`; the
  raw inline and the raw blocks as metadata are helpers (gap 30).
- **Julia**: `insert!(content, 2, x)` at the end of a one-inline
  paragraph works, as Lua's `table.insert`.

No copies: every number, span, link and raw node is made anew.
