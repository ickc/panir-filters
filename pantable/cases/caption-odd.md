# A list, a map, booleans and nothing as captions

```table
---
caption: [one, 2, true]
---
a,b
```

```table
---
caption: {key: value, n: 1}
---
a,b
```

`yes` is YAML 1.1's boolean, `false` 1.2's:

```table
---
caption: yes
short-caption: false
---
a,b
```

An empty value, YAML's null:

```table
---
caption:
---
a,b
```
