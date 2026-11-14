#!/usr/bin/env python3
"""GitLab's math to pandoc's: a pandoc filter. A code block whose first
class is `math` becomes display math; inline math loses the backticks
around it (GitLab writes $`x`$).

A port of pandocfilters' examples/gitlab_markdown.py (Copyright (c) 2013,
John MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

from panir import CodeBlock, Filter, Math, MathType, Para

f = Filter()


@f.on(CodeBlock)
def display_math(code: CodeBlock) -> Para | None:
    if code.classes[:1] == ["math"]:
        return Para(Math(MathType.DisplayMath, code.text))
    return None


@f.on(Math)
def inline_math(math: Math) -> None:
    if math.math_type == MathType.InlineMath:
        math.text = math.text.strip("`")


if __name__ == "__main__":
    f.main()
