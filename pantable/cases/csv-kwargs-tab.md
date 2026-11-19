# A tab as the delimiter: YAML's escape

A tab character, which pandoc reads as nothing (the data from a file:
pandoc's reader expands a tab in a code block):

```{.table include="tabs.tsv"}
---
csv-kwargs:
  delimiter: "\t"
---
```
