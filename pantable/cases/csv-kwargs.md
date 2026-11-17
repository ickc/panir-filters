# csv-kwargs

A delimiter:

```table
---
csv-kwargs:
  delimiter: "|"
---
a|b, c|d
```

A quote character, and blanks after the delimiter skipped:

```table
---
csv-kwargs:
  quotechar: "'"
  skipinitialspace: true
---
a, 'b, c', d
1,  '2',3
```

An escape character instead of doubled quotes:

```table
---
csv-kwargs:
  escapechar: "\\"
  doublequote: false
---
"a \"quoted\" word",b\,c
```

No quoting at all (`csv.QUOTE_NONE`):

```table
---
csv-kwargs:
  quoting: 3
---
"a","b"
```

The `unix` dialect, and `excel` named:

```table
---
csv-kwargs:
  dialect: unix
---
a,"b"
```

```table
---
csv-kwargs:
  dialect: excel
  delimiter: ;
---
a;b
```

A tab-separated file:

```table
---
include: tabs.tsv
csv-kwargs:
  dialect: excel-tab
---
```

`strict` on well-formed input:

```table
---
csv-kwargs:
  strict: true
---
a,"b"
```
