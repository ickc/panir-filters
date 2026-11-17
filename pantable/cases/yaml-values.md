# YAML 1.1's values

Octal, sexagesimal, and underscores:

```table
---
caption: 010
short-caption: 1:30
---
a,b
```

```table
---
caption: 1_000
short-caption: 0x1F
---
a,b
```

Floats with and without a dot, and infinity:

```table
---
caption: 1e3
short-caption: 1.0e+3
---
a,b
```

```table
---
caption: .inf
short-caption: -.Inf
---
a,b
```

Booleans of YAML 1.1 and 1.2, as captions:

```table
---
caption: on
short-caption: True
---
a,b
```

```table
---
caption: Off
short-caption: FALSE
---
a,b
```

The null of `~`:

```table
---
caption: ~
---
a,b
```
