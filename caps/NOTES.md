# caps

pandocfilters' `examples/caps.py`: every `Str` upper-cased.

All four ports are panir's own upper-case example, nearly word for word:
the trivial filter needs nothing panir lacks. What it does show:

- **The metadata is walked too.** pandocfilters walks the whole JSON, so
  the title is upper-cased as well; panir's walks do the same (as pandoc's
  Lua does), in every language and order. Nothing to do in the ports;
  case `meta`.
- **Upper case is the language's, and Julia's differs.** Python's
  `str.upper`, JavaScript's `toUpperCase` and Rust's `to_uppercase` apply
  Unicode's full case mapping (`ß` to `SS`, `ŉ` to `ʼN`, `ﬁ` to `FI`);
  Julia's `uppercase` maps each character to one (`ß` to `ẞ`, `ŉ` and `ﬁ`
  unchanged), and Julia's standard library has no full mapping. So the
  Julia port differs on those letters: case `unicode`, and two real
  documents (the pandoc changelog's "Voß", Python's documentation's
  `'straße'.lower()`), listed in `known.tsv`. Making it the same would mean
  carrying Unicode's SpecialCasing table in the port; not panir's job, and
  not done. (The Unicode versions differ too, Python 17, Node.js 18, Rust
  17: no letter of the documents tried is affected.)

panir#14 (the gaps the first ports found) changed nothing here: Rust's
`apply` now returns a `Result`, which `panir::filter` takes as it is.

Differential runs (pandoc's test documents, eight real documents): the
same AST in Python, TypeScript and Rust everywhere; Julia as above.
