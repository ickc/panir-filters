---
amsthm:
  plain: [Theorem]
  qed_symbol: \blacksquare
---

::: proof
Twice in one display: $$a \qedhere b \qedhere$$ and not \qedherex: $\qedherex$.
:::

::: proof
After spaces and a break \qedhere
:::

::: proof
Raw LaTeX: `\qedhere`{=latex}, raw HTML: `\qedhere`{=html}, raw TeX with spaces: `  \qedhere  `{=tex}.
:::

::: proof
In a theorem in the proof:

::: Theorem
$$x \qedhere$$
:::
:::

::: Theorem
A theorem's $\qedhere$ is not a proof's.
:::
