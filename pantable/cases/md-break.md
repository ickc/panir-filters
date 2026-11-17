# A cell that closes a div

The original reads all the cells of a table as one document, each in a
div; a cell holding a line `:::` closes its div early:

```table
---
markdown: true
---
a,b
"before
:::
after",c
```
