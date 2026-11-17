# Alignment of cells

```table
---
alignment: RR
alignment-cells: |
  LC
  CL
---
a,b
1,2
```

Fewer rows and columns than the table, and lower case:

```table
---
alignment-cells: |
  c
---
a,b
1,2
3,4
```

More rows than the table, letters that mean nothing, and a line not ASCII:

```table
---
alignment-cells: |
  xyz
  Lé
  RR
  CC
---
a,b,c
1,2,3
```

On one line:

```table
---
alignment-cells: RC
---
a,b
1,2
```
