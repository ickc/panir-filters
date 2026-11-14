An empty list table, and one with only a caption, stay as they are:

::: list-table
:::

::: {.list-table #only-caption}
A caption, and nothing else.
:::

A div whose first class is something else stays too:

::: {.other .list-table}
- - a
:::

A body without a head, row attributes, widths without alignments, a
header column:

::: {.list-table-body widths=3,1 header-cols=1 data-x=kept}
* []{#row-1 .odd}
  - a
  - b
* - c
  - d
:::

Two bodies, the second from a table in an item (its head is dropped); the
header column only in the first:

::: {.list-table header-cols=1 header-rows=abc}
* - head 1
  - head 2
* - one
  - two
* | x | y |
  |---|---|
  | 1 | 2 |
  | 3 | 4 |
:::

A table as the first item, then more rows, in a new body:

::: {.list-table header-rows=0}
* | x | y |
  |---|---|
  | 1 | 2 |
* - three
  - four
:::

More header rows than rows; spans with alignment, rows and columns:

::: {.list-table header-rows=5 aligns=r,c}
* - []{colspan=2 rowspan=1 align=c} wide
* - []{align=x} *emphasis* and more
  - []{} empty attributes
:::

A list table in a cell of another:

::: list-table
* - outer
  - ::: list-table
    * - inner a
      - inner b
    * - inner c
      - inner d
    :::
:::

A cell whose first block is a header, with attributes in a span:

::: list-table
* - # []{.h colspan=1} Heading in a cell
  - plain
:::
