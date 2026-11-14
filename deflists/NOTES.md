# deflists

pandocfilters' `examples/deflists.py`: a definition list becomes a bullet
list, an item per term: the term in bold as a paragraph, then the blocks
of all its definitions.

## The original's behaviour, kept

- **Definitions run together**: a term's definitions become one item's
  blocks, nothing between them; compact definitions (`Plain`) stay
  `Plain`, so two of them read as one paragraph in most writers (case
  `edges`).
- **Nested lists, lists in items, quotes and the metadata** are converted
  too (`edges`). The original is top-down; the result is the same in any
  order, so the ports use panir's default (typewise).
- A term with no definitions gives an item of its term alone; an empty
  term a paragraph with an empty bold; an empty list an empty bullet list
  (case `bare`, from JSON).

## By language

All four are a single expression, building the bullet list from the
definition list's pairs; nothing missing, and nothing new from panir#14
needed. Rust takes the pairs out with `std::mem::take` and builds each
item with `iter::once(...).chain(defs.into_iter().flatten())`; Julia
`[Para(Strong(term)); reduce(vcat, defs; init = Block[])]`; TypeScript
`defs.flat()`. Julia's `BulletList(items)`, given one vector of vectors,
takes it as the items (and one vector of blocks as one item), as its
README says.

Differential runs (pandoc's test documents; eight real documents, the
pandoc manual with 44 definition lists, Python's documentation with 9;
five documents made of every filter's cases): the same AST in every
language.
