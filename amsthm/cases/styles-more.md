---
amsthm:
  styles:
    plain:
      headfont: smallcaps
      bodyfont: [bold, italic]
      headspace: 0.5em
    proof: {headfont: bold}
    empty: {}
    quiet:
      headfont: [normal, loud]
      bodyfont: smallcaps
      headpunct: ""
    broken: text
  plain: [Theorem]
  empty: [Empty]
  quiet: [Quiet\*]
  broken: [Broken]
---

::: {#t .Theorem info="Smith"}
In bold italics, citing @t and [@t], *emphasis* and **bold** and [small]{.smallcaps}, and \ref{t} and \eqref{t}.

**Bold** *text*.
:::

::: Empty
A style of its own, all defaults.
:::

::: Quiet
Small capitals: @t, [@t].
:::

::: Broken
A style that is text: not one.
:::
