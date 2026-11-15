---
amsthm:
  plain: [Theorem]
  definition: [Definition]
---

::: Theorem
# A heading first

Text with *emphasis*.

- a list, then
- a [div]{.x}:

::: {.x amsthm-held="1"}
A user's div with the attribute the original uses.
:::

::: Definition
A definition held out of the italics.

::: Theorem
A theorem in it.
:::
:::

1. *Emph* *again* and *one* more.
:::

::: Theorem
:::

::: proof
It ends in code:

```
x = 1
```
:::

::: proof
# A heading last
:::

::: proof
- a list last
:::

::: proof
:::

::: {.Theorem .Definition}
Two environments: left alone.
:::

::: {.Theorem .Theorem .amsthm-plain}
The class amsthm-plain already there.
:::

> ::: Definition
> In a quote.
> :::
