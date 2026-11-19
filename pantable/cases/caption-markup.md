# A plain table's caption: markdown stays text

```table
---
caption: "*Not* **emphasis**"
---
a,b
```

Escapes, entities and raw HTML:

```table
---
caption: "a \\* b &amp; <b>c</b>"
---
a,b
```
