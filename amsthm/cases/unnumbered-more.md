---
amsthm:
  plain:
    - Aside*: [Remark, Note*]
  definition:
    - Quiet*: Hint
  parent_counter:
    Remark: section
---

# One

::: Remark
Within the section, as its own parent_counter says.
:::

::: Note
Unnumbered itself: no warning.
:::

::: Hint
A single value under an unnumbered environment.
:::

# Two

::: Remark
Restarted.
:::
