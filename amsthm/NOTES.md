# amsthm

ickc/pandoc-amsthm's `amsthm.lua`, the largest filter of round 1 (1,323
lines, 1,033 of code): amsthm's theorem and proof environments, declared
in the metadata by style (`plain`, `definition`, `remark`, and styles of
one's own), with counters shared (`Theorem: [Lemma]`) or within LaTeX's
sectioning units (`parent_counter`), written as divs of their class. In
LaTeX and beamer, amsthm's own: `\newtheorem`s and `\newtheoremstyle`s in
`header-includes`, `\begin{...}...\end{...}` around the div's blocks,
citations of identifiers `\ref` and `\eqref`. In any other format the
filter numbers as amsthm would (sections counted as LaTeX counts them,
which needs the writer's options), sets the heading (`Theorem 1 (note).`)
and the body in the style's fonts, holding nested environments out of
them, ends a proof with a symbol (or puts it at `\qedhere`), and makes
citations of identifiers links to the numbers; HTML gets a little CSS.
Two top-down filters, sharing their state as Lua's upvalues. The owner's
own filter: its behaviour questions are answered from its README, its
design notes, and its own tests, which are the cases here.

## Round 3: a note needs pandoc's reader

A div's `info`, the note in its heading, is read by the original with
`pandoc.read(info, "markdown")`, a call into pandoc (round 3), in three
places: the heading of a theorem, the heading of a proof (where the note
is the whole heading, in italics), and LaTeX's `\begin{Theorem}[note]`
(braced where it holds math, a citation, an image or raw TeX).

**Where a port has pandoc's reader, it reads the note as the original
does**, and braces it as the original does: Python with libpandoc
(`libpandoc.read(info, "markdown")`) where this process has it (pandocpy,
`libpandoc.convert`, or libpandoc installed), Julia with LibPandoc.jl where
its environment has it (pandocjl's), Rust built for wasm32-wasip1
(`libpandoc::read`, the host's: pandocrs, pandocpy, pandocjl,
libpandoc.wasm). A note that is not inlines (a list, two paragraphs) fails,
as the original fails putting a block among inlines. TypeScript has none:
no JavaScript host gives a filter the pandoc it runs in, and the `pandoc`
command would cost a process a note (`HOSTS.md`).

**Without a reader** (`run.py test`; a Rust JSON filter; TypeScript
everywhere), a note of **plain text** is what pandoc's reader would make
of it without reading: words of ASCII letters and digits or of any
character that is not ASCII nor a space, a hyphen inside a word, one of
`,;:?!` after one, single spaces between: a `Str` a word, a `Space`
between, exactly. `plain-notes.py` checks the rule against pandoc 3.14
(3,000 random notes it takes, each read by pandoc as a paragraph: all
the same). Any other note (an apostrophe, which pandoc makes curly;
emphasis; a citation; math; `e.g.`, which pandoc joins to the next word
with a no-break space; two spaces) is **kept as its text** (its words, as
Lua's `pandoc.Inlines` makes them of a string), with a warning naming it:
closer to what a user wants than a failure, as amsthm itself warns and
goes on wherever it can, and the heading is still there, its markup
showing. The brace in LaTeX is then never needed (text has no math).

Which of the original's own cases this changes, known differences there
(`known.tsv`: `python@standalone`, `rust@standalone`,
`rust@pandocrs-native`, `julia@standalone`, and `ts` everywhere), each
with a twin whose notes are plain and which is the same in every
language:

- `model-source` (and `-n`, `-chapter`, the golden test's two runs):
  `**This** is *markdown*.`, `can cite \ref{simplestEquation}`,
  `pandoc-crossref style cite @simplestEquation`, `Proof of the *little*
  theorem`; twin `model-source-plain`;
- `natbib`: all five notes (citations, `$[0,1]$`, `[1]`, an image); twin
  `natbib-plain`;
- `proof-note`: `Proof of @Euler, see [@euler]`; twin `proof-note-plain`;
- `example-proofs` (the documentation's): `Another proof of @Sqrt2`;
  twin `example-proofs-plain`;
- and this repository's `notes-markup`; `notes-plain` passes.

In every host with a reader (`run.py host`: pandocpy and libpandoc.convert
for Python, pandocjl for Julia, the four wasm hosts for Rust) all of them
are the original's, but `proof-note` in HTML and markdown, whose body is
the empty list item below.

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
All the same (the 36 Rust rows `known.tsv` had for them are gone); the
`cargo test` in the Rust port runs the cases with the conversion's
`options` in process.

## The original's behaviour, kept

- **Metadata, as Lua reads it**: `amsthm` not a map (empty, a list)
  defines only the proof (`config-empty`, `config-list`); a style's
  entries one value or a list, an entry a name or a map of a name to the
  names sharing its counter, its keys in order (`map-order`), a value a
  list of names or one value's text (a map's text, `Prop: {a: b}` sharing
  with `b`: `theorems`); `name_to_text` a map (a list: no names), text,
  null or a boolean failing, as Lua's `pairs` does (`error-name-to-text-*`);
  `parent_counter` a map, or any value's text (`[chapter]` is `chapter`,
  `true` unsupported: `config-lists`, `config-text`); `counter_depth`
  Lua's `tonumber` of its text (`"1.5"`, `"0"`, `abc`, `false`:
  `counter-depth*`); `css` off for the text `false` only (`css: no`
  keeps it); `swapnumbers` on for the text `true`; `qed_symbol` the
  text of its math, raw TeX and words, joined by spaces (`qed-symbol-*`),
  a map none, `true` or blocks failing as Lua's `pandoc.Inlines`.
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
  (`duplicate-class`, `bodies`).
- **Bodies**: the heading goes into a first paragraph, plain text or
  heading, else a paragraph of its own; a proof's symbol onto a last one,
  else a paragraph; the body font is each `Str` wrapped, emphasis in
  italics undone, bold in bold flattened, neighbouring elements joined,
  and the citations and `\ref`s the font applies to wrapped, with nested
  environments held out (`bodies`, `italic-ref`, `styles-more`).
- **`\qedhere`**: in math, each `\qedhere` not followed by a letter is
  `\quad` and the symbol; a raw `tex` or `latex` inline of it (blanks
  around) is the symbol's span, after the blanks before it are removed;
  nested environments keep theirs (`qedhere`, `qedhere-more`).
- **References**: `[@a]` `(1)`, `@a` `1`, `@A` `Theorem 1` (the
  capitalised id of an environment's), `[@a; @b]` each in parentheses;
  `\ref{a}` and `\eqref{a}` raw TeX (format `tex` only); a reference to an
  unnumbered environment left, with one warning per id; in LaTeX `\ref`
  and `\eqref`, `Theorem~\ref{euler}` (`refs`, `named-ref-text`,
  `multi-ref`, `unnumbered-ref`).

### pandoc's Lua, not the filter: an empty list item

`tests/cases/named-ref.md` writes its theorems' bodies as `A.` to `D.`,
which pandoc's markdown reads as ordered lists, each of one empty item
(`fancy_lists`). The original tests the first block of each environment
(`div.content[1].content`); reading a list's `content` in pandoc 3.14's
Lua, then returning the element, makes each empty item `[Plain []]`
(checked with a filter of one line, `local x = d.content[1].content;
return d`; without the read, or returning without it, the item stays
`[]`). The ports, reading the list but changing nothing in it, keep `[]`:
a known difference in HTML and markdown (`named-ref`, `proof-note-plain`,
whose body is `A.` too); `named-ref-text`, the same with text, is the
same everywhere. Reproducing pandoc's conversion of an empty table would
be porting pandoc's internals, not the filter.

## For the owner: what looks unintended

1. **`[see @a]` and `[@a, p. 3]` lose their prefix and locator**: a
   reference to one environment resolves to `(1)` whatever else the
   citation holds (case `refs`), where `[see @a; @b, p. 2]` is left to
   citeproc with a warning. The single case could do the same, or keep the
   affixes around the link.
2. **A map's key keeps its backslash**: `Foo\*: [Bar]` (as the README
   writes a list's `Conjecture\*`) is the environment `Foo\`, unnumbered,
   and `\newtheorem{Bar}[Foo\*]{Bar}` in LaTeX, which breaks; YAML keys
   are not markdown. `name_to_text`'s keys likewise: `Name\*` names
   nothing, `Name*` would (case `theorems`). The README could say `Foo*`
   as a key, or the filter strip a backslash before the star.
3. **A heading as an environment's first block takes the heading in**:
   `# A heading` first in a theorem is, in HTML, the `<h1>` "Theorem 1. A
   heading" (case `bodies`), where LaTeX puts `\begin{Theorem}` in a line
   before the heading. A heading probably wants the paragraph of its own.
4. **`amsthm-held` is not reserved**: the body's font holds nested
   environments out as divs with that attribute, and puts back whatever
   div has it: a user's div with `amsthm-held="1"` becomes a copy of the
   first nested environment (case `bodies`).
5. **`name_to_text: {proof: ""}`** names the proof `proof.` in HTML (an
   empty text falls back to the environment's name) and `\proofname{}` in
   LaTeX (case `theorems`).
6. **Metadata errors are Lua's**: `name_to_text` as text, null or a
   boolean fails with `bad argument #1 to 'for iterator' (table expected,
   got string)`; `qed_symbol: true` with pandoc's `Inline, list of Inlines,
   or string expected`; amsthm's own warnings would name the key.
7. **The test cases `named-ref` and `proof-note` have lists for bodies**
   (above): their assertions are about the references, so they pass, but
   the bodies are not the text they look like.

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| two passes, shared state | `return {pass1, pass2}`, an upvalue | `main([first, second])`, a module `State` | `runFilter([first, second])`, module `let`s | `apply_with` after `apply_with`, the options moved into the second | `run_filter([first_pass => :topdown, second_pass => :topdown])`, `const` `Ref`s |
| a theorem, the proof | a class, a subclass with its own `to_header` | a dataclass, a subclass | a class with a `proof` flag | a struct with a `proof` flag | a struct with a `proof` flag |
| a div's theorem, two classes compared | `theorem ~= found` | `is not` | `!==` | an index into the list | `!==` |
| a sub-walk from a function | `pandoc.Div(div.content):walk{...}` | `div.content.walk(f)` | `applyFilter(div.content, f)` | `ctx.apply(&mut div.content, &mut f)?` | `walk!(f, div.content)` |
| a font's filter, for each font | a table built by a function | a `Filter` built by a function | an object built by a function | `struct Wrap(Font)` | `struct Wrap{T}`, a method per kind |
| the held environments put back | `held[tonumber(i)]`, a copy each (Lua) | `copy.deepcopy` | `copy` | `.clone()` | `deepcopy` |
| Lua's `tonumber` | built in | a regular expression | a regular expression | by hand (`f64`'s parse takes `inf`) | a regular expression |
| the writer's options | `PANDOC_WRITER_OPTIONS` | `ctx.conversion.options` or `.writer_options`, a dict | `ctx.conversion.options` or `.writerOptions`, an object | `conversion.options` or `.writer_options`, a JSON value | `ctx.conversion.options` or `.writer_options`, a `Dict` |
| a note | `pandoc.read(info, "markdown")` | `libpandoc.read(info, "markdown")`, if imported | none | `libpandoc::read(info, "markdown")`, built for wasm | `LibPandoc.read(info, "markdown")`, if in the environment |

- **Python**: dataclasses for the style, the theorem, the counter and the
  options; `match` for metadata shapes. The held environments are put
  back as copies, as Lua returns a copy each time: without it, a user's
  div with `amsthm-held` got the same environment object, which was then
  numbered twice in one place (the case `bodies` found it; `GAPS.md` 35
  again).
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
- **Julia**: a font's filter is a parametric callable struct, `Wrap{T}`,
  with a method per kind of node (`(::Wrap{T})(x::T)`); the passes are
  two functions with methods, the state `const` `Ref`s.

Copies: the held environments, as above; nothing else is in two places
(`CHECK_UNIQUE=1` on every case, every language).
