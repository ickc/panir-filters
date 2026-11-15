# auto-lang

ickc/font's `bin/auto-lang.lua`: the metadata maps Unicode scripts to
languages (`auto-lang: {Hebrew: he, Greek: el, Han: zh-Hant}`), and each
run of text in a mapped script becomes a span with `lang` (and `dir=rtl`
for a right-to-left script), across inline boundaries. Two filters in
turn, as the original returns them: the map (`Meta`, in a filter of its
own, as a filter's `Meta` runs after its `Inlines`), then a top-down walk
whose `Inlines` function regroups each list, and whose `Span` function
skips a span with a `lang` and everything in it. The owner's own filter:
its behaviour questions are answered from its documentation (the file's
header, `src/multilingual.md`) and its output.

## The table of scripts

The original `dofile`s `script-ranges.lua` beside it, generated upstream
(`scripts/generate_script_ranges.py`) from the Unicode 15.0.0 Character
Database: every script's own ranges (`Scripts.txt`), sorted and flat; the
ranges of Script=Inherited; each script's Script_Extensions
(`ScriptExtensions.txt`, aliases resolved with `PropertyValueAliases.txt`);
the right-to-left scripts (`DerivedBidiClass.txt`, R or AL); and the
scripts' names.

The ports' tables are made from that one file: `upstream/script-ranges.lua`
is the original's, verbatim, and `tables.py` (here, 121 lines) reads it and
writes `python/script_ranges.py`, `ts/script-ranges.ts`,
`rust/src/script_ranges.rs` and `julia/script-ranges.jl` (750 ranges of
161 scripts, 29 inherited, the extensions of 68), each a constant of the
language's own (tuples, arrays, `&[(u32, u32, &str)]` behind
`#[rustfmt::skip]`, `UInt32` tuples), searched by bisection;
`tables.py --check` says whether they are as it writes them. No file is
read at run time, as the survey asked. The data is Unicode's, under the
Unicode License v3 (`upstream/UNICODE-LICENSE`, as fetched from
unicode.org, whose terms of use put every Unicode data file under it):
permissive, asking that the notice go with copies, which it does.

No language's own facility gives the original's answer, so none is used:

- **Python**: `unicodedata` has no Script property (it is Unicode 15.0.0
  too, but has categories and names only).
- **TypeScript**: `\p{Script=...}` and `\p{Script_Extensions=...}` exist,
  but are Node.js's ICU, Unicode 18 under Node.js 26: 𮯰 (U+2EBF0, CJK
  Extension I, Unicode 15.1) is Han there and unassigned, so neutral, in
  the original (case `cjk`), and the answers would change with Node.js.
- **Rust**: no Script property in `std`; `unicode-script` has it at its
  own Unicode version, a dependency that would differ the same way.
- **Julia**: none (`Base.Unicode` is utf8proc, Unicode 16, with categories
  only).

## The original's behaviour, kept

- **Classes of a code point**, in this order: its own script if mapped;
  else tied to the mapped scripts whose Script_Extensions hold it, even if
  its own script is another (the Arabic-Indic digits continue a Thaana run:
  case `thaana`); else its own script, unmapped, which ends a run; else
  Script=Inherited (a combining mark, ZWJ, ZWNJ, a variation selector),
  which goes with what precedes it; else neutral (spaces, ASCII digits and
  punctuation). Neutral text joins a run only if the run goes on after it;
  tied text joins a run it touches, before or after (`「*神說*」`, `*神說*。`
  across inlines: cases `basic`, `inlines`).
- **A run with the whole list to itself** takes the neutral text around
  it (a paragraph of Greek ending in a full stop is one span, the stop in
  it); otherwise the neutral text stays out (`καὶ, εἶπεν;` in an English
  sentence: case `neutral`).
- **Containers** (emphasis, strong, links, quotes, small caps,
  strike-out, sub- and superscripts, underlines, spans without `lang`)
  belong to a run when all their text is the run's script (or tied to it);
  one holding another script, code, math, a note, a citation or a span
  with `lang` ends the run and is walked on its own (case `inlines`).
- **The metadata is walked too** (a title, case `basic`), as the second
  filter's `Inlines` applies to every list of inlines.
- **The map**: a map of scripts; a key that names no script (`Klingon`,
  `greek`) is warned about and left out (case `map-values`); a value is
  its text (`*el*-GR` is `el-GR`). `auto-lang: false` maps nothing; a list
  or text (`auto-lang: Greek`, inlines) maps nothing, warning about its
  indices, as Lua's `pairs` gives them (`map-list`, `map-inlines`); text
  from the command line (`-M auto-lang=Greek`) and `true` fail
  (`error-map-string`, `error-map-true`).
- **Idempotent**: a span with a `lang` is skipped whole, even `lang=""`
  (Lua's truth of an empty string: case `inlines`); the repository's
  `src/multilingual.md` (case `multilingual`) comes out as it went in, and
  stripped of its spans by the repository's `strip-lang.lua`
  (`multilingual-plain`) it gets them back, but for the hand-written
  override, which is the map's language then, as `strip-lang.lua` says.

## For the owner: what looks unintended

1. **A text is cut at its pieces even where nothing joins a run**: once a
   list gets a span, each `Str` in it is rebuilt from `split`'s pieces, so
   another script's text and the neutral text after it become two `Str`s
   (`Greek;` is `Greek`, `;`), and a combining mark after a character of
   an unmapped script becomes a `Str` of its own, apart from its base
   (`café`, decomposed, is `cafe`, `́`: case `marks-unmapped`). Writers
   mostly print them the same, but the AST differs from what was read, and
   a mark alone is not a character. In `split`, a mark after a foreign
   character could go onto that piece (`pieces[#pieces]`) as it goes onto
   a run.
2. **A line break ends a run at the top of a list, but not in a
   container**: `καὶ\`, newline, `εἶπεν` is two spans; the same inside
   emphasis is one (case `inlines`), as `list_script` takes `LineBreak` as
   neutral and `group` closes on it.
3. **A mark after a space stays with the space**: `: ́αβ` keeps `́` out of
   the Greek run (case `marks`); a decomposed accent at the start of a
   word, after a space, is so rare that this may be right.
4. **The map's errors are Lua's**: text or `true` fail with `bad argument
   #1 to 'for iterator' (table expected, got string)`; a list or inlines
   warn `no Unicode script named 1`. A message naming `auto-lang` would
   help.
5. **Unicode 15.0.0**: characters added since (CJK Extension I, 15.1) are
   neutral: `神𮯰說` is one Han span, but `𮯰` alone gets none (case
   `cjk`). Raising `VERSION` in the generator is all it takes.

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| the table | `dofile` of the generated file | `from script_ranges import ...` | `import ... from "./script-ranges.ts"` | `mod script_ranges;` | `include("script-ranges.jl")` |
| a code point's class | strings and a table of ties, memoised in a table | a `str`, a `frozenset`, `Mark.INHERITED` or `None`, `functools.cache` | a tagged union, a `Map` | an `enum Kind`, the ties an `Rc<BTreeSet>`, a `HashMap` | a `String`, a `Set`, `:inherited` or `nothing`, `get!` on a `Dict` |
| the walk | `traverse = "topdown"`, `Span` returning `el, false` | `Filter(traverse="topdown")`, `ctx.skip_children()` | `Filter<"topdown">`, `ctx.skipChildren()` | `Order::Topdown`, `ctx.skip_children()` | `tag => :topdown`, `skip_children!(ctx)` |
| `Inlines` declining (Lua's `nil`) | `return changed and out or nil` | `return None` | `return undefined` | decide first (`has_run`), as the list is changed in place | `nothing` |
| two filters | `return {{Meta}, {...}}` | `main([configure, tag])` | `runFilter([configure, tag])` | `apply` after `apply`, the map given on | `run_filter([configure, tag => :topdown])` |

- **Python, TypeScript, Julia**: the original's three functions (`split`,
  `list_script`, `group`) port as they are, with their pending lists and
  closures; `match` on a code point's class reads well in Python, a
  tagged union in TypeScript, dispatch on the value's type in Julia.
- **Rust**: `inlines` changes its list in place, so to decline as Lua's
  `return nil` does (the list as it was, its `Str`s not rebuilt) the port
  decides first whether a run is there (`has_run`, a second look at
  the list) before taking it apart: `GAPS.md` 36. The grouping's
  state is a struct (`Grouping`) with `close` and `add`, as closures
  could not share it mutably; the ties an `Rc`, as the memo hands out the
  same set for every tied character.
- **Running a script that loads a file beside it**: the tables are a file
  beside each port. Run as a program, every language finds it; run in
  process by a host, Python's `runpy.run_path` does not put the script's
  folder on `sys.path`, and Julia's `Module()` has no `include`: this
  repository's hosts (`host/python.py`, `julia-cases.jl`) had to be fixed
  for it (`GAPS.md` 37).

## Checked

21 cases (one format, `html`: the filter does not look at it; 2 fail in
the original), every port the same as the original, `CHECK_UNIQUE=1` too.
Differential runs, every port the same: the real documents in all
scripts' maps (`assemble.py --lang`: every script mapped, and every one
but Latin): the Wikipedia article (Cyrillic, Arabic, Han, Devanagari,
Myanmar, Bengali, Hebrew, Armenian spans), War and Peace, the Chinese
page of code (724 Han spans), the pandoc manual (3,260 Latin spans when
Latin is mapped); and the documents made of the cases.
