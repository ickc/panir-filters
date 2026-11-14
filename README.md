# panir-filters

Real pandoc filters, ported to each language of
[panir](https://github.com/ickc/panir) (Python, TypeScript, Rust, Julia)
and checked against their originals, to find what panir's filter API lacks
or makes awkward. The list of gaps, [`GAPS.md`](GAPS.md), is the product;
the ports are the means.

Round 1 (filters that need only the AST) is chosen in
[`SURVEY.md`](SURVEY.md) and listed, with each port's status, in
[`filters.tsv`](filters.tsv). Ported so far: `caps`, `metavars`,
`list-table`; lines and times in [`RESULTS.md`](RESULTS.md).

## What goes here

One folder per filter:

- `LICENSE`: the original's licence, verbatim; it covers the original and
  the ports in the folder;
- `upstream/`: the original, as it is, and `ORIGIN` (where it came from,
  the commit, the licence, how it runs, the output formats its cases are
  run for);
- `cases/`: input documents (`*.md`, read by pandoc's markdown reader, or
  `*.json`, a document as pandoc's JSON), and `expected.jsonl`, what the
  original made of each under pandoc 3.14, one line per document and
  format (`{"name", "format", "input", "output"}`, or `"error": true` where
  the original fails); committed, so the tests need neither pandoc nor the
  original's runtime;
- `python/`, `ts/`, `rust/`, `julia/`: the ports, each written as someone
  using that language and panir would write it;
- `NOTES.md`: what was awkward or missing, per language, and every place
  a port differs from its original;
- `known.tsv`, if any: the differences `NOTES.md` explains (`language`,
  case or document, perhaps with the one format it differs in, as `name
  (markdown)`, why); they don't fail a run, and one that stops differing
  does.

At the top: `run.py`, the runner; one Cargo workspace for the Rust ports
(`Cargo.toml`), one Node.js package for the TypeScript ones and the type
checkers (`package.json`, `tsconfig.json`, `pyrightconfig.json`), one
Julia project (`Project.toml`); `tojson.lua`, which writes what an
original Lua filter made; `oracle-requirements.txt`, what the Python
originals need; `julia-cases.jl`, which runs the Julia ports' cases in one
process; `fetch-docs.sh`, pandoc's test documents for the differential
runs, and `assemble.py`, documents made of the cases for them.

## Running

panir is a checkout beside this one (`../panir`, or `$PANIR`), with its
TypeScript package built (`npm ci && npm run build` in `panir/ts`). Then,
once:

    npm ci                                                # TypeScript, tsc, pyright
    julia --project=. -e 'using Pkg; Pkg.instantiate()'   # Panir.jl from ../panir/julia

and:

    python3 run.py test                  # every port on every case
    python3 run.py test list-table --lang rust,julia -v
    npm run check                        # tsc and pyright on the ports

`run.py` builds the Rust ports (`cargo build --release`), and runs each
port as pandoc runs a JSON filter: Python with panir from
`../panir/python/src` (`$PYTHON`, default `python3`), TypeScript with
Node.js 22.18 or later as it is (`$NODE`), Rust from `target/release`,
Julia with this project (`$JULIA`), whose cases `test` runs in one Julia
process (each case in a new module, through `Panir.handoff`: a process per
case compiles Panir's walk again each time). Two documents are the same
when their JSON is ("the same AST"), `pandoc-api-version` aside; an
original's output is taken as pandoc reads it (`pandoc -f json -t json`
for a Python one), as pandoc reads what a filter writes.

Against the originals, which needs pandoc 3.14 (`$PANDOC`) and, for the
Python originals, a Python with `oracle-requirements.txt`
(`$ORACLE_PYTHON`):

    python3 run.py diff caps --docs .cache/pandoc-tests/* some/doc.json   # documents it wasn't written for
    python3 run.py expect caps           # regenerate cases/expected.jsonl
    python3 run.py time caps --doc doc.json   # time and peak memory, original and ports

`diff` takes pandoc JSON documents (`.json`) or markdown (anything else,
read by pandoc with `--sandbox`). The ones used so far: `./fetch-docs.sh`
(pandoc's test documents), some of libpandoc's benchmark documents (the
pandoc manual, a Wikipedia article as HTML, a textbook in LaTeX, Python's
documentation in RST, a novel as EPUB...), and, as real documents rarely
hold a filter's syntax, `python3 assemble.py` (five documents made of
every filter's cases, shuffled, at several depths, in
`.cache/assembled`); none are kept here.

CI (`.github/workflows/ci.yml`) runs `run.py test` for the four languages
on Linux, with `tsc`, pyright, rustfmt and clippy, panir's main checked
out beside this repository; it needs neither pandoc nor the originals.

## Licences

A port derives from its original and keeps the original's licence, stated
in the filter's folder. What is this repository's own (the harness, the
notes) is under the BSD 3-Clause licence in `LICENSE`.

Originals under the GPL are not ported from their source. If one is done
at all, it is written anew without reading that source (from its
documentation, its behaviour and its tests, which are fetched and not kept
here), and published under the original's licence.
