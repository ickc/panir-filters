# Encodings: the included file is UTF-8

A byte-order mark is dropped:

```{.table include="bom.csv"}
```

Latin-1 and UTF-16 are not UTF-8: the block is left as it is:

```{.table include="latin1.csv"}
```

```{.table include="utf16.csv"}
```

`include-encoding` was the `include` key's: beside the attribute it is
read by no one, a name Python knows or not:

```{.table include="latin1.csv"}
---
include-encoding: latin-1
---
```

```{.table include="plain.csv"}
---
include-encoding: klingon
---
```
