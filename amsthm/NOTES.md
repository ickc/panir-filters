# amsthm

ickc/pandoc-amsthm's `amsthm.lua`, the largest filter of round 1 (1,412
lines at the proposed v4.0.0): amsthm's theorem and proof environments,
declared in the metadata by style (`plain`, `definition`, `remark`, and
styles of one's own), with counters shared (`Theorem: [Lemma]`) or within
LaTeX's sectioning units (`parent_counter`), written as divs of their
class. In LaTeX and beamer, amsthm's own: `\newtheorem`s and
`\newtheoremstyle`s in `header-includes`, `\begin{...}...\end{...}` around
the div's blocks, citations of identifiers `\ref` and `\eqref`. In any
other format the filter numbers as amsthm would (sections counted as LaTeX
counts them, which needs the writer's options), sets the heading
(`Theorem 1 (note).`) and the body in the style's fonts, holding nested
environments out of them, ends a proof with a symbol (or puts it at
`\qedhere`), and makes citations of identifiers links to the numbers; HTML
gets a little CSS. Two top-down filters, sharing their state as Lua's
upvalues. The owner's own filter: its behaviour questions are answered
from its README, its design notes, and its own tests, which are the cases
here.

**The version mirrored** is the proposed v4.0.0 (`upstream/ORIGIN`:
`8606511`, the tip of the stack of pull requests ickc/pandoc-amsthm#39 to
#48, not merged yet): eight fixes, then the note moved from the `info`
attribute to a span. Until 2026-11-18 the ports mirrored v3.1.1
(`ba1008f`), whose note was the attribute read as markdown by the filter
(`pandoc.read`), round 3's call into pandoc; the ports read it with the
host's pandoc where they had one and kept a note of plain text as its
words where they had none, and `known.tsv` had 60 rows for it and for
two test cases whose bodies were lists. All of that is gone (the git
history has it): the filter, and every port, is pure, calling no reader,
reading no file and no variable.

## The note

- **A span with class `info` that is the first inline of the div's first
  block, a `Para` or `Plain`** (`[Lagrange]{.info} The order of...`), as
  pandoc has read it with the document: its content is the note, the span
  goes with one `Space` or `SoftBreak` after it, and the paragraph too if
  nothing is left (a note in a paragraph of its own before a list or a
  display: the heading then takes a paragraph of its own). The span's
  identifier, other classes and attributes are dropped (`.smallcaps` shows
  because pandoc's reader makes a `SmallCaps` inside the span). A span with
  class `info` anywhere else, or first in a heading, is an ordinary span.
  An empty span is no note (cases `note-span`, `note-edges`).
- **Else the `info` attribute, as plain text** (`pandoc.Inlines(info)`:
  its words, single `Space`s, a `SoftBreak` for a newline), none when it
  is empty or blank. Markup in it is shown as typed, with one warning a
  document (the first such note, and the porting guide), where "markup" is
  any character but ASCII letters and digits, Lua's spaces, anything not
  ASCII, and `,.;:!?()/-` (`note-attribute`, `note-text`).
- **Both**: the span, with a warning naming the attribute; the markup
  test is then not made (`note-both`).
- Where the note goes is v3's: in parentheses in a theorem's heading, in
  the note font; a proof's whole heading, in italics but its references;
  LaTeX's `\begin{Theorem}[note]`, braced where it holds math, a
  citation, an image or raw TeX. The note is taken before the body is set
  in its font, and in LaTeX in the second pass (each pass takes it where
  the original does, so the one warning names the same note).

## The writer's options

The original reads `PANDOC_WRITER_OPTIONS`: `number_sections` (-N),
`top_level_division`, `number_offset`, and the variable `secnumdepth`;
without -N it numbers theorems within sections as LaTeX does then, from
0. pandoc gives a Lua filter these, and a JSON filter none (checked:
pandoc 3.14 passes `PANDOC_READER_OPTIONS` and `PANDOC_VERSION` only).
libpandoc (1.10 and later) gives them to every filter it runs, as
`$PANDOC_WRITER_OPTIONS` for a JSON filter and a script and the
conversion's `writer_options` for one in process: Lua's values under
pandoc's JSON keys (`number-sections`, `top-level-division` as
`top-level-chapter`, `number-offset`, `variables.secnumdepth`). The ports
read the conversion's `options` (defaults-file keys, where a host running
them in process knows them) and else its `writer_options`, the prefix
`top-level-` dropped as the original drops it: so the same in every host
and as a JSON filter run by libpandoc, Rust's included.

The original's tests run with them, so `run.py` has a case's options
(`NAME.opts`, `GAPS.md` 38): the original runs with them (`pandoc -d`);
`run.py test` runs every port with `$PANDOC_WRITER_OPTIONS` as libpandoc
sets it for those options, and the Python, TypeScript and Julia ports in
process with them in `ctx.conversion.options` too; `run.py host` runs
them through the hosts themselves, with the options as a defaults file.
All the same; the `cargo test` in the Rust port runs the cases with the
conversion's `options` in process.

## The original's behaviour, kept

- **Metadata, as Lua reads it**: `amsthm` not a map (empty, a list)
  defines only the proof (`config-empty`, `config-list`); a style's
  entries one value or a list, an entry a name or a map of a name to the
  names sharing its counter, its keys in order (`map-order`), a value a
  list of names or one value's text (a map's text, `Prop: {a: b}` sharing
  with `b`: `theorems`); a key of that map, of `name_to_text` and of
  `parent_counter` drops a backslash before ASCII punctuation, as markdown
  does in a value (`Claim\*` as a key is `Claim*`: `escaped-keys`,
  `escaped-more`, `theorems`); `name_to_text` a map, anything else
  ignored, with a warning unless it is empty text (`name_to_text:` or
  null: `name-to-text-*`, `meta-shapes`, `config-lists`, whose list now
  warns);
  `parent_counter` a map, or any value's text (`[chapter]` is `chapter`,
  `true` unsupported: `config-lists`, `config-text`); `counter_depth`
  Lua's `tonumber` of its text (`"1.5"`, `"0"`, `abc`, `false`:
  `counter-depth*`); `css` off for the text `false` only (`css: no`
  keeps it); `swapnumbers` on for the text `true`; `qed_symbol`, when it
  is inlines or text, the text of its math, raw TeX and words, joined by
  spaces (`qed-symbol*`), anything else (a map, a list, a boolean,
  blocks) ignored with a warning, keeping `\Box`.
- **Styles**: a user's style named as an option or `proof` is warned
  about and left out, as is one not a map; one named `plain` replaces the
  built-in one (and is a `\newtheoremstyle{plain}` in LaTeX); an unknown
  font is warned about; `headpunct: ""` is an empty `Str`
  (`styles-more`, the repository's `styles`, `styles-bad`).
- **Environments**: a trailing `*` unnumbered; a name with spaces a class
  with underscores (`A B` and `A_B` are both `A_B`: the later wins); a
  user's environment named `proof` is a `\newtheorem{proof}` in LaTeX but
  the proof everywhere else, as the proof is added last (`theorems`); a
  div of two environments' classes is left alone with a warning
  (`duplicate-class`, `bodies`); a name listed under an unnumbered one
  (`Aside*: [Remark]`) is numbered on its own, within its own
  `parent_counter` if it has one, with a warning
  (`unnumbered-shared`, `unnumbered-more`); an empty name for the proof
  is none: "Proof", and no `\proofname` in LaTeX (`proof-name-empty`,
  `theorems`).
- **Bodies**: the heading goes into a first paragraph or plain text, else
  a paragraph of its own (a heading keeps its own line); a proof's symbol
  onto a last one, likewise (`heading-edge`, `heading-more`, `bodies`);
  the body font is each `Str` wrapped, emphasis in italics undone, bold in
  bold flattened, neighbouring elements joined, and the citations and
  `\ref`s the font applies to wrapped, with nested environments held out
  (`bodies`, `italic-ref`, `styles-more`), as placeholders whose attribute
  starts with a mark no document knows, so a user's div with
  `amsthm-held` is an ordinary div (`held-attribute`, `held-more`).
- **`\qedhere`**: in math, each `\qedhere` not followed by a letter is
  `\quad` and the symbol; a raw `tex` or `latex` inline of it (blanks
  around) is the symbol's span, after the blanks before it are removed;
  nested environments keep theirs (`qedhere`, `qedhere-more`).
- **References**: `[@a]` `(1)`, `@a` `1`, `@A` `Theorem 1` (the
  capitalised id of an environment's), `[@a; @b]` each in parentheses;
  `\ref{a}` and `\eqref{a}` raw TeX (format `tex` only); a reference to an
  unnumbered environment left, with one warning per id; one with a prefix
  or suffix (`[see @a]`, `[@a, p. 3]`, `@a [p. 3]`) left to citeproc with
  a warning each, as several are; in LaTeX `\ref` and `\eqref`,
  `Theorem~\ref{euler}` (`refs`, `named-ref`, `multi-ref`,
  `unnumbered-ref`, `affixed-ref`, `affixed-more`).

Every warning is the original's, word for word and in its order: checked
for each case in LaTeX and markdown, every language against the original
(not by `run.py`, which compares documents only).

## For the owner

### Filed and fixed upstream (proposed)

What this file listed as looking unintended in v3.1.1 was filed as
ickc/pandoc-amsthm#31 to #38 and is fixed by the stack #39 to #46 (then
the porting filter, #47, and the note span, #48); the ports here follow
each fix:

| issue | v3.1.1 | fix | cases here |
|---|---|---|---|
| #31 | the test cases `named-ref` and `proof-note` had lists for bodies (`A.`: an ordered list of one empty item, which pandoc's Lua turned into `[Plain []]` once the filter read the list's `content`) | #39: words for bodies | `named-ref`, `proof-note` (the twins `named-ref-text`, `proof-note-plain` and their known rows gone) |
| #32 | `[see @a]` and `[@a, p. 3]` lost their prefix and locator | #40: left to citeproc with a warning, as several are | `affixed-ref`, `affixed-more`, `refs` |
| #33 | a heading first in an environment took the heading in (`<h1>Theorem 1. A heading`), the symbol the last | #41: a heading keeps its own line | `heading-edge`, `heading-more`, `bodies` |
| #34 | a user's div with `amsthm-held` became a copy of the first nested environment | #42: the placeholders' mark is the address of their list | `held-attribute`, `held-more`, `bodies` |
| #35 | a map's key kept its backslash: `Foo\*` was the environment `Foo\`, `\newtheorem*{Foo\}` | #43: a backslash before punctuation dropped | `escaped-keys`, `escaped-more`, `theorems` |
| #36 | `Foo*: [Bar]` shared a counter that does not exist (`\newtheorem{Bar}[Foo*]`) | #44: numbered on its own, with a warning | `unnumbered-shared`, `unnumbered-more`, `theorems` |
| #37 | `name_to_text: {proof: ""}` named the proof `proof.` in HTML and `\proofname{}` in LaTeX | #45: an empty name is none | `proof-name-empty`, `theorems` |
| #38 | `name_to_text` or `qed_symbol` of the wrong shape stopped with Lua's error | #46: ignored with a warning naming the key | `meta-shapes`, `name-to-text-*`, `qed-symbol-blocks`, `-true`, `-list` |

### Found in the v4 original while porting it

Small, none worth a port's difference (each port reproduces it, and the
cases pin it). Each with a document that shows it, run as
`pandoc -L amsthm.lua -t markdown doc.md` (pandoc 3.14), front matter
`amsthm: {plain: [Theorem]}`:

1. **A blank span is a note; a blank attribute is not.** `[ ]{.info}
   Body.` (pandoc reads `Span [Space]`) gives `Theorem 1 ( ).`; the
   attribute's test is `info:match("%S")`, the span's `#note > 0`. The
   same test for both (no note when the span's content stringifies to
   blanks) would match (case `note-edges`, Theorem 5).
2. **An empty span with the attribute warns "using the span" and gives no
   note.** `::: {.Theorem info="the attribute"}` then `[]{.info} Body.`:
   warning `a note in both a leading [...]{.info} span and info="the
   attribute"; using the span`, heading `Theorem 1.`. Either the
   attribute could be used when the span is empty, or the warning could
   say the note is empty (`note-both`, Theorem 2).
3. **The attribute's spaces at its ends are kept, without a warning.**
   `info="trailing "` gives `(trailing )`, where v3, reading markdown,
   gave `(trailing)`; the porting guide says a note of plain text renders
   as before. pandoc's attribute syntax refuses a leading space, so only
   a trailing one (or HTML or JSON input) reaches it. Trimming before
   `pandoc.Inlines` would match v3 (`note-text`, Theorem 4).
4. **`name_to_text: ""` warns, `name_to_text:` does not.** The quoted
   empty string is `MetaInlines []`, not `""`, so it is "not a map"; the
   pull request names only the empty and null forms. Harmless.
5. **When both forms are on one div, the attribute stays on it** in
   other output (`data-info="the attribute"` in HTML) while the heading
   shows the span's note: v3 left the attribute too, but then it was the
   note (`note-both`, Theorem 1).

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| two passes, shared state | `return {pass1, pass2}`, an upvalue | `main([first, second])`, a module `State` | `runFilter([first, second])`, module `let`s | `apply_with` after `apply_with`, the options moved into the second | `run_filter([first_pass => :topdown, second_pass => :topdown])`, `const` `Ref`s |
| a theorem, the proof | a class, a subclass with its own `to_header` | a dataclass, a subclass | a class with a `proof` flag | a struct with a `proof` flag | a struct with a `proof` flag |
| a div's theorem, two classes compared | `theorem ~= found` | `is not` | `!==` | an index into the list | `!==` |
| a sub-walk from a function | `pandoc.Div(div.content):walk{...}` | `div.content.walk(f)` | `applyFilter(div.content, f)` | `ctx.apply(&mut div.content, &mut f)?` | `walk!(f, div.content)` |
| a font's filter, for each font | a table built by a function | a `Filter` built by a function | an object built by a function | `struct Wrap(Font)` | `struct Wrap{T}`, a method per kind |
| the held environments' mark | `tostring(held)`, the table's address | `id(held)` | `crypto.randomUUID()` (no address) | `RandomState::new().hash_one(())` (a local's address is the same in every run of a wasm build) | `objectid(held)` |
| the held environments put back | `held[i]`, each once | the node itself | the node itself | moved out of an `Option` | the node itself |
| Lua's `tonumber` | built in | panir's `lua_number` | `luaNumber` | `lua_number` | `lua_number` |
| the writer's options | `PANDOC_WRITER_OPTIONS` | `ctx.conversion.options` or `.writer_options`, a dict | `ctx.conversion.options` or `.writerOptions`, an object | `conversion.options` or `.writer_options`, a JSON value | `ctx.conversion.options` or `.writer_options`, a `Dict` |
| the note | `take_note`: the span out of the first paragraph, or `pandoc.Inlines(info)` | `take_note`, `inlines(info)` | `takeNote`, `inlines(info)` | `take_note`, `panir::inlines(info)` | `take_note!`, `inlines(info)` |
| the markup test | a Lua pattern, `[^%w%s\128-\255,.;:!?()/%-]` | a regular expression | a regular expression (`u`) | a `chars().all` | `all` over the characters |
| a key's backslash | `gsub("\\(%p)", "%1")` | `re.sub` | `replace` with `/g` | by hand (`next_if`) | `replace` with a regex |
| the metadata's shape | `pandoc.utils.type` | `match` on the classes | `.t` | `match` on `MetaValue` | `isa` |

- **Python**: dataclasses for the style, the theorem, the counter and the
  options; `match` for metadata shapes (`MetaString("")` as a pattern for
  the empty `name_to_text:`).
- **TypeScript**: `Extract<Inline, { t: ... }>` for the font elements
  and raw inlines; `given` for Lua's `x and f(x)` (`documentclass`,
  `counter_depth`); a class's parameter properties are not erasable
  syntax, so fields by hand.
- **Rust**: the longest port (rustfmt's layout of the long `match`es);
  `Block::raw`, `Inline::raw`, `ctx.ok` for the options' `Result`,
  `given`, `items`, `into_items` for `header-includes`, all from
  panir#17, in use; theorems by index, as Lua's identity comparison
  needs a stable name; the three walks within a function (hold, fonts,
  restore; `\qedhere`) are structs with `Filter`, each `ctx.apply`'d. A
  `\qedhere` in math by hand (the `regex` crate has no look-ahead).
  Taking the note out of the first paragraph is a `let` chain matching
  the span, then a second match to move it out (`remove(0)`), as the
  borrow of the first match can't be moved from.
- **Julia**: a font's filter is a parametric callable struct, `Wrap{T}`,
  with a method per kind of node (`(::Wrap{T})(x::T)`); the passes are
  two functions with methods, the state `const` `Ref`s.

Copies: none. With the placeholders' mark each held environment is put
back once, so the ports put back the node itself (v3's copies, for a
user's div with the attribute, are gone); nothing is in two places
(`CHECK_UNIQUE=1` on every case, every language).
