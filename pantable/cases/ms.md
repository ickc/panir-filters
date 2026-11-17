# Row blocks (ms) and row heads (ns-head), without markdown

A head of one row, a body of two, a foot of one:

```table
---
ms: [1, 0, 2, 1]
---
h1,h2
a,b
c,d
f1,f2
```

Two bodies, each with a head row, and a row head column in the second:

```table
---
ms: [0, 1, 1, 1, 2, 0]
ns-head: [0, 1]
---
bh1,x
b1,y
bh2,z
b2,w
b3,v
```

No head (`ms` takes over from `header`):

```table
---
ms: [0, 0, 2, 0]
header: true
---
a,b
c,d
```

Of odd length, too short, negative, and not adding up to the rows: each
dropped:

```table
---
ms: [1, 0, 1]
---
a,b
c,d
```

```table
---
ms: [1, 1]
---
a,b
c,d
```

```table
---
ms: [3, 0, -1, 0]
---
a,b
c,d
```

```table
---
ms: [1, 0, 5, 0]
---
a,b
c,d
```

`ns-head` of the wrong length, more than the columns, and as text:

```table
---
ns-head: [1, 1]
---
a,b
c,d
```

```table
---
ns-head: [3]
---
a,b
c,d
```

```table
---
ns-head: "1"
---
a,b
c,d
```

Not numbers:

```table
---
ms: [a, b, c, d]
ns-head: [x]
---
a,b
c,d
```
