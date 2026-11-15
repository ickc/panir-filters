#!/usr/bin/env python3
"""LaTeX page breaks to the output format's: a pandoc filter.

A raw TeX block holding only `\\newpage` or `\\pagebreak` becomes the
format's page break (a raw block, or a paragraph of a form feed in
formats without one); with `pagebreak.break-on.form-feed`, so does a
paragraph of a form feed alone, and with `break-on.plaintext-command`, a
paragraph of the command as text. `html-class` and `odt-style` (or
`$PANDOC_PAGEBREAK_HTML_CLASS`, `$PANDOC_PAGEBREAK_ODT_STYLE`) style the
HTML and ODT ones; the configuration is removed.

A port of pandoc-ext/pagebreak (Copyright © 2017-2025 Benct Philip
Jonsson, Albert Krewinkel; ISC; the repository's MIT licence, see
../LICENSE).
"""

import copy
import os
import re
from collections.abc import Callable

from panir import (
    Block,
    Context,
    Filter,
    Meta,
    MetaBool,
    Pandoc,
    Para,
    RawBlock,
    Str,
    lookup,
    stringify,
)

NEWPAGE = re.compile(r"\\(?:newpage|pagebreak)\{?\}?")


def option(meta: Meta, key: str, variable: str) -> str | None:
    """A text option: the metadata's, else the environment's. (The
    top-level field: the original reads `html-class`, not
    `pagebreak.html-class` as its README says.)"""
    value = meta.get(key)
    return stringify(value) if value else os.environ.get(variable)


def pagebreak(fmt: str, meta: Meta) -> Block:
    """A page break in the format."""
    if "asciidoc" in fmt:
        return RawBlock("asciidoc", "<<<\n\n")
    if fmt == "context":
        return RawBlock("context", "\\page")
    if fmt == "docx":
        return RawBlock("openxml", '<w:p><w:r><w:br w:type="page"/></w:r></w:p>')
    if "epub" in fmt:
        return RawBlock("html", '<p style="page-break-after: always;"> </p>')
    if "html" in fmt:
        if html_class := option(meta, "html-class", "PANDOC_PAGEBREAK_HTML_CLASS"):
            return RawBlock("html", f'<div class="{html_class}"></div>')
        return RawBlock("html", '<div style="page-break-after: always;"></div>')
    if "latex" in fmt:
        return RawBlock("tex", "\\newpage{}")
    if "ms" in fmt:
        return RawBlock("ms", ".bp")
    if "odt" in fmt:
        style = option(meta, "odt-style", "PANDOC_PAGEBREAK_ODT_STYLE") or "Pagebreak"
        return RawBlock("opendocument", f'<text:p text:style-name="{style}"/>')
    if "typst" in fmt:
        return RawBlock("typst", "#pagebreak()\n\n")
    return Para(Str("\f"))  # a form feed, in any other format


def alone(para: Para) -> str | None:
    """The text of a paragraph of one `Str`."""
    match para.content:
        case [Str(text)]:
            return text
    return None


f = Filter()


@f.on(Pandoc)
def page_breaks(doc: Pandoc, ctx: Context) -> None:
    fmt = ctx.format or ""
    config = doc.meta.pop("pagebreak", None)
    if config == MetaBool(True) or lookup(config, "break-on") == MetaBool(True):
        raise ValueError("pagebreak: a configuration of true")  # (Lua indexes a boolean)
    raw = pagebreak(fmt, doc.meta)
    checks: list[Callable[[str], object]] = []
    if lookup(config, "break-on", "form-feed"):
        checks.append(lambda text: text == "\f")
    if lookup(config, "break-on", "plaintext-command"):
        checks.append(NEWPAGE.fullmatch)

    breaks = Filter()

    if not fmt.endswith("tex"):  # (in LaTeX, raw TeX stays as it is)

        @breaks.on(RawBlock)
        def raw_tex(block: RawBlock) -> Block | None:
            if "tex" in block.format and NEWPAGE.fullmatch(block.text):
                return copy.deepcopy(raw)  # (one page break, in many places)
            return None

    if checks:

        @breaks.on(Para)
        def paragraph(para: Para) -> Block | None:
            text = alone(para)
            if text is not None and any(check(text) for check in checks):
                return copy.deepcopy(raw)
            return None

    doc.walk(breaks)  # the metadata's blocks too


if __name__ == "__main__":
    f.main()
