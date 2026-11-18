---
amsthm:
  plain: [Theorem]
  definition: [Definition]
---

::: {.Theorem info="Gauß, Ἀρχιμήδης and 祖冲之"}
Letters of any script, and a comma: plain text.
:::

::: {.Definition info="well-known; or not? yes! 1801: x (a/b)"}
Punctuation the test takes as plain.
:::

::: {.Theorem info="Fermat’s"}
A typed curly apostrophe: plain text.
:::

::: {.Theorem info="e.g. this -- and that..."}
Plain by the test: smart typography is not applied.
:::

::: {.Theorem info="two  spaces, trailing "}
Spaces inside as pandoc.Inlines makes them of a string, at the end dropped.
:::

::: {.Theorem info="1. Not a list"}
What markdown would read as a list is text.
:::

::: {.Theorem info="the *last* one"}
Markup: shown as typed, with the one warning.
:::

::: {.proof info="Proof of $x$"}
A proof's note as typed, in italics.
:::

::: {.Theorem info="100% & #1_x [see] {a} \ref{b}"}
Characters LaTeX escapes, brackets among them: text, not braced.
:::

::: {.Theorem info=""}
An empty attribute: no note.
:::

::: {.proof info=""}
An empty attribute on a proof: no note.
:::
