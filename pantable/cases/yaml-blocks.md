# Where the YAML is

Closed by `...`:

```table
---
caption: closed by dots
...
a,b
```

Two YAML blocks, the second after the data, merged (the later wins):

```table
---
caption: first
alignment: R
---
a,b
1,2
---
caption: second
---
```

The data before the YAML:

```table
a,b
---
caption: after
---
```

Longer delimiters, and a YAML block never closed:

```table
-----
caption: long dashes
.....
a,b
---
alignment: C
```

Keys with underscores, both spellings (the later wins), and an unknown key:

```table
---
short-caption: hyphen
short_caption: underscore
colour: blue
---
a,b
```

A key given twice:

```table
---
caption: once
caption: twice
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
