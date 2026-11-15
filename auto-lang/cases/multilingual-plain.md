---
auto-lang:
  Greek: el
  Han: zh-Hant
  Hebrew: he
format:
  html:
    code-copy: true
    code-overflow: wrap
    css: assets/fonts.css
    email-obfuscation: javascript
    format-links:
    - href: multilingual-mathjax.html
      icon: filetype-html
      text: MathJax 4 HTML
    - pdf
    - typst
    html-math-method: mathml
    output-file: multilingual.html
    respect-user-color-scheme: true
    theme:
      dark: darkly
      light: flatly
  mathjax4-html:
    code-copy: true
    code-overflow: wrap
    css: assets/fonts.css
    email-obfuscation: javascript
    format-links:
    - href: multilingual.html
      icon: filetype-html
      text: MathML HTML
    - pdf
    - typst
    output-file: multilingual-mathjax.html
    respect-user-color-scheme: true
    theme:
      dark: darkly
      light: flatly
  pdf:
    babelfonts:
      chinese-hans: NotoSansCJKtc
      chinese-hant: NotoSansCJKtc
      greek: Gentium
      hebrew: Ezra SIL
    filters:
    - ../config/absolute-links.lua
    latex-auto-install: false
    latex-tinytex: false
    link-base: "https://font.kolen.dev"
    mainfont: TeX Gyre Schola
    mathfont: TeX Gyre Schola Math
    monofont: JetBrains Mono
    output-file: multilingual-lualatex.pdf
    pdf-engine: lualatex
  typst:
    codefont: JetBrains Mono
    filters:
    - ../config/absolute-links.lua
    include-in-header: ../config/fonts.typ
    link-base: "https://font.kolen.dev"
    mainfont: TeX Gyre Schola
    mathfont: TeX Gyre Schola Math
    output-file: multilingual-typst.pdf
lang: en
title: Multilingual font sample
toc: true
toc-depth: 3
---



# Text samples

Every span in this section was written into this file by
`pixi run format`, from lines that were typed as plain text; the one in
the next section is the hand-written exception, and is there to show
what the formatter leaves alone. Nothing runs at render time: what the
four recipes read is the ordinary Pandoc markup checked in here, which
is also what a reviewer reads in the diff.

וַיֹּ֥אמֶר אֱלֹהִים יְהִ֣י א֑וֹר וַֽיְהִי־אֽוֹר׃

καὶ εἶπεν ὁ θεός Γενηθήτω φῶς. καὶ ἐγένετο φῶς.

And God said, Let there be light: and there was light.

神說：「要有光」，就有了光。

Runs inside a sentence are found the same way: 神說 was rendered καὶ
εἶπεν ὁ θεός by the Septuagint, from the Hebrew וַיֹּ֥אמֶר אֱלֹהִים.

# Overriding the map

A script carries no more than a script: Han is written by Traditional
Chinese, Simplified Chinese and Japanese alike, and no rule can tell
them apart. Where the language is not the one the map names, write it
out. The formatter leaves a span that already carries a `lang` --- and
everything inside it --- alone:

神说："要有光。"就有了光。

That is the fourth line above again, written the way Simplified Chinese
writes it: one Han character differs, and the rest is punctuation
convention. No rule over the script can see either, so left to the map
this line would have come out labelled zh-Hant --- the right script, the
wrong language.

Nothing else in the file marks the span as different from the ones the
formatter wrote, and nothing needs to: once formatting has run, its
output is markup like any other, editable in place. Noto Sans CJK TC
carries the glyphs either way, so the override changes what the document
says about the text rather than which face draws it.
