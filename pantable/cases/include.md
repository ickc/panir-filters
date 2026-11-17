# Including a CSV file

Relative to the directory pandoc runs in, in a subdirectory, and in place
of the block's own data:

```table
---
include: plain.csv
---
```

```table
---
include: sub/nested.csv
caption: nested
---
ignored,data
```

An empty `include` reads the block:

```table
---
include: ""
---
from,the block
```

Not found: the block is left as it is:

```table
---
include: no-such-file.csv
---
```

The table after it is converted:

```table
a,b
```
