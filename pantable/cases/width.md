# Widths

```table
---
width: [0.25, 1/4, D, 3]
---
a,b,c,d
```

Negative, text, missing, and more than the columns:

```table
---
width: [-0.5, abc, 1/3, 0.1, 0.2]
---
a,b,c
```

A boolean and an integer:

```table
---
width: [true, 0]
---
a,b
```

Fractions written other ways:

```table
---
width: ["  2/10 ", 1e-1, 1_0/1_00, +.5]
---
a,b,c,d
```

A number, not a list:

```table
---
width: 0.5
---
a,b
```

Text, not a list:

```table
---
width: "12"
---
a,b
```

Not a number:

```table
---
width: [.nan, .NaN]
---
a,b
```
