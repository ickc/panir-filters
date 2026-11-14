//! GitLab's math to pandoc's: a pandoc filter. A code block whose first
//! class is `math` becomes display math; inline math loses the backticks
//! around it (GitLab writes $`x`$).
//!
//! A port of pandocfilters' examples/gitlab_markdown.py (Copyright (c)
//! 2013, John MacFarlane; BSD-3-Clause, see ../LICENSE).

use panir::{Block, Ctx, Filter, Inline, Math, MathType};

struct GitlabMarkdown;

impl Filter for GitlabMarkdown {
    fn block(&mut self, x: &mut Block, _: &mut Ctx) -> Option<Vec<Block>> {
        let Block::CodeBlock(code) = x else {
            return None;
        };
        if code.attr.classes.first().is_none_or(|c| c != "math") {
            return None;
        }
        let math = Math {
            math_type: MathType::DisplayMath,
            text: std::mem::take(&mut code.text),
        };
        Some(vec![Block::Para(vec![math.into()])])
    }

    fn inline(&mut self, x: &mut Inline, _: &mut Ctx) -> Option<Vec<Inline>> {
        if let Inline::Math(math) = x
            && math.math_type == MathType::InlineMath
        {
            math.text = math.text.trim_matches('`').into();
        }
        None
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut GitlabMarkdown, format));
}
