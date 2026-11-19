# Where the YAML is

Closed by `...`:

```table
---
caption: closed by dots
...
a,b
```

Keys with underscores, and an unknown key:

```table
---
short_caption: underscore
fancy_table: false
colour: blue
---
a,b
```

A data line of three dots, which starts no YAML:

```table
a,b
...
c,d
```

Blank lines around the data are dropped, inside it kept:

```table
---
caption: blank lines
---


a,b

c,d


```
