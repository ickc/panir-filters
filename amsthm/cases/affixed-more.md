---
amsthm:
  plain: [Theorem*, Lemma]
---

::: {#u .Theorem}
Unnumbered.
:::

::: {#a .Lemma}
Numbered.
:::

[see @u] has a prefix and names an unnumbered environment: two warnings.
[-@a, p. 2] suppresses the author and has a suffix; @A [p. 3] is named
and has a suffix; [see @knuth] is not an environment; \ref{a} and
[-@a] are as before.

::: Lemma
In an italic body, @a [p. 3] and [@a, p. 3].
:::
