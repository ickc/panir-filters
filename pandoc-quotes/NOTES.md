# pandoc-quotes

pandoc/lua-filters' pandoc-quotes.lua: quotations (`Quoted`) become their
content between the marks of the document's language, given by
`quot-marks`, `quot-lang` or `lang`, from a table of 72 languages
(`QUOT_MARKS_BY_LANG`) that `quot-marks-by-lang` extends. Two filters, as
the original returns them: one reads the metadata, the other rewrites the
quotations.

The quotation marks are data, but no file: the original keeps its table
in the script (no `require` but pandoc's `text` module, no file read), so
each port has the same table, as a dict, an object, a slice of pairs, a
`Dict`. Pure AST, round 1.

## The original, fixed for pandoc 3 (the cases' oracle)

Under pandoc 3.14 the original fails on every document that sets
`quot-marks` or `quot-marks-by-lang`: it tests a metadata value's type
with `meta.t == 'MetaInlines'`, and since pandoc 2.17 a metadata value in
Lua has no `t` (it is `Inlines`, a `List`, a string...: checked with
3.14). So "neither a string nor a list", then a warning, and the warning
itself fails (below). Porting that would leave the filter's main options
unported and untested. So the cases are what the original makes with
those two tests written as pandoc 3 needs (`pandoc.utils.type(meta) ==
'Inlines'`, `== 'List'`): `upstream/pandoc-quotes-pandoc3.lua`, the
original with those two lines changed (`ORIGIN` says so; `diff` shows
it), under which its own tests pass again. For upstream: those two lines.

## The original's behaviour, kept

- **A warning is a failure**: the script sets `_ENV` to its module, which
  has no `string`, so `warn`'s `string.format` fails, under any pandoc.
  Every warning of the original stops the conversion: a bad `quot-marks`
  (not four, or neither a string nor a list: cases `error-count`,
  `error-list`, `error-type`), a bad entry of `quot-marks-by-lang`, even
  when another language is used (`error-by-lang`), an unknown language
  (`error-lang`). The ports fail with the warning's text. For upstream:
  `local string = string` with the other locals.
- **Precedence**: `quot-marks`, else `quot-lang`, else `lang`, each
  counting only if set and not false (case `false`); `quot-marks-by-lang`
  is read first, whatever is used.
- **Four marks**: `MetaInlines` of four characters (Unicode characters:
  pandoc's `text.len`), or a list of four, each its text (case `list`: `lu`
  and `li'u`). A `MetaString` is neither (the original tests for
  `Inlines`): `-M quot-marks=...` fails. YAML reads each mark as markdown:
  `'>>'` is a block quote, whose text is empty (case `prefix`).
- **The language**: as given (`de-CH`), then its letters up to the first
  other character (Lua's `%a`, ASCII: `de-AT` is `de`; case `region`,
  `lookup-void-country`), then a region of it. A language not starting
  with a letter fails (`error-letters`: Lua concatenates `nil`).
- **Bug: the region is found with a Lua pattern** `'^' .. lang .. '-'`,
  in which `-` is not a hyphen but "the letter before, as few times as
  possible": the pattern matches any key starting with the language but
  its last letter. `xxx-yy` finds `xx-YY` (case `region-pattern`); `deu`
  finds `de` or `de-CH`, `ena` `en`, `en-GB`, `en-US`...: whichever Lua's
  `pairs` meets first, and Lua 5.4 seeds its string hashes anew each run,
  so the original gives German or Swiss marks at random (6 to 4 in ten
  runs). The ports take the first in their own table's order; no case
  holds an ambiguous one. For upstream: `'^' .. lang .. '%-'`, and a
  sorted search.
- A quotation's marks are `Str`s around its content, spliced; inner ones
  first (a typewise walk), so nested quotations get the secondary marks
  inside the primary (case `nested`), and the metadata's too (the title).

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| the table (72 languages) | 74 lines | 74 | 74 | 74 | 74 |
| four marks from a string or a list (`get_quotation_marks`) | 19 | 11 | 8 | 10 | 4 |
| `quot-marks-by-lang`: a map, else an error | 11 | 8 | 8 | 12 | 8 |
| a field, missing or false (`GAPS.md` 18) | built in | `given`, 2 lines → none (`meta.get("quot-lang") or meta.get("lang")`, as Lua) | 1 → `truthy` | 3 → `truthy` | 3 → `truthy` |
| the field, then the language: as given, its letters, a region | 18 | 13 | 10 | 28 | 11 |
| two filters, metadata then quotations | `return {{Meta=...}, {Quoted=...}}` | a `Pandoc` function walking the document with a second filter → `main([configure, quotes])` | `runFilter([configure, quotes])`, as Lua | two `apply` in `main` | a `Pandoc` method calling `walk!` → `run_filter([configure, insert_marks])` |

- **Two filters in turn** is the original's shape, and needed: in one
  typewise walk the metadata comes after the inlines, and one top-down walk
  would leave a quotation directly in a quotation unrewritten (what a
  top-down function returns is not given to a function again). TypeScript
  (`runFilter` takes an array) and Rust (`main` applies one filter, then
  the other) say it as Lua does. Before panir#16, Python's `Filter.main`
  and Julia's `run_filter` took one filter, so those ports had a `Pandoc`
  function that read the metadata and walked the document with the second
  (`GAPS.md` 20); now `main([configure, quotes])` and
  `run_filter([configure, insert_marks])`, as Lua, the marks a module
  variable the first sets (a `global` in Python, a `Ref` in Julia).
- **Python**: `QuoteType.DoubleQuote` (an enum), `quoted.quote_type`
  (Lua's `quotetype`). A metadata value's truth is Lua's since panir#16:
  `if marks := meta.get("quot-marks"):` and `or` are the Lua lines.
- **TypeScript**: as the original; the marks a module variable the first
  filter sets. `[...stringify(v)]` splits into characters (code points).
  `truthy` is a `boolean`, not a type guard: after `if (truthy(marks))`
  `marks` is still `MetaValue | undefined`, so `marks!` (the port's own
  `given` was `v is MetaValue`; `GAPS.md` 26).
- **Rust**: `Configure` (with `meta`) keeps the marks in its struct;
  `main` applies `InsertMarks` only if there are some. `marks.try_into()`
  makes the four-element array or the "not four" error in one line; the
  table a `BTreeMap`, so a region is the first in key order. Since
  panir#16 `meta` returns `Option<()>`: `Err(e) => return ctx.fail(e)`.
  `truthy` takes an `Option<&MetaValue>`, as `meta.get` gives; a value in
  hand is `truthy(Some(m))` (`.filter(|m| truthy(Some(m)))`; `GAPS.md`
  27).
- **Julia**: methods by kind for the marks; `findfirst(f, dict)` tests the
  values, not the keys (the first try failed on that): a comprehension over
  the pairs instead.

Differential runs: the same AST in every language. Of the real documents
only the Wikipedia article sets a language (`lang: en`: its eight
quotations get English marks); two made documents fail in the original
and every port alike (`quot-marks` as a list of one string, not four;
`quot-marks-by-lang` as a list of one map, not a map).
