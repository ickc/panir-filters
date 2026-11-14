#!/usr/bin/env node
// French punctuation's spaces for LaTeX: a pandoc filter. In LaTeX and
// beamer output, a space after `«`, `“`, `‹` or before `;`, `?`, `!`, `»`,
// `”`, `›` becomes a thin space (`\thinspace{}`), and one before `:` an
// unbreakable space (`~`).
//
// A port of chdemko/pandoc-latex-french-spaces (Copyright (c) 2018-2024,
// Christophe Demko; BSD-3-Clause, see ../LICENSE).

import { RawInline } from "panir";
import { runFilter } from "panir/node";

const thin = () => RawInline("tex", "\\thinspace{}");

await runFilter({
  Space: (_space, ctx) => {
    if (ctx.format !== "latex" && ctx.format !== "beamer") return;
    const { prev, next } = ctx;
    if (prev?.t === "Str" && /[«“‹]$/.test(prev.text)) return thin();
    if (next?.t === "Str") {
      if (next.text.startsWith(":")) return RawInline("tex", "~");
      if (/^[;?!»”›]/.test(next.text)) return thin();
    }
  },
});
