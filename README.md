# panir-filters

Real pandoc filters, ported to each language of
[panir](https://github.com/ickc/panir) (Python, TypeScript, Rust, Julia)
and checked against their originals, to find what panir's filter API lacks
or makes awkward.

Not started yet: this is the plan.

## What goes here

One folder per filter:

- `upstream/`: the original, as it is, with where it came from (URL and
  commit) and its licence;
- `cases/`: input documents and the AST the original gives for each under
  pandoc (pandoc with the original is the oracle: the same AST is enough);
- `python/`, `ts/`, `rust/`, `julia/`: the ports, each written as someone
  using that language would write it;
- `NOTES.md`: what was awkward, per language.

`GAPS.md` will collect what the ports found missing in panir, each with a
proposed change.

## Licences

A port derives from its original and keeps the original's licence, stated
in the filter's folder. What is this repository's own (the harness, the
notes) is under the BSD 3-Clause licence in `LICENSE`.
