# Including a CSV file: the include attribute

Relative to the directory pandoc runs in (the document's, given on
standard input), in a subdirectory, and after the block's own rows:

```{.table include="plain.csv"}
```

```{.table include="sub/nested.csv"}
---
caption: nested
---
own,rows
```

An empty `include` names no file: the block is left as it is:

```{.table include=""}
from,the block
```

Not found: the block is left as it is:

```{.table include="no-such-file.csv"}
```

The table after it is converted:

```table
a,b
```
