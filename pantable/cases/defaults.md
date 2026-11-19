---
pantable:
  caption: From the metadata
  alignment: RC
  table-width: 1
---

# Defaults from the document's metadata, under `pantable`

A table without a header takes them:

```table
a,b
```

A header's key replaces the default's whole value; the others stay:

```table
---
caption: Its own
---
a,b
```
