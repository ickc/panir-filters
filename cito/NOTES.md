# cito

pandoc-ext/cito: a citation `@property:key` (a CiTO property, or one of
its aliases) is cited as `@key`, its key listed under the property in the
metadata's `cito_cites`; any other citation is listed under `citation`.
A `Pandoc` function walks the document with a `Cite` function, which
walks each citation's content with a `Str` function, then writes the
lists as metadata: a map of lists of strings, built from plain values.

## The original's behaviour, kept

- **Every citation is listed**, under `citation` if it has no property
  (an unknown prefix, `@x:y:z`, is part of the key: case `many`); the
  document's `cito_cites` is replaced, an empty map if it has no citation
  (case `none`).
- **The metadata's citations come first** (`doc:walk` walks it before the
  body): in `many`, the title's `t1` is listed before a note's `n1`. panir
  walks in the same order in every language (checked before porting).
- **Bug: the content is rewritten with the id as a Lua pattern**
  (`str.text:gsub(citation.id, cite_id)`). `.` is any character, `-`, `+`,
  `?` and `*` repeat the character before: `[@cites:a-b]`,
  `[@cites:aa+b]`, `[@cites:ab?c]` keep `cites:` in their content (the
  key is cleaned, the text not), and a key can rewrite its neighbour's
  text (`[@extends:x.y; @x.y; @xzy]` shows `[@x.y; @x.y; @x.y]`: case
  `patterns`). Hyphenated keys (`doe-2020`) are common. The ports
  translate the id into their language's regular expression, item by
  item as Lua reads a pattern: a character (`.` any, `%x` the character
  x), perhaps followed by `*`, `+`, `-` (fewest) or `?`; checked against
  Lua's `gsub` on 195 pattern and text pairs. They differ where Lua's
  patterns work on bytes (a `.` or a repeat next to a non-ASCII
  character), where an empty match falls (an id all of whose characters
  are optional: none pandoc reads), and on what pandoc's keys hold only in
  braces (`@{...}`): `%a`-like classes, `[sets]`, captures, anchors, taken
  literally here. For upstream: escape the id (`citation.id:gsub('%p',
  '%%%0')`).
- **The `Str` function returns a string** (the `gsub` result), so a
  string that has spaces becomes words and spaces, and an empty one
  nothing, as pandoc converts it; the ports return the string too, which
  panir converts the same way (checked: `"a b\nc"` is `Str`, `Space`,
  `Str`, `SoftBreak`, `Str` in both, `""` none).
- The `gsub` count the function also returns is ignored by pandoc in a
  typewise walk.

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| the table of properties and aliases | 155 lines | 80 (`ruff format`) | 52 | 136 (`rustfmt`) | 46 |
| `cito_cites` from plain values (a map of lists of strings) | `doc.meta.cito_cites = t` | `doc.meta["cito_cites"] = by_property` | `MetaMap(Object.fromEntries(byProperty))` | 14 lines (rustfmt's) of `MetaValue::MetaMap(... MetaValue::MetaList(... MetaValue::MetaString(k.into()) ...))` | `doc.meta["cito_cites"] = by_property` |
| the id as a Lua pattern | built in | 15 | 11 | 25 | 20 |
| the content's walk, from the `Cite` function | `cite.content:walk{Str = f}` | `cite.content.walk(rewrite(id, key))` | `applyFilter(cite.content, { Str })` | `ctx.apply(&mut cite.content, &mut Rewrite { .. })?` | `walk!((s::Str) -> ..., cite.content)` |

- **Building metadata from plain values** is one line where the
  language converts them (Python's `from_python` when setting a field,
  TypeScript's `MetaMap`, Julia's `convert`), as in Lua; Rust has no
  conversion: the nested constructors by hand (`GAPS.md` 23).
- **Python**: `rewrite(id, key)` makes the second filter (a closure per
  citation; one defined in the loop is ruff's B023); `re.sub` with a
  function, so the key is not a template.
- **TypeScript**: `replace(pattern, () => key)` likewise; no
  `RegExp.escape` in the `es2022` library, so a character class of the
  syntax characters; the regular expression is on UTF-16 code units.
- **Rust**: the one port with a dependency, `regex` (`NoExpand(&key)`,
  `regex::escape`), as no pattern matching is in the standard library.
  The `inline` function collects into the filter's struct; its `meta`
  function, called after every inline in the typewise order, writes
  `cito_cites`: no `Pandoc` function, no walk of the document by hand.
  `ctx.apply(...)?` works here, in an `inline` function. A `Str` becomes
  `panir::inlines(&text)`, the words Lua would make of the string.
- **Julia**: `walk!` with an anonymous function of one typed argument
  (`(s::Str) -> ...`) is the shortest sub-walk of the four; `replace(text,
  pattern => key)` takes the key literally. No regex escaping function:
  a backslash before ASCII punctuation.

Differential runs: the same AST in every language: 16 citations in
pandoc's citation test, 30 in its changelog, 476 in the textbook (none
with a CiTO property, all under `citation`, their content rewritten by
the id's pattern), and the made documents' cases.
