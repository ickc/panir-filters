# fancy-table

A head, a body with head rows, a second body, a foot, attributes on
blocks and rows, and a first column that can't be read:

```table
---
markdown: true
fancy-table: true
ns-head: [1, 0]
---
{#head} === {.hrow},H1,H2
,bh,bh2
{.b1} --- {.r},x,y
,b,b2
___,c,c2
nonsense,d,d2
___ {#last-body-row},e,e2
=== {.foot},F1,F2
```

Bodies only (no `===` first), and a body after a body:

```table
---
markdown: true
fancy-table: true
---
,a,b
___,c,d
___,e,f
```

A foot needs two markers: a single `===` is the head:

```table
---
markdown: true
fancy-table: true
---
,a,b
===,c,d
```
