# A tab as the delimiter: the two characters `\t`

In single quotes (the data from a file: pandoc's reader expands a tab in
a code block):

```{.table include="tabs.tsv"}
---
csv-kwargs:
  delimiter: '\t'
---
```

Escaped in double quotes:

```{.table include="tabs.tsv"}
---
csv-kwargs:
  delimiter: "\\t"
---
```
