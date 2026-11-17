# A first row starting with ---

panflute's YAML filter starts a YAML block at a chunk of the code that
begins with `---`: the data after the YAML is that chunk.

```table
---
markdown: true
fancy-table: true
...
---,head of a body,x
___,a body row,y
```
