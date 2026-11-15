# first-line-indent

pandoc/lua-filters' first-line-indent: a paragraph of the body right
after a block quote, a list, a code block, a definition list or a rule
(the list is configurable) gets `\noindent` in LaTeX, or a marker div
before it in HTML; `\indent` or `\noindent` opening a paragraph is kept
and marked. The metadata's `indent` is set, and `header-includes` gets
CSS (HTML) or the indent's size (LaTeX). Two filters, as the original
returns them: `Meta` (the options, the metadata), then `Pandoc` (the
body), sharing the options as module state: a test of a list of filters
and of `header-includes` "whatever shape it has".

## The original's behaviour, kept

- **Its `Meta` function returns `nil` unless it set `indent`**, and Lua
  then drops everything it changed: with `indent: false` or
  `set-metadata-variable: false`, no `header-includes` either (cases
  `indent-false`, `no-variable`). The trap of `GAPS.md` 4, on the
  metadata this time: the ports decide first (`indent` set or not), and
  change after. For upstream: return `meta` whenever it changed.
- **`header-includes`, as the original builds it**: the new block first,
  then what was there (a list's items, any other value alone, nothing if
  missing or false), all made blocks: `pandoc.MetaBlocks{...}` converts
  each value as Lua does where a block goes. Text is a `Plain` of it
  (`-M header-includes=a b`: its words), a map an empty `Plain` (a Lua
  table without items), blocks, a list or `true` fail (cases
  `header-includes-*`, `error-header-includes-*`). The result is
  `MetaBlocks`, where it was a list.
- **Lua's patterns as format tests**: `FORMAT:match('html*')` is `htm`
  followed by any `l`s, so any format holding `htm`; LaTeX's is `latex`
  (not Beamer).
- **The size goes through `string.gsub` as a replacement**, where `%` is
  Lua's: `%1` (`%0`) is the matched `SIZE`, `%%` a percent sign, any other
  `%` an error. So `size: 5%` fails in HTML, and works in LaTeX, which
  concatenates (case `size-percent`); `%1x%%` is `SIZEx%` (`size-capture`).
  Each port has a small `lua_replacement` for that. For upstream: escape
  `%` in the size, or concatenate.
- **The CSS's `header + p { text-indent: ; }`** (an empty value) is kept.
- **Options**: only `false` turns one off (`auto-remove: "no"` is on,
  case `options-text`); `remove-after` adds, `dont-remove-after` removes
  names, each text (one name) or a list (a name each), anything else
  ignored; `size` is any value's text (`false` is `false`, case
  `size-false`). `first-line-indent: true` fails (Lua indexes a boolean),
  text has no options (`config-text`). `indent: "no"` is not false:
  `indent` becomes true (`indent-text`).
- **Only the body's own blocks** are read: a div's paragraphs keep their
  indents, and a div resets the "after" state as any block not in the
  list (case `defaults`). A paragraph opening with an image is a
  paragraph; a figure is not.
- **Explicit commands are inlines** `RawInline tex "\indent "` (with the
  space the markdown reader keeps): `\noindent  e`, two spaces, is not
  one (case `explicit`); a raw block `\indent` alone is a block like
  another.

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| two filters, sharing the options | `return {metadata_filter, body_filter}`, a module table | `main([configure, body])`, a dataclass instance | `runFilter([configure, body])`, a module object | `apply` after `apply`, the options moved from the first to the second | `run_filter([configure, body])`, a `const` mutable struct |
| an option, only `false` turning it off (`== false`) | 3 lines each | `lookup(user, k) == MetaBool(False)` | `equal(lookup(user, k), MetaBool(false))` | `matches!(lookup(meta, &[.., k]), Some(MetaValue::MetaBool(false)))` | `lookup(user, k) == MetaBool(false)` |
| `header-includes`: what is there, as blocks after the new one | 12 | `MetaBlocks(header, *map(as_block, items(x) if x else []))`, `as_block` one line (`to_python`) | the same, `items(given(x))`, `toPlain`, and a cast (gap 32, closed in part: the cast stays) | a loop over `items(given(x))`, `as_block` a 6-line `match` (gap 33, left) | `as_block` four one-line methods |
| `\indent` at the start (`utils.equals`) | `utils.equals(a, b)` | `==` | `equal` | `==` | `==` |
| Lua's `gsub` replacement | built in (the bug) | 8 (`re.sub` with a function) | 6 | 16 | 7 |

- **#16's API, in use**: `items` is exactly the original's "a list's
  items, or the value alone", after Lua's `if meta['header-includes']`
  (so `items(x) if x else []`: `items` takes `false` as a value, as Lua's
  `ensure_list` would). `lookup` reads each option in one call; Lua's
  truth decides whether there is a configuration (`if user:` in Python).
  The plain values carry the conversion to blocks in Python and
  TypeScript: `to_python(v)` and `toPlain(v)` give what Lua would put in
  the list, and the `MetaBlocks` constructor converts it as Lua does,
  failing where Lua fails (a map, which Lua takes as an empty list, is the
  one difference: one line). TypeScript's `toPlain` returned `unknown`,
  so the array needed `as Blocks` (gap 32); since panir#17 it returns a
  `PlainValue`, which for any metadata value is the union of what Lua
  converts and what it fails on: the cast stays, saying "as Lua would". Rust and Julia have no plain
  values: a small function by kind. Python's `meta["indent"] = True`,
  Julia's `meta["indent"] = true` and Rust's `true.into()` convert;
  TypeScript's `meta.indent = MetaBool(true)` (the metadata's fields
  don't convert, as its README says).
- **A list of filters** reads as the original in Python, TypeScript and
  Julia; the options are module state, as Lua's upvalues. Rust's two
  `apply`s pass the first filter's options to the second
  (`Body(configure.0)`), as the README shows.
- **Python**: a `@dataclass` for the options, `main([configure, body])`;
  `next(k for k in ... if first == command(k))` for the explicit command.
- **TypeScript**: `(["indent", "noindent"] as const).find(...)` keeps
  the kind's type.
- **Rust**: `block.tag()` against the option's names; `Configure::meta`
  gives the `Result` of its work to the walk, `ctx.ok(self.configure(..))`;
  the raw nodes are `Block::raw` and `Inline::raw`; what is in
  `header-includes` is `items(given(meta.get(..)))`: all panir#17's
  (before, a `match` into `ctx.fail`, struct literals, and
  `.filter(|v| truthy(Some(v)))`: gaps 31, 30, 27). TypeScript reads it
  the same, `items(given(meta["header-includes"]))`.
- **Julia**: `Base.@kwdef mutable struct Options`; `replace(text, r"%.?"s
  => f)` gives `f` the matched text, not a match (the first try indexed
  it).

Nothing here needed a copy: the original's one `\noindent` inline and
marker divs, which Lua copies on each use, are made anew each time
(`command(kind)`).
