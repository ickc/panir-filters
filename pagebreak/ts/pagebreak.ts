#!/usr/bin/env node
// LaTeX page breaks to the output format's: a pandoc filter.
//
// A raw TeX block holding only `\newpage` or `\pagebreak` becomes the
// format's page break (a raw block, or a paragraph of a form feed in
// formats without one); with `pagebreak.break-on.form-feed`, so does a
// paragraph of a form feed alone, and with `break-on.plaintext-command`, a
// paragraph of the command as text. `html-class` and `odt-style` (or
// `$PANDOC_PAGEBREAK_HTML_CLASS`, `$PANDOC_PAGEBREAK_ODT_STYLE`) style the
// HTML and ODT ones; the configuration is removed.
//
// A port of pandoc-ext/pagebreak (Copyright © 2017-2025 Benct Philip
// Jonsson, Albert Krewinkel; ISC; the repository's MIT licence, see
// ../LICENSE).

import { applyFilter, copy, lookup, Para, RawBlock, Str, stringify, truthy, type Block, type Filter, type Meta, type MetaValue } from "panir";
import { runFilter } from "panir/node";

const NEWPAGE = /^\\(?:newpage|pagebreak)\{?\}?$/;

/** A text option: the metadata's, else the environment's. (The top-level
 * field: the original reads `html-class`, not `pagebreak.html-class` as its
 * README says.) */
const option = (meta: Meta, key: string, variable: string): string | undefined =>
  truthy(meta[key]) ? stringify(meta[key]!) : process.env[variable];

/** A page break in the format. */
function pagebreak(format: string, meta: Meta): Block {
  if (format.includes("asciidoc")) return RawBlock("asciidoc", "<<<\n\n");
  if (format === "context") return RawBlock("context", "\\page");
  if (format === "docx") return RawBlock("openxml", '<w:p><w:r><w:br w:type="page"/></w:r></w:p>');
  if (format.includes("epub")) return RawBlock("html", '<p style="page-break-after: always;"> </p>');
  if (format.includes("html")) {
    const htmlClass = option(meta, "html-class", "PANDOC_PAGEBREAK_HTML_CLASS");
    return RawBlock("html", htmlClass ? `<div class="${htmlClass}"></div>` : '<div style="page-break-after: always;"></div>');
  }
  if (format.includes("latex")) return RawBlock("tex", "\\newpage{}");
  if (format.includes("ms")) return RawBlock("ms", ".bp");
  if (format.includes("odt")) {
    const style = option(meta, "odt-style", "PANDOC_PAGEBREAK_ODT_STYLE") || "Pagebreak";
    return RawBlock("opendocument", `<text:p text:style-name="${style}"/>`);
  }
  if (format.includes("typst")) return RawBlock("typst", "#pagebreak()\n\n");
  return Para([Str("\f")]); // a form feed, in any other format
}

await runFilter({
  Pandoc: (doc, ctx) => {
    const format = ctx.format ?? "";
    const config = doc.meta.pagebreak;
    delete doc.meta.pagebreak;
    const isTrue = (v: MetaValue | undefined) => v?.t === "MetaBool" && v.value;
    if (isTrue(config) || isTrue(lookup(config, "break-on"))) throw new Error("pagebreak: a configuration of true"); // (Lua indexes a boolean)
    const raw = pagebreak(format, doc.meta);
    const checks: ((text: string) => boolean)[] = [];
    if (truthy(lookup(config, "break-on", "form-feed"))) checks.push((text) => text === "\f");
    if (truthy(lookup(config, "break-on", "plaintext-command"))) checks.push((text) => NEWPAGE.test(text));

    const breaks: Filter = {};
    // (in LaTeX, raw TeX stays as it is)
    if (!format.endsWith("tex")) {
      breaks.RawBlock = (block) => (block.format.includes("tex") && NEWPAGE.test(block.text) ? copy(raw) : undefined);
    }
    if (checks.length > 0) {
      breaks.Para = (para) => {
        const [first] = para.content;
        const alone = para.content.length === 1 && first?.t === "Str";
        return alone && checks.some((check) => check(first.text)) ? copy(raw) : undefined;
      };
    }
    applyFilter(doc, breaks); // the metadata's blocks too
  },
});
