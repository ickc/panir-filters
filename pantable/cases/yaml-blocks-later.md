# YAML blocks not starting the code block

Two YAML blocks, the second after the data, merged (the later wins):

```table
---
caption: first
alignment: R
---
a,b
1,2
---
caption: second
---
```

The data before the YAML:

```table
a,b
---
caption: after
---
```

Longer delimiters, and a YAML block never closed:

```table
-----
caption: long dashes
.....
a,b
---
alignment: C
```
