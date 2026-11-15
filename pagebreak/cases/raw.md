---
abstract: |
  Before.

  \newpage

  After.
---

The LaTeX commands, alone in a raw block:

\newpage

\pagebreak

\newpage{}

\pagebreak{}

```{=latex}
\newpage{
```

```{=tex}
\pagebreak}
```

```{=context}
\newpage
```

Not alone, or not TeX:

```{=latex}
\newpage
\newpage
```

```{=html}
\newpage
```

`\newpage`{=latex}

\clearpage

> \newpage

::: aside
- \pagebreak
:::
