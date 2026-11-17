# Alignment

```table
---
alignment: LRCD
---
a,b,c,d
1,2,3,4
```

Lower case, and fewer letters than columns:

```table
---
alignment: rc
---
a,b,c
```

More letters than columns, and blanks around:

```table
---
alignment: "  CLRCLR  "
---
a,b
```

Letters that mean nothing, digits, a blank inside, punctuation:

```table
---
alignment: XYZ0 9.
---
a,b,c,d,e,f,g
```

A number:

```table
---
alignment: 12
---
a,b
```

A character that is not ASCII makes every column default:

```table
---
alignment: Lä
---
a,b
```

Upper case of ß is two letters:

```table
---
alignment: ßl
---
a,b,c
```
