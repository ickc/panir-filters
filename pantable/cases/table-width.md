# Table width

```table
---
table-width: 1
---
Short,A longer column,x
a,bb,ccc
```

A fraction, with one width given:

```table
---
table-width: 2/3
width: [1/6]
---
Short,A longer column,x
a,bb,ccc
```

Zero:

```table
---
table-width: 0
---
a,b
```

Less than the widths given: dropped, and the widths kept:

```table
---
table-width: 0.5
width: [0.4, 0.3]
---
a,b,c
```

Equal to the widths given, with the rest to share:

```table
---
table-width: 0.7
width: [0.4, 0.3]
---
a,b,c
```

Text that is no number:

```table
---
table-width: wide
---
a,b
```

Every cell empty but one, which spans lines:

```table
---
table-width: 1
---
,"one
two lines"
,
```
