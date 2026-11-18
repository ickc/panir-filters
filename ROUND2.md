# Rounds 2 and 3, in a page

Round 1 tested panir, the AST and its filter API (`ROUND1.md`). Rounds 2
and 3 test the **hosts**, libpandoc's bindings, which run a filter inside
a pandoc conversion, where it can ask the pandoc it runs in to read a
fragment or run citeproc with no second process, and where a directory
sandbox can confine the conversion: libpandoc-python (`pandocpy`,
`libpandoc.convert`), libpandoc-rs (`pandocrs`, its `libpandoc` crate,
wasm filters), LibPandoc.jl (`pandocjl`, `LibPandoc.convert`) and
libpandoc's JavaScript host (libpandoc.wasm in Node). Each was built from
its `main` against libpandoc 1.12, and again against 1.13 and 1.14
(`HOSTS.md` has the commits).

## What was done

- **The hosts run every port** (`run.py host`): the 24 filters' ports on
  their cases, each inside its language's hosts and compared with the
  same expected output: Python as a script in `pandocpy` and as filter
  functions through `libpandoc.convert`; Rust built for wasm32-wasip1 in
  pandocrs, pandocpy, pandocjl and Node, and natively as a JSON filter run
  by pandocrs; Julia as a script in `pandocjl`; TypeScript as a JS
  function inside libpandoc.wasm in Node. RESULTS.md has the counts: every
  round 1 port the same in every host, but pagebreak's environment in the
  wasm hosts (H8).
- **The writer's options** (round 1's open gap 38): libpandoc gives every
  filter they run `PANDOC_WRITER_OPTIONS`; amsthm's ports read them there
  where no host gives the options, and the Rust port's 36 known
  differences are gone, in every host and as a JSON filter run by
  libpandoc.
- **Round 2**: include-files (pandoc-ext; the owner co-maintains it) and
  include-code-files (pandoc/lua-filters), in the four languages: 15 and
  11 cases (nested and relative includes, a missing file, files outside
  the directory through `..`, a symbolic link and an absolute name, line
  ranges, dedent on bytes, other formats, reader options). Reading a file
  is each language's; parsing it is pandoc's, so include-files is also
  round 3's first: Python and Julia read with libpandoc in their process,
  Rust built for wasm with the host's pandoc, TypeScript with the `pandoc`
  command (no JavaScript host gives a filter its pandoc: H2). Both ran in
  every host's **directory sandbox**: everything inside the directory is
  the original's, and `HOSTS.md` tables what each kind of filter reads
  outside it.
- **Round 3**: amsthm's notes are read by the host's pandoc where a port
  has one (every host but JavaScript), and are then the original's in
  every case; standalone, the ports keep round 1's plain-text rule.
  multibib (pandoc-ext) runs citeproc from inside a
  filter, twice: in the four languages, the same in every host but the
  wasm ones (H3); since libpandoc 1.14 in those too, the Rust port reading
  the bibliography files itself (`libpandoc::citeproc`) and giving pandoc
  their contents. A call into pandoc from a filter costs about 0.3 ms
  in every native host, six times `pandoc.read`'s, and batched
  (`read_many`, which every host but JavaScript's offers a filter) less
  than Lua's loop (H15).
- **pantable** (the owner's; BSD-3-Clause; 2026-11-17, on libpandoc 1.13
  and panir 0.2): CSV in a code block with YAML options made a table, in
  the four languages, 80 cases (its own tests and examples among them).
  Its `include` is round 2's, its markdown cells round 3's, and the first
  port that **batches**: the original reads a table's texts in one
  `pandoc` process, as one document of divs; the ports read them with one
  `read_many` (`pantable/NOTES.md` has what the joined document did that
  the ports don't). CSV and YAML are each language's library (the owner
  ruled out replicating Python's `csv` module: `csv-kwargs` is Python's
  only); `pantable2csv` left out. Its bugs, for the owner, in its
  `NOTES.md`.

## What the hosts lack

`HOSTS.md`, ranked: H1 a filter's `read(text, ctx.conversion)` reads
files outside a command's directory sandbox (pandocpy and pandocjl
scripts, Rust callbacks; the READMEs say pandoc's sandbox); H2 a
JavaScript filter can't reach the pandoc it runs in, and has no panir
glue; H3 a wasm filter's calls to pandoc can't read the files the filter
can (citeproc's bibliographies); H4 libpandoc's `read` expands tabs where
`pandoc.read` doesn't; H5 a filter's module state outlives its document
in some hosts (a Julia script) and not in others (a Python script); H6
and H7 a wasm filter sees only the working directory on a command line,
and none in Node; H8 nor the process's environment; H9 to H13 smaller
(Rust's callbacks get no options, a wasm filter that may call pandoc needs
a host, WASI's empty name, binary output in JavaScript, `pandocjl`'s
`main` ignoring `LIBPANDOC_SANDBOX`); H14 every filter in a host is handed
the whole document as JSON (the cost, not the process, is pandoc's
encoding), H15 a call into pandoc costs six times `pandoc.read`'s; with
pantable, H16 a Julia script's packages come only from `pandocjl`'s load
path, H17 `pandocjl` cuts a Lua filter's output to a pipe at 64 KiB, H18
`read_many`'s errors name a text by its place in the batch. (Since, in
`HOSTS.md`: closed are H1, H2, H3, H4, H6, H9, H12, H13 and H17;
H7's fix reversed, a wasm filter being given no directory by default in
every host, Node's included, and `LIBPANDOC_WASM_DIRS` the user's way to
give some; documented H5, H8, H10, H11, H15, H16 and H18; H14 measured,
82% of a callback filter's cost pandoc decoding the JSON it returns, what
to build the owner's decision, not made.) What each filter needs from a
host, from the runs: 19 of the 25 are pure, 4 read files, 1 the
process's environment, 4 call pandoc (`HOSTS.md`, "What each filter
needs"). And in panir
(`GAPS.md` 41 to 46): pandoc.path's functions, Lua's `tonumber`, a
fragment's walk that forgets the conversion, Python's list constructors,
and with pantable a table from a grid of cells and a parser of pandoc's
attribute syntax.

## Time inside a host

`RESULTS.md`, "Inside a warm host": caps, list-table, auto-lang and
amsthm on the textbook and War and Peace, standalone, in a host's
command, and warm. A filter in a native host pays pandoc's JSON both ways,
0.35 s a filter on the 9.4 MB textbook whatever it does, as much as a
JSON filter's process (H14), and four times a Lua filter that touches
little (list-table's original: 0.09 s). Where a filter works on every
node the ports win: auto-lang on War and Peace 1.0 s (Rust), 1.4
(Julia), 1.8 (Python) against Lua's 4.1 to 4.4. Julia gains most from a
warm host: amsthm 3.4 s standalone or in `pandocjl` (its system image
saves 0.1 to 0.25 s), 0.7 warm. In Node everything runs in libpandoc.wasm,
3.5 times slower.
