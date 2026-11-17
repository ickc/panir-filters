# Gaps in the hosts

Round 1 tested [panir](https://github.com/ickc/panir), the AST and the
filter API (`GAPS.md`). Rounds 2 and 3 test the **hosts**: the libpandoc
bindings that run filters inside a pandoc conversion, where a filter can
ask the pandoc it runs in to read, convert or run citeproc with no second
process, and where a directory sandbox can confine a conversion. Each
port of every filter ran inside its language's hosts on the cases of
`run.py test`, compared with the same expected output (`run.py host`); the
filters that read files ran again inside each host's directory sandbox
(`run.py host --sandbox`), and probes measured what each kind of filter
can reach there and what calls into pandoc cost. What a host lacks or
makes awkward is below, in `GAPS.md`'s form: what the original does, what
the host does, which hosts, a proposed change, and how many filters would
hit it, ranked; then every difference between hosts in the same
situation, the table of what each kind of filter reaches under the
sandbox, and what each host offers a filter that calls pandoc.

Measured 2026-11-16 on Linux (x86-64, 32 cores), libpandoc 1.12 (the
`continuous` release, pandoc 3.14, downloaded and checked against its
`SHA256SUMS` and build attestation by libpandoc-python's
`scripts/fetch-libpandoc.sh`), each host built from its `main` as its
README says:

| host | commit | built |
|---|---|---|
| libpandoc-python (`pandocpy`, `libpandoc.convert`) | `edd5eac` | `pip install .[wasm]` against the release (wasmtime-py 50), Python 3.13 |
| libpandoc-rs (`pandocrs`, the `libpandoc` crate) | `cdd1f3a` | `cargo build --release -p pandocrs` (the `wasm` feature), Rust 1.97.1; the ports' guest side at the same commit |
| LibPandoc.jl (`pandocjl`, `LibPandoc.convert`) | `f31e113` (`main`, which has no directory sandbox); the sandbox's runs used its unmerged branch `sandbox` at `35ad4ce` | `Pkg.instantiate()`, Julia 1.12.7, Panir.jl at panir's `main`; a system image by `sysimage/build.jl` |
| libpandoc's JavaScript hosts (`wasm/node.mjs`, `wasm/wasm-filter.mjs`) | `bad3c1d` | `npm ci` in `wasm/`, Node 26.13, the release's `libpandoc.wasm` |

## Since: libpandoc 1.13 and panir 0.2 (2026-11-17)

libpandoc 1.13 (ickc/libpandoc#8: filters told whether pandoc's sandbox
is on; cancel tokens; the JavaScript hosts give a filter the instance it
runs in, `fn(doc, ctx, pandoc)`, with `pandoc.read`, and bytes for binary
output; a wasm filter in Node sees the working directory), panir 0.2.0
(ickc/panir#19: `Conversion.sandbox`, `Conversion.from_context`,
`lua_number`, a sub-walk inherits the conversion, TypeScript's
`libpandocFilter`), and the three bindings adopting both (their branches
`adopt-1.13`). The ports now use panir's `lua_number` (not each its own
copy) and a sub-walk's inherited conversion; the TypeScript ports of
include-files, multibib and amsthm read and run citeproc with the
instance of libpandoc's JavaScript host they run in (each port's
`hosted(pandoc)`, given to `libpandocFilter`; as a JSON filter they run
the `pandoc` command, or for amsthm read only plain notes, as before),
and the Node driver runs every TypeScript port through `libpandocFilter`
(no glue of its own) with the case's options. Each gap below:

| # | now |
|---|---|
| H1 | **closed**: libpandoc 1.13 tells every filter whether pandoc's sandbox is on (a callback's context, `PANDOC_SANDBOX`); each binding confines a read through a conversion when it is told so, or its options ask for a sandbox, or nothing can tell (no `sandbox`, no options), and its directory sandbox tells every filter it runs outside the module `true`; a test in each binding, for every kind of filter (the bindings' `adopt-1.13`) |
| H2 | **closed**: libpandoc 1.13's JavaScript host gives a filter its instance, panir 0.2's `libpandocFilter` makes the conversion; the TypeScript ports call pandoc there (amsthm's notes read as the original reads them) |
| H3 | open: a wasm filter's calls still run in pandoc's sandbox (multibib's bibliography files; `known.tsv`) |
| H4 | **closed** (the bindings' host-gaps pull requests: a read with a conversion keeps tabs; libpandoc 1.13's JS host's `readOptions` too) |
| H5 | documented (each README; the ports reset their state) |
| H6 | open (proposed: `LIBPANDOC_WASM_DIRS`) |
| H7 | **closed** by libpandoc 1.13: in Node a wasm filter sees the working directory, read-only (`node-dir.mjs`), as in the other hosts |
| H8 | documented |
| H9 | **closed** (libpandoc-rs#15: a callback's conversion has the options, `read_options(conversion, format)`, a directory sandbox mapped to `true`) |
| H10 | documented (libpandoc-rs's README) |
| H11 | documented (each README's sandbox section); include-code-files' `include=""` inside the sandbox stays known |
| H12 | **closed** by libpandoc 1.13: the JS host returns bytes for binary writers |
| H13 | **closed** (LibPandoc.jl#6: `pandocjl` has the directory sandbox) |
| H14 | open (performance: the JSON exchange) |
| H15 | open (`read_many` documented in each README); measured with pantable's batch |
| H16 | open (found with pantable) |
| H17 | open (found with pantable) |
| H18 | open (found with pantable) |

## pantable (2026-11-17)

The owner's pantable (`pantable/NOTES.md`), the last filter of rounds 2
and 3: CSV with YAML options, an included file (round 2), markdown cells
read by the host's pandoc, all of a table's texts in one `read_many`
(round 3: the first port that batches). Its hosts were built in scratch
from clean exports of the bindings' `adopt-1.13` branches, against the
released libpandoc 1.13 (`continuous`, pandoc 3.14, checked against its
`SHA256SUMS` and build attestation by `fetch-libpandoc.sh`):

| host | commit | built |
|---|---|---|
| libpandoc-python | `6a51dba` (`adopt-1.13`; its tip `9248791` adds tests and CI only) | a venv, Python 3.13, panir 0.2.0 (panir's `main`, `62c78c2`), wasmtime-py 50 |
| libpandoc-rs | `0b82607` (`adopt-1.13`, the tree its `main` merged as `b709b02`) | `cargo build --release -p pandocrs`, Rust 1.97.1 |
| LibPandoc.jl | `f3faaf3` (`adopt-1.13`; its tip `9b1d07f` adds a Windows-only change to Ctrl-C and tests) | `Pkg.instantiate()`, Julia 1.12.7, no system image |
| libpandoc's JavaScript host | `f2bd848` (`main` then; now `7000f75`, a fix to cancelling during a callback) | `npm ci` in `wasm/`, Node 26.13, the release's `libpandoc.wasm` |

`run.py host pantable` (`CHECK_UNIQUE=1`): 696 case runs, 548 the
original's, 148 known, none failing (`pantable/NOTES.md` has each mode's);
`--sandbox`: 474, 387 the original's, 87 known, none failing. The known
ones are the ports' standalone ones (CSV and YAML libraries, the owner's
decision on `csv-kwargs`), the original's joined document (three cases
in every mode), a wasm filter's `outside` (H6), YAML 1.2's `no`, and H17.
It found H16 to H18 below; and measured H15 with a batch (below, "Calls
into pandoc").

## How the ports ran

`run.py host` (its docstring has the variables naming each host). A mode
is a language in a host:

| mode | what runs | filters | case runs | the same | known (`known.tsv`) |
|---|---|---|---|---|---|
| `pandocpy` | a Python port as a filter script, in process (`pandocpy -F port.py`) | 24 | 881 | 871; since 1.13: 872 | 10; 9 |
| `python-api` | a Python port's `panir.Filter`s as functions, `libpandoc.convert(filters=[...])` | 24 | 881 | 871; 872 | 10; 9 |
| `pandocrs-wasm` | a Rust port built for wasm32-wasip1 (`pandocrs -F port.wasm`) | 24 | 881 | 860; 861 | 21; 20 |
| `pandocrs-native` | a Rust port as a JSON filter, a program libpandoc runs (`pandocrs -F port`) | 22 (not the two that need pandoc) | 854 | 819; 819 | 35 (32 of them amsthm's notes: no reader); 35 |
| `pandocpy-wasm`, `pandocjl-wasm`, `node-wasm` | the same wasm file in pandocpy, pandocjl, and libpandoc.wasm's Node host (`wasm-filter.mjs`) | 24 each | 881 each | 860, 860, 838; 861, 861, 861 | 21, 21, 43; 20, 20, 20 |
| `pandocjl` | a Julia port as a filter script, in process (`pandocjl -F port.jl`), a process a case | 24 | 881 | 869; 870 | 12; 11 |
| `node` | a TypeScript port's filter as a JS function inside libpandoc.wasm in Node (`convertWithFilters`; since 1.13 panir's `libpandocFilter`) | 24 | 881 | 845; 872 | 36 (amsthm's notes: no reader, H2); 9 |

7,902 runs, 7,693 the original's, 209 known, none failing. Since 1.13
("Since", above; the same runs on 2026-11-17, the hosts on their
`adopt-1.13` branches against a local build of libpandoc 1.13): 7,749
the original's, 153 known, none failing: H4's tabs (the reader-options
case, in eight modes), H7 (every include of include-files and
include-code-files in Node) and H2 (amsthm's notes and include-files'
reader options in the TypeScript port) no longer differ. The known
ones are round 1's (the originals' bugs a port doesn't copy, as
standalone, and amsthm's empty list item of pandoc's Lua), amsthm's notes where a port has no reader,
and the hosts' own: every difference from the standalone result is one
of the gaps below; none was worked around. In the directory sandbox
(`--sandbox`: include-files and include-code-files in nine modes, the
Lua originals among them), 297 runs the original's and 27 known: the
wasm filters' and the reader's tabs, as outside it, and the three of H11;
a JSON filter is refused there, status 83 (checked with pandocrs). Since
1.13: 303 the original's, 21 known (the tabs no longer differ), none
failing; what each read from outside unchanged (the filter scripts, the
user's code, read the canaries themselves, by design; nothing else did).

## The gaps, ranked

By rank (the last column), then as found; H14 and H15 came last, from the
timings, and H14 ranks with H3 to H5; H16 to H18 with pantable, H17 with
H3 to H5.

| # | gap | hosts | filters hitting it (of 24) | rank |
|---|---|---|---|---|
| H1 | a filter's `read(text, ctx.conversion)` reads files outside a command's directory sandbox | Python (`pandocpy` scripts), Julia (`pandocjl` scripts, sandbox branch), Rust (every callback) | any that parses document text in a format whose reader reads files (include-files with `format=latex`, `org`, `rst`; pantable's cells): no case here, the probes found it | high: security, and the READMEs say otherwise |
| H2 | a JavaScript filter can't reach the pandoc it runs in | JavaScript (Node, browsers) | 3: include-files, multibib, amsthm (its notes) | high |
| H3 | a wasm filter's calls to pandoc can't read the files the filter can | the four wasm hosts | 1: multibib (and section-refs, section-bibliographies, multiple-bibliographies: SURVEY's round 3) | medium |
| H4 | libpandoc's `read` expands tabs; `pandoc.read` doesn't | every libpandoc host | 1: include-files (any port of a `pandoc.read` on text with tabs) | medium |
| H5 | a filter's module state outlives its document in some hosts and not others | Julia (scripts, loaded once per process); Python and JavaScript functions reused | 9 Julia ports keep state in globals; theorem numbers on (`Theorem 2`) | medium |
| H6 | a wasm filter on a command line sees only the working directory, read-only; nothing names more | pandocrs, pandocpy, pandocjl | 2: include-files, include-code-files (a file through `..`) | medium |
| H7 | a wasm filter in Node sees no directory | Node (`wasm-filter.mjs`) | 2: include-files, include-code-files | medium |
| H8 | a wasm filter is not given the process's environment | the four wasm hosts (and Lua inside the directory sandbox) | 1: pagebreak | low |
| H9 | Rust's callbacks get no `options`; Rust's `read_options` refuses a directory sandbox and takes no format | Rust | 3 (every reader of fragments: include-files, multibib, amsthm) | low |
| H10 | a wasm filter that may call pandoc runs only where pandoc is | every host without libpandoc (`wasmtime run`) | 2: amsthm (optional reader), any filter calling pandoc sometimes | low |
| H11 | in WASI an empty name is the working directory | libpandoc.wasm (sandbox, Node), wasm filters | 1: include-code-files (`include=""`) | low |
| H12 | `convertWithFilters` throws on binary output unless told `bytes` | JavaScript | every docx, odt or epub conversion (pagebreak's, track-changes' cases) | low |
| H13 | `pandocjl`'s `main` ignores `LIBPANDOC_SANDBOX` | Julia (`main`) | every sandboxed command | low (the branch closes it) |
| H14 | a filter in a host is handed the whole document as JSON, both ways: a filter that does little costs four times a Lua one | every host, every non-Lua filter | every filter; 0.35 s a filter on a 9 MB document | medium (performance) |
| H15 | a call into pandoc from a filter costs six times `pandoc.read`'s | every host | 3 that read fragments (amsthm: hundreds of calls) | low (`read_many` exists) |
| H16 | a Julia filter script's packages come from pandocjl's load path only (its project, the default environment), not from the script's own project; the README doesn't say | Julia (`pandocjl`, scripts) | 1: pantable (YAML.jl, DelimitedFiles, StringEncodings, OrderedCollections; `run.py` stacks this repository's project with `JULIA_LOAD_PATH`) | medium |
| H17 | `pandocjl` cuts a Lua filter's writes to standard output and error at 65,536 bytes when the stream is a pipe; to a file they are whole | Julia (`pandocjl`) | every filter whose output passes `run.py`'s `capture.lua` above 64 KiB: pantable's three big cases, in `pandocjl` and `pandocjl-wasm` | medium (silent loss of output) |
| H18 | `read_many`'s error names a text by its place in the batch, counted from 0 in Python and Rust and from 1 in Julia (JavaScript's is in the text's place, unnumbered): a filter that batches maps it back to its cell itself | every host | 1: pantable (a table's texts in one call) | low |

### H1. `read(text, ctx.conversion)` reads files outside a command's directory sandbox

**The original**: a Lua filter's `pandoc.read` runs inside the module with
the rest of the conversion when the conversion is sandboxed: confined.

**The hosts**: a filter in the host's process (Python, Julia, a Rust
callback) is the user's code and unconfined by design, documented. What
it reads through pandoc is another matter: it passes the *document's*
text to pandoc's reader, and that text may name files (`\input{...}` in
LaTeX, RST's `include`, Org's `#+INCLUDE`). libpandoc-python's README:
"A filter that reads fragments with `read(text, ctx.conversion)` reads
them in pandoc's own sandbox: no files"; LibPandoc.jl's sandbox branch
says the same. That holds for a filter function given to
`libpandoc.convert(..., sandbox={...})` and `LibPandoc.convert(...;
sandbox = ...)`, whose conversion carries the options, which
`read_options` turns into pandoc's own sandbox. It does not hold for:

- a **filter script under `pandocpy`** with `LIBPANDOC_SANDBOX` set: the
  script's conversion has no options (it is made from the environment, as
  for a JSON filter), so `read(text, ctx.conversion)` and
  `read_many(..., ctx.conversion)` read natively with no sandbox: a probe
  read a LaTeX fragment `\input`ting a file inside the listed directory
  and one outside it, and got both;
- a **filter script under `pandocjl`** (the sandbox branch): the same,
  both read;
- a **Rust callback** (`libpandoc::convert_with` with `"sandbox"`):
  libpandoc-rs gives a callback's conversion no options at all, so
  `read_options(conversion)` has no sandbox: both read. (Were the options
  there, Rust's `read_many` would refuse a directory sandbox outright,
  where Python and Julia map it to `true`: H9.)

A wasm filter's calls are always `"untrusted"` (pandoc's sandbox): read
nothing, in every host. The JavaScript host's instance, called from a JS
function, reads what the instance was given (`preopens`): the inside
file, not the outside one; with `sandbox: true` neither.

**Proposed**: (a) libpandoc tells a callback that its conversion is
sandboxed (a `"sandbox": true` in the context, or the options), and each
host's `read_options` maps it to pandoc's own sandbox, for scripts and
callbacks alike; until then pandocpy and pandocjl put `{"sandbox":
true}` in the conversion they make for a script when they run the
command sandboxed, and libpandoc-rs gives a callback the options and maps
a directory sandbox to `true` as the others do; (b) the READMEs say what
holds now: a script's `read` under the command's sandbox is not confined.
A test in each host: the probe (`\input` of a canary inside and outside).

**How common**: every filter that parses document text in a format whose
reader reads files. Of this round's, include-files (`format=latex`, `org`,
`rst` on an included file) and pantable's cells (round 3's next); amsthm
reads markdown, whose reader reads nothing.

### H2. A JavaScript filter can't reach the pandoc it runs in

**The original**: `pandoc.read`, `pandoc.utils.citeproc`: pandoc's
functions, there for any Lua filter.

**The host**: libpandoc's JavaScript host runs a JS function as a filter
(`convertWithFilters`), with the document and libpandoc's context as JSON
text. A function can call the instance again (nested calls work), but
only if the code that loaded the instance hands it over: a filter written
as a script (`runFilter` from `panir/node`, the TypeScript ports) has no
way to reach it, and nothing in the context leads to it. Nor is there
panir glue: Python's host builds a `panir.Conversion` from the context
and has `read(text, conversion)`; in JavaScript every user writes the
parse, the `Conversion`, the `applyFilter` and the serialising
(`host/libpandoc-node.mjs` does). So the TypeScript ports run the
`pandoc` command, a process a call, as pandocfilters and panflute filters
do: include-files a process per included file (without the document's
reader options), multibib two or more per document, and amsthm none (a
process per note would cost more than it is worth: its notes stay text).
In a browser there is no command at all.

**Proposed**: (a) the JS host gives a filter function the instance (or
bound `read`, `readMany`, `convert`) as a third argument, and runs panir
filters itself: `convertWithFilters(options, input, [panirFilter])`
recognising a panir filter object, or a small `panir/libpandoc` module
with `runFilter`'s twin for a host, making the `Conversion` from the
context (`Conversion.fromContext` in panir's TypeScript, as Python has);
(b) a `read(text, conversion)` there, as the other languages have, so a
port reads the same way in every host.

**How common**: 3 of round 2's and 3's 3 filters that call pandoc
(include-files, multibib, amsthm's notes); every round 3 filter.

### H3. A wasm filter's calls to pandoc can't read the files the filter can

**The original**: `pandoc.utils.citeproc(doc)` reads the bibliography
files the metadata names, relative to the working directory.

**The hosts**: a wasm filter sees the working directory (read-only), and
its calls to pandoc are marked `"untrusted"`: pandoc's own sandbox, no
files. citeproc then can't read `test/primary.bib`, which the filter
itself could read: multibib fails (`PandocResourceNotFound`) in pandocrs,
pandocpy, pandocjl and Node, wherever the metadata names a file. Inline
`references` work.

**Proposed**: let a wasm filter's calls see what the filter sees: run
them with the filter's own directories (natively, pandoc's sandbox with
those directories readable, as the directory sandbox's mounts are; in the
sandbox, the same mounts), so that untrusted means "no more than the
filter", which is what the READMEs promise ("the filter gets no more
access through pandoc than it has itself"). Otherwise, document that a
wasm filter's citeproc takes only inline references (a filter could read
the `.bib` itself and convert it with a call from biblatex to CSL JSON:
a workaround each filter would write).

**How common**: 1 here (multibib); SURVEY's round 3 citeproc filters all
(section-refs, section-bibliographies, multiple-bibliographies); any call
whose input names a file (a LaTeX fragment's `\input`, by design refused).

### H4. libpandoc's `read` expands tabs; `pandoc.read` doesn't

**The original**: `pandoc.read(text, format, reader_options)` runs the
reader alone: a tab inside a line stays a tab.

**The hosts**: libpandoc's `read` and `read_many` (every host's) read as
the `pandoc` command reads its input, whose first step replaces tabs by
spaces unless `preserve-tabs`. An included file with `indented\tcode`
gives `indented  code` (include-files' `reader-options`, in Python,
Rust's four wasm hosts and Julia; the TypeScript port's `pandoc` command
likewise). With `"preserve-tabs": true` libpandoc's read gives what
`pandoc.read` gives (checked: tabs, CRLF and a byte-order mark).

**Proposed**: libpandoc's fragment readers (`read_many`, the hosts'
`read(text, conversion)`) read as `pandoc.read` does, the text as given,
or the hosts' `read_options` set `preserve-tabs` unless the conversion
says otherwise; at least a line in each "Coming from Lua": `pandoc.read`
keeps tabs, `read` expands them.

**How common**: 1 (include-files, on files with tabs); any port of a
filter that reads code or tables with `pandoc.read`.

### H5. A filter's module state outlives its document in some hosts

**The original**: pandoc loads a Lua filter anew for each conversion, and
a JSON filter is a new process: a counter starts at 0 for each document.

**The hosts**: `pandocjl` and `LibPandoc.convert` load a Julia script
once per process and keep its module ("A script's globals persist
between the documents of one process", the README says): theorem's port,
run twice in one Julia process, numbers the second document's theorem
`Theorem 2`. pandocpy and `libpandoc.convert(filters=["x.py"])` run a
Python script anew each time (`Theorem 1` twice), but a Python or JS
filter *function* passed in keeps its module's state too. A port written
like its Lua original, with state in module globals, is right in one
host and wrong in the next: nine Julia ports keep state so (auto-lang,
comments, count-para, first-line-indent, pandoc-quotes, theorem,
track-changes, and this round's include-files and multibib, which now
reset theirs at each document).

**Proposed**: each language's "Coming from Lua" (and each host's README)
says that a filter's globals live as long as the host's process, and to
reset them in a first `Pandoc` or `Meta` function; the ports do (round 1's
nine). For Julia, perhaps the host runs a script's top level again for
each document in its cached module (state reset, methods kept compiled),
at least for scripts that ask.

**How common**: 9 of 25 Julia ports; the same holds of the Python and
TypeScript ports used as functions.

### H6. A wasm filter on a command line sees only the working directory

**The original**: `io.open("../chapters/a.md")` reads any file.

**The hosts**: a wasm filter given on a command line (`-F x.wasm` in
pandocrs, pandocpy, pandocjl) sees the working directory, read-only, and
nothing else: an include through `..`, or through a symbolic link leaving
it, is a missing file (include-files' and include-code-files' `outside`),
with or without the directory sandbox. The APIs can give it more
(`WasmFilter(dirs=...)` in Python, `dirs =` in Julia,
`WasmFilter::preopen` in Rust); the commands have no flag or variable.

**Proposed**: a list as `LIBPANDOC_SANDBOX`'s, `LIBPANDOC_WASM_DIRS`, for
the commands, read-only, the working directory by default; and, under
the directory sandbox, the sandbox's own list.

**How common**: 2 (include-files, include-code-files), for a book whose
chapters include a common directory beside them.

### H7. A wasm filter in Node sees no directory

**The hosts**: `wasm-filter.mjs` gives a filter only the directories its
caller builds in memory (browser_wasi_shim's `PreopenDirectory`), none by
default, in Node as in a browser; pandocrs, pandocpy and pandocjl give
the working directory. Every include of include-files and
include-code-files is a missing file in Node (`known.tsv`).

**Proposed**: in Node, the working directory, read-only, by default, read
through `node:fs` (a shim directory backed by the disk, or `node:wasi`
for the filter), as the other hosts; in a browser, documented as none.

**How common**: 2 (the filters that read files).

### H8. A wasm filter is not given the process's environment

**The original**: pagebreak reads `PANDOC_PAGEBREAK_HTML_CLASS` and
`PANDOC_PAGEBREAK_ODT_STYLE` (`os.getenv`).

**The hosts**: a wasm filter gets only the variables pandoc gives filters
(documented), so pagebreak's port ignores both in the four wasm hosts
(cases `env`, `env-meta`). Inside the directory sandbox the Lua original
gets only `SOURCE_DATE_EPOCH`, so it ignores them too.

**Proposed**: an allow-list (`WasmFilter(env=...)`, and
`LIBPANDOC_WASM_ENV=NAME,...` for the commands); for the sandbox, the
same list. Low.

**How common**: 1 of 24 (pagebreak).

### H9. Rust's callbacks: no options; `read_options` takes no format and refuses a sandbox

**The hosts**: a Python or Julia filter function given to `convert` gets
the options in its conversion (`ctx.conversion.options`); a Rust callback
(`Filter::func`, `Filter::panir`) gets none, so a port reading the
writer's options or the sandbox through them finds nothing there (the
writer's options are in `writer_options`: amsthm reads those). Python's
`read_options(conversion, format)` and Julia's `read(text, conversion;
from = format)` read a fragment in another format the conversion's way;
Rust's `read_options(conversion)` has no format, so a port sets
`options["from"]` by hand (include-files). And Rust's `read_many`
refuses a directory sandbox in the options, where Python and Julia map
it to pandoc's own (H1).

**Proposed**: the options in a callback's conversion, as the other
languages'; `read_options(conversion, format)`; a directory sandbox
mapped to `true`. Low.

**How common**: 3 (the readers of fragments).

### H10. A wasm filter that may call pandoc runs only where pandoc is

**The hosts**: a filter built with libpandoc-rs's guest side imports the
`libpandoc` module as soon as it may call pandoc, so it can't be
instantiated without it (`wasmtime run`, or `filters/wasm-filter.sh`'s
fallback without pandocrs), even for a document with nothing to read:
amsthm's wasm build, which reads a note only when there is one, needs a
host for every document. Documented ("a filter that calls pandoc imports
what it has not").

**Proposed**: nothing in the hosts; a line in libpandoc-rs's README for
a filter whose calls are optional (two builds, or a feature). Low.

**How common**: 1 (amsthm).

### H11. In WASI an empty name is the working directory

**The hosts**: wasi-libc resolves `""` to the working directory: Lua's
`io.open("")` succeeds inside libpandoc.wasm and `fh:lines()` then fails
on a directory, so include-code-files' `include=""` fails the conversion
inside the directory sandbox (pandocpy, pandocrs, pandocjl), where natively
it is a missing file and an empty block; a wasm filter likewise (its ports
test for `""`).

**Proposed**: document it beside the sandbox's other differences (`..`
across mounts, "permission denied"); or refuse `""` in the hosts' WASI
path resolution. Low.

**How common**: 1 (include-code-files, on an odd input).

### H12. `convertWithFilters` throws on binary output unless told `bytes`

**The hosts**: libpandoc-python and LibPandoc.jl return bytes for docx,
odt and epub by themselves; the JS host decodes the output as UTF-8 unless
`{ bytes: true }`, and a docx fails with "The encoded data was not valid
for encoding utf-8" after the conversion succeeded (every docx, odt and
epub case of pagebreak and track-changes, until the driver passed
`bytes`).

**Proposed**: bytes for the binary writers (the host can ask libpandoc
which are text, as Python's `_text_writers` does), or an error before
converting. Low.

**How common**: every binary output.

### H13. `pandocjl`'s `main` ignores `LIBPANDOC_SANDBOX`

**The host**: LibPandoc.jl's `main` has no directory sandbox yet (its
`sandbox` branch does): with `LIBPANDOC_SANDBOX` set, `pandocjl` converts
with no sandbox and says nothing, where pandocpy and pandocrs confine the
conversion. Until the branch is merged a user setting the variable gets
no protection and no warning. Low (transient).

### H14. A filter in a host is handed the whole document as JSON

**The original**: a Lua filter in pandoc gets elements marshalled as it
asks for them: list-table's, which looks at divs only, adds 0.09 s to a
conversion of the 9.4 MB textbook, in any host.

**The hosts**: every other filter gets the whole document as pandoc's
JSON and gives it back so: a Rust callback that does nothing worth
measuring (caps, list-table) adds 0.35 s, a wasm filter 0.4, a JSON
filter (a program libpandoc starts) 0.37, a Python one 0.6: pandoc's own
encoding and decoding of the JSON, in Haskell, is most of it (the same
Rust port standalone, reading and writing the JSON itself, takes 0.05 s).
So in a native host a filter in process costs what a JSON filter costs;
the process was the cheap part. Where the filter does real work (auto-lang,
amsthm) the ports still beat Lua (`RESULTS.md`, "Inside a warm host").

**Proposed**: measure where libpandoc's time goes in a callback (aeson's
encoding and decoding, the copy across the C ABI) and, if it is aeson, a
faster encoder or a binary exchange (the AST as panir lays it out, or
CBOR); consecutive in-process filters of one host could share one
exchange (LibPandoc.jl already parses once for consecutive Julia
filters). The promise of the hosts, against a JSON filter, is the call
into pandoc and the start of a runtime (Julia's compiling: 3.4 s against
0.7 s warm), not the exchange.

**How common**: every filter; on the textbook, 0.35 s each.

### H15. A call into pandoc costs six times `pandoc.read`'s

**The hosts**: `read(text, conversion)` sets up a conversion for each
call (options, the reader, JSON out, panir in): 0.3 ms a note, against
`pandoc.read`'s 0.05 ms in Lua; `read_many` reads the 738 notes in 13 to
16 ms, cheaper than Lua's loop. A filter ported as its original was
written (a call per note, per file) pays the difference: +0.25 s for
amsthm's 738 notes in Python, +0.06 s in Lua.

**Proposed**: a cheaper single read (the reader set up once per
conversion and kept for the filter's calls), or each README's section on
calls from filters saying to batch with `read_many`, with this number.
Low.

**How common**: 4 (the filters that read fragments); amsthm's notes,
pantable's cells (thousands of texts) most. pantable's ports batch (a
table's texts in one `read_many`): on a table of 10,011 texts, Python
reads them in 0.20 s batched against 3.50 s a call each (0.35 ms a
call), Julia 0.18 against 0.58, Rust as wasm 0.17 to 0.22 against 0.58
to 0.92; libpandoc.wasm in Node 2.73 against 3.03 (one thread: the
reading is the cost). The original's one `pandoc` process reads its
joined document in 0.51 s.

### H16. A Julia filter script's packages: pandocjl's load path only

Found with pantable, the first Julia port with dependencies of its own
(YAML.jl, DelimitedFiles, StringEncodings, OrderedCollections; this
repository's `Project.toml`).

**The original**: a Python filter imports what its Python has; under
plain pandoc a Julia filter script runs as `julia --project=DIR
script.jl`, a wrapper's choice.

**The hosts**: `pandocjl` loads a script into a module of its own, in its
own process, so the script's `using YAML` resolves against pandocjl's
load path: LibPandoc.jl's environment (or the app's), the default one
(`@v#.#`), the standard library. The script's own project (a
`Project.toml` beside it or above) isn't consulted: `Package
OrderedCollections not found in current path`, then pandocjl's advice to
run it as a subprocess. The README says scripts run as a subprocess get
"its load path", and nothing of in-process scripts. `run.py` sets
`JULIA_LOAD_PATH` to pandocjl's project, then this repository's, then
the standard library, which works (checked: the case runs).

**Proposed**: while loading a script, stack the project found from the
script's directory (`Base.current_project(dirname(script))`) after
pandocjl's own, so its dependencies resolve as `julia --project` would
find them, pandocjl's packages winning; or at least a README line: a
script's packages come from pandocjl's load path, extend it with
`JULIA_LOAD_PATH` or add them to the default environment. Medium: any
Julia filter that uses a package.

**How common**: 1 of 25 (pantable); any Julia filter with a dependency.

### H17. `pandocjl` cuts a Lua filter's output to a pipe at 64 KiB

Found with pantable's big cases, whose documents `run.py`'s
`capture.lua` writes to standard output.

**The hosts**: a Lua filter (in `pandocjl`, any) that writes 100,000
bytes with `io.stdout:write` and 100,000 with `io.stderr:write`:

    -- big.lua
    function Pandoc(doc)
      io.stdout:write(string.rep("x", 100000))
      io.stderr:write(string.rep("y", 100000))
      return doc
    end

    echo hi | pandocjl -t plain -o /dev/null -L big.lua | wc -c   # 65536
    echo hi | pandocjl -t plain -o /dev/null -L big.lua > out      # 100000

pandocrs, pandocpy and pandoc write 100,000 to the pipe. Standard error
to a pipe is cut the same way (65,536 bytes of `y`), and with standard
output to a file and standard error to a pipe, the file holds 65,536
`x` and the pipe 65,536 `x` as well as the `y`s: the bytes go astray.
pandoc's own output (`-t json` to standard output, 1.3 MB) is whole: only
what the Lua filter writes; and in `pandocjl`'s directory sandbox
(`run.py host --sandbox`) the same three cases come whole. Likely Julia's
libuv putting a pipe into non-blocking mode, so libpandoc's writes stop at
the pipe's buffer (unverified). Through a file, the Julia port and the
Rust port as wasm in `pandocjl` give the original's result on all three
cases (`known.tsv`, `@host` rows: `run.py host`, not `--sandbox`).

**Proposed**: in LibPandoc.jl, make the standard streams blocking while
pandoc runs (or hand Lua's `io` writes to Julia's streams), with a test
that writes 100 KB through a pipe. Medium: silent loss of output, which
a filter's author would not suspect.

**How common**: any Lua filter writing more than 64 KiB to a pipe under
`pandocjl` (here 6 case runs: 3 cases, 2 modes).

### H18. `read_many`'s error names a text by its place in the batch

Found with pantable (`md-yaml-cell`: a cell holding a YAML block pandoc
can't read).

**The original**: one `pandoc` process a table; when it fails panflute
raises `OSError('')`, naming nothing.

**The hosts**: `PandocParseError: reading input 2: Error parsing YAML
metadata...` in Python and Rust (from 0), `reading input 3` in Julia (from
1), for the same text; JavaScript's `readMany` puts `{error: {kind,
message}}` in the text's place and the port throws `kind: message`. The
number is the text's place in what the filter sent (pantable sends only
the non-empty texts: the caption, the short caption, the cells, the
attributes), so it means nothing to the document's author unless the
filter maps it back; `return_exceptions` (Python, Julia) and Rust's
`ReadMany::docs` let it.

**Proposed**: each README's `read_many` section: the number counts from
0 (or 1 in Julia), and a filter reporting a failed text should map it
back (`return_exceptions=True`); perhaps one numbering everywhere. Low.

**How common**: 1 of 25 (the only filter that batches).

## Differences between hosts in the same situation

- **A script's read under the command's sandbox** (H1): confined for a
  filter function given to `convert(sandbox=...)` (Python, Julia); not for
  a script under `pandocpy` or `pandocjl`, nor for a Rust callback. (Now
  confined in every host: "Since", above.)
- **A Python script and a Julia script, run twice in one process** (H5):
  the Python script starts afresh, the Julia one goes on.
- **What a wasm filter sees** (H6, H7): the working directory in
  pandocrs, pandocpy and pandocjl; nothing in Node.
- **What a callback's conversion holds** (H9): the options in Python and
  Julia, none in Rust.
- **A directory sandbox in a fragment's read** (H9): Python and Julia map
  it to pandoc's own sandbox, Rust refuses it.
- **A fragment's read in another format**: `read_options(conversion,
  format)` in Python, `read(text, conversion; from = format)` in Julia,
  none in Rust (H9), nothing in JavaScript (H2).
- **Binary output** (H12): bytes in Python and Julia, an exception in
  JavaScript without `bytes`.
- **A failed wasm filter's message**: every host passes the filter's
  standard error on (`panir filter: ... in filter function block`), then
  pandoc's "exited with status 1"; pandocpy says it once more, pandocjl
  adds a Julia stack trace of its own code, Node prefixes each line with
  the filter's path.
- **Warnings of a call from a filter**: Python issues them as
  `PandocWarning` (a filter silences them with `warnings`), Julia logs
  them (`with_logger`), Rust returns them in the output's log, JavaScript
  returns them in the result's log; Lua's `pandoc.log.silence` has a twin
  in none (multibib silences each its own way).

- **A failed text in `read_many`** (H18): Python and Rust number it from
  0, Julia from 1 (`reading input N`); JavaScript puts the error in its
  place.
- **A Lua filter's output to a pipe** (H17): `pandocjl` cuts it at 64 KiB;
  the others pass it whole.

Same everywhere, checked: the writer's options a filter is given
(`PANDOC_WRITER_OPTIONS` for scripts and wasm filters, `writer_options`
in process, Node's context included: amsthm's 27 documents with options,
every host); the reader's options; a JSON filter refused inside the
directory sandbox (pandocrs, "libpandoc.wasm runs no programs"); and every
round 1 port, in every host, gives what it gives standalone, but
pagebreak's built for wasm (H8). (`run.py host` runs a document a process,
or a Python script anew, so H5 doesn't show there; H12 is passed by its
Node driver.)

## What each kind of filter reaches under the directory sandbox

The cases of include-files and include-code-files in each host's
directory sandbox (`run.py host --sandbox`: only the cases' directory
listed, read-only, and a temporary one for the document, the Lua filters
and the output): every case inside the directory gives the original's
result in every host (but H11's `include=""` for the Lua original); the
three canaries outside it, reached through `..`, through a symbolic link
in the directory, and by an absolute name, were read or not as below.
Then probes from a filter: a file outside opened directly, and a LaTeX
fragment `\input`ting a file inside the directory and one outside, read
through pandoc.

| filter | pandocpy | pandocrs | pandocjl (`sandbox` branch) | JavaScript (Node) |
|---|---|---|---|---|
| Lua (the original), inside the module | confined: none of the three read | confined | confined | (no sandbox setting: Lua inside libpandoc.wasm sees the instance's `preopens`) |
| wasm filter (the Rust port) | sees the mounts: none read | the same | the same | sees no directory (H7) |
| Python or Julia filter **script** (`-F port.py`, `port.jl`) | unconfined, by design and documented: all three read | – | unconfined, documented: all three read | – |
| Python or Julia filter **function** (`convert(sandbox=...)`) | unconfined, documented: all three read | – | the same | a JS function: unconfined (no setting to confine it) |
| Rust callback (`convert_with` with a sandbox) | – | unconfined, documented: the file read | – | – |
| JSON filter (a program) | refused, documented | refused (status 83: "libpandoc.wasm runs no programs") | refused, documented | can't run |
| **a script's `read(text, ctx.conversion)`** | **reads inside and outside** (H1; the README says pandoc's sandbox); since 1.13: neither | – | **reads inside and outside** (H1); since 1.13: neither | – |
| a function's `read(text, ctx.conversion)` | pandoc's sandbox: neither | – | pandoc's sandbox: neither | the instance: inside only; `sandbox: true`: neither |
| a callback's `read_many(read_options(conversion))` | – | **reads inside and outside** (H1); since 1.13: neither | – | – |
| a wasm filter's `read`, `read_many` | untrusted: neither | untrusted: neither | untrusted: neither | untrusted: neither |
| `read(text, "latex")` with no conversion, anywhere in the process | reads both (no sandbox asked) | reads both | reads both | – |

`pandocjl` on `main` ignores the variables (H13).

pantable's `include` (2026-11-17, the hosts on `adopt-1.13`, `pandocjl`
with its sandbox) gives the same: its Python port as a script
(`pandocpy`) and as functions (`python-api`), and its Julia port as a
script (`pandocjl`), read all three canaries, by design (the filter is the
user's code, its reads its own); its Rust port as wasm reads none in
pandocrs, pandocpy and pandocjl (the include fails: a wasm filter sees the
mounts only), as outside the sandbox (`known.tsv`); every case inside the
directory gives the original's result in every mode. Its markdown cells
are read with `read_many` and the conversion, so in pandoc's sandbox when
the conversion is (H1, closed); markdown reads no files.

## Calls into pandoc from a filter

What each host offers a filter, and what a call costs (from inside a
filter, warm, best of three; 738 notes of a textbook, half plain text and
half with emphasis, math and a citation, read as markdown):

| | Lua | Python | Rust, wasm | Rust, callback | Julia | JavaScript |
|---|---|---|---|---|---|---|
| read one text | `pandoc.read` | `libpandoc.read(text, conversion)` | `libpandoc::read`, `read_as` | the same | `LibPandoc.read(text, conversion)` | none for a filter (H2); since 1.13 `pandoc.read(text, conversion)`, the instance a filter is given |
| read many at once | none | `read_many` | `read_many`, `read_many_as` | the same | `read_many` | the instance's `readMany` |
| citeproc | `pandoc.utils.citeproc` | `convert(json, from_="json", to="json", citeproc=True)` | `libpandoc::convert` (no files: H3) | `convert` | `convert(...; citeproc = true)` | none (H2); since 1.13 `pandoc.convert({..., citeproc: true}, json)` |
| write | `pandoc.write` | `write` | `write` | the same | `write` | none; since 1.13 `pandoc.convert` |
| 738 notes, a call each (s) | 0.04 | 0.25 | 0.23 (pandocrs), 0.30 (pandocpy), 0.24 (pandocjl), 0.67 (Node) | 0.21 | 0.23 | 0.24 (the instance's) |
| the same, one `read_many` (s) | – | 0.015 | 0.014, 0.014, 0.016, 0.18 | 0.013 | 0.012 | 0.17 |
| amsthm on the textbook, a note on each of 738 environments, against none (s) | +0.06 | +0.25 | +0.15 (libpandoc-rs), +0.28 (libpandoc-python) | (no reader natively: +0.02) | +0.21 | (no reader: H2) |

A call from a filter costs about 0.3 ms in every native host, six times
`pandoc.read`'s 0.05 ms (H15); batched, less than Lua's loop. Every host
lets a filter batch (`read_many`; JavaScript's instance `readMany` since
1.13); of the ports, pantable's only, as its original reads a table's
texts in one process: a table of 1,001 rows by 10 markdown cells (10,011
texts), the reads alone, warm (Rust: a process each, its filter's own
timing):

| | Python | Julia | Rust, wasm (pandocrs, pandocpy, pandocjl) | JavaScript (Node) | the original (`pandoc` on its joined document) |
|---|---|---|---|---|---|
| a call a text (s) | 3.50 | 0.58 | 0.58, 0.92, 0.73 | 3.03 | – |
| one `read_many` (s) | 0.20 | 0.18 | 0.20, 0.17, 0.22 | 2.73 | 0.51 |
