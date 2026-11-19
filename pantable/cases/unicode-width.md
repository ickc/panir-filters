# Widths from the length of the text

East Asian wide characters, each as wide as two letters when shown:

```table
---
table-width: 1
---
語,Word
中文字符,abcd
```

Characters outside the Basic Multilingual Plane (two UTF-16 units each),
combining marks, and a family emoji of several code points:

```table
---
table-width: 1
---
😀😀😀,abc
e̊e̊,👨‍👩‍👧
```

Line breaks as Python's `str.splitlines` finds them, from a file: a line
break inside a quoted field (CRLF in the file, a line feed once
included), a form feed, a vertical tab, U+2028, U+0085:

```{.table include="lines.csv"}
---
table-width: 1
---
```
