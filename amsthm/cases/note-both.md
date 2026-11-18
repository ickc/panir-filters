---
amsthm:
  plain: [Theorem]
---

::: {.Theorem info="the attribute"}
[the span]{.info} Both: the span wins, with a warning.
:::

::: {.Theorem info="the attribute"}
[]{.info} An empty span and the attribute: a warning, and no note.
:::

::: {.Theorem info=""}
[the span]{.info} An empty attribute and a span: the span, no warning.
:::

::: {.proof info="the *attribute*"}
[the span]{.info} A proof with both: the attribute's markup is not looked at.
:::

::: {.Theorem info="the attribute"}
A [later span]{.info} is not the note: the attribute is.
:::

::: {.Theorem info="*markup*"}
The first note of markup in the attribute alone: the one warning.
:::
