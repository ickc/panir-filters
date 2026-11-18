---
amsthm:
  plain: [Theorem]
  definition: [Definition]
  remark: [Remark*]
---

::: Theorem
[A note alone]{.info}
:::

::: Theorem
[A note before code]{.info}

    code
:::

::: Theorem
[A hard line break after]{.info}\
stays.
:::

::: Theorem
[No space after]{.info}takes nothing with it.
:::

::: Theorem
[ ]{.info} A span of a space is no note, and goes.
:::

::: Theorem
[Small caps]{.info .smallcaps} as pandoc reads the class.
:::

::: Theorem
*[In emphasis]{.info}* is not the first inline itself: an ordinary span.
:::

::: Theorem
# [In a heading]{.info}, not a paragraph

An ordinary span.
:::

::: Theorem
A first paragraph.

[In the second paragraph]{.info} is an ordinary span.
:::

::: Definition
[*Emphasis*, `code`, a footnote^[A footnote.] and [a link](https://example.org)]{.info} In a definition.
:::

::: Remark
[An unnumbered remark's note]{.info} A remark.
:::

::: {.Theorem .Definition}
[Two environments]{.info} The div is left alone, and the span with it.
:::

::: proof
[Proof with *emphasis* and $x$]{.info} Obvious.
:::

::: proof
[A proof's note before a list]{.info}

- one
:::

::: Theorem
::: Definition
[Only the inner one]{.info} The outer has none: its first block is the inner.
:::
:::
