# YAML that can't be scanned: the block is left as it is

```table
---
caption: "unclosed
---
a,b
```

```table
---
caption: a: b
---
a,b
```

The table after is converted:

```table
a,b
```
