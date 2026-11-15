---
amsthm:
  plain:
    - Foo\*: [Bar, Baz]
    - Prop: {a: b}
    - A B
    - A_B
    - proof
    - true
  definition:
    - Name\*
  name_to_text:
    Name\*: The Name
    Bar: Barre
    proof: ""
---

::: Bar
Shares the counter of Foo, which is unnumbered.
:::

::: Baz
Again.
:::

::: Prop
The value of a map: its text.
:::

::: A_B
Two environments for one class: the later.
:::

::: proof
The proof, named "proof".
:::

::: true
A theorem named true.
:::

::: {#n .Name}
Named by name_to_text, by its name with the star.
:::

See @n and [@N].
