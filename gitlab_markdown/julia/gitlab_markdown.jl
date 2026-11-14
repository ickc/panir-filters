#!/usr/bin/env julia
# GitLab's math to pandoc's: a pandoc filter. A code block whose first
# class is `math` becomes display math; inline math loses the backticks
# around it (GitLab writes $`x`$).
#
# A port of pandocfilters' examples/gitlab_markdown.py (Copyright (c) 2013,
# John MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

function gitlab(code::CodeBlock)
    classes = code.attr.classes
    !isempty(classes) && classes[1] == "math" ? Para(Math(DisplayMath, code.text)) : nothing
end

function gitlab(math::Math)
    math.math_type == InlineMath && (math.text = strip(math.text, '`'))
    nothing
end

run_filter(gitlab)
