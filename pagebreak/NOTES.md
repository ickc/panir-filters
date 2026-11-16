# pagebreak

pandoc-ext/pagebreak: a raw TeX block holding only `\newpage` or
`\pagebreak` becomes the output format's page break; optionally a
paragraph of a form feed alone, or of the command as text. A `Pandoc`
function reads the configuration (nested options, the environment), then
walks the document with a second filter. The first filter here with
per-case environment variables (`cases/env.env`: `run.py`).

## The original's behaviour, kept

- **It reads `html-class` and `odt-style` from the top of the metadata**,
  not from `pagebreak` as its README documents:
  `pagebreak_from_config(doc.meta)` is passed the metadata, not `config`
  (case `html-class`: `pagebreak.html-class` is ignored). For upstream:
  pass `config`.
- **The environment** (`$PANDOC_PAGEBREAK_HTML_CLASS`,
  `$PANDOC_PAGEBREAK_ODT_STYLE`) counts only where the metadata's field
  is missing or false: `config['html-class'] and stringify(...) or
  os.getenv(...)`. An empty `html-class: ""` is set (Lua's `""` is
  true), so the environment is not read, and the empty class is not used
  either (case `env-meta`).
- **A configuration of `true`** (`pagebreak: true`, `break-on: true`)
  fails: Lua indexes a boolean (cases `error-config`, `error-break-on`).
  Text, a list or a string have no fields: the defaults (`config-text`,
  `config-list`, `config-string`). `form-feed: false` is off, the text
  `"no"` on (`truth`).
- **The format tests are Lua patterns**: `format:match 'html.*'` (any
  format holding `html`), `'ms'` (any holding `ms`), and raw TeX is left
  alone only when the format ends in `tex` (`FORMAT:match 'tex$'`):
  LaTeX, but not Beamer, where a raw `\newpage` becomes a paragraph of a
  form feed (every case, format `beamer`). For upstream: test Beamer too.
- **Raw blocks of any format containing `tex`**, ConTeXt's included
  (`el.format:match 'tex'`), are read.
- **The walk covers the metadata**: `doc:walk` is the whole document, so a
  `\newpage` in `abstract` (blocks) becomes a page break too (case `raw`).
  Each port walks the document, not its blocks.
- One page-break element is made once and returned for every marker: in
  Lua each return is a copy.

## Nodes in two places

The ports make the page break once, as the original, and return a copy
each time (`copy.deepcopy`, `copy`, `deepcopy`, `.clone()`). Returning the
node itself from each call puts one node in many places:
`CHECK_UNIQUE=1` fails on it (tried: 10 cases fail in Python). The
READMEs' "As in Lua, what a function returns are values: `[el, el]` puts
two nodes in the document, the second a copy" can be read as covering
this; it doesn't: a value returned by two calls is in two places
(`GAPS.md` 35).

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| the configuration's tests (`(meta.pagebreak or {})['break-on']['form-feed']`, failing on `true`) | 4 | `lookup(config, "break-on", "form-feed")`, and a test for `true` | the same | `lookup(&config, &["break-on", "form-feed"])` of the configuration removed, as Python (since panir#17; before, read from `&doc.meta` before removing it: gap 29) | `lookup`, as Python |
| a text option, else the environment (Lua's `a and b or c`) | 2 | `stringify(v) if v else os.environ.get(...)` | `truthy(v) ? stringify(v) : process.env[...]` (a needless `!` before: gap 26) | `given(meta.get(k)).map(stringify).or_else(...)`, `std::env::var` (panir#17) | `truthy(v) ? stringify(v) : get(ENV, ...)` |
| the second filter, with functions only in some cases | a table with `nil`s | functions registered under an `if` | a `Filter` object filled in | one `block` method, flags in the struct | two methods of one inner function |
| walking the document from the `Pandoc` function | `doc:walk{...}` | `doc.walk(breaks)` | `applyFilter(doc, breaks)` | `ctx.apply(doc, &mut breaks)`, the function's last expression | `walk!(breaks, doc)` |
| a raw block | `pandoc.RawBlock(f, t)` | `RawBlock(f, t)` | `RawBlock(f, t)` | `Block::raw(f, t)` (panir#17; a helper before, gap 30) | `RawBlock(f, t)` |

- **#16's API, in use**: Lua's truth carried the options in every
  language (`if lookup(config, "break-on", "form-feed"):` in Python is the
  Lua line); `lookup` read the nested option in one call, from the
  configuration once removed from the metadata (Python, TypeScript,
  Julia; Rust since panir#17, which closed gap 29: before, Rust's
  `lookup` started from a `Meta` only, so the port read `&doc.meta`
  first, then removed `pagebreak`). `lookup` returns
  nothing where Lua fails on `true`, as documented: each port tests for
  `true` itself (one line). Rust's `pandoc` returning `Option<()>` lets
  `ctx.apply(doc, ...)` be its result, and `return ctx.fail(...)` work
  there.
- **Python**: `re.fullmatch` for Lua's `^...$` (Python's `$` also
  matches before a final newline); the paragraph check a `match` on
  `[Str(text)]`.
- **TypeScript**: `truthy` is not a type guard, so the port had
  `stringify(meta[key]!)` after it (gap 26); `stringify` takes any value,
  so the `!` was never needed, and is gone. (`given` would do too.) `ctx.format ?? ""` for Lua's `FORMAT`.
- **Rust**: no regular expressions needed: `strip_prefix` and four
  suffixes are the pattern `^\\newpage%{?%}?$`. A raw block is
  `Block::raw(format, text)` (panir#17; a four-line struct literal, so a
  helper, before: gap 30).
  `"\u{c}"` for Lua's `'\f'`.
- **Julia**: the second filter is one inner function with a method for
  `RawBlock` and one for `Para` (closures may have several methods);
  `get(ENV, ...)`.

## In the hosts

The same in every host (`run.py host`) but the wasm ones: a wasm filter
is given only the variables pandoc gives filters, not the process's, so
the Rust port built for wasm ignores `PANDOC_PAGEBREAK_HTML_CLASS` and
`PANDOC_PAGEBREAK_ODT_STYLE` in pandocrs, pandocpy, pandocjl and Node
(cases `env` and `env-meta`, `known.tsv`); inside a directory sandbox the
Lua original likewise sees only `SOURCE_DATE_EPOCH` (`HOSTS.md`, H8).
