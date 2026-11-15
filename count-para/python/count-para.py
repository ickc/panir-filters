#!/usr/bin/env python3
"""Numbered paragraphs, and citations of them: a pandoc filter.

Each paragraph of the body (not one opening with an image) gets a number,
in the margin (LaTeX) or in a span before it, per chapter with
`resetAtChapter`. A paragraph opening with `{#name}` can be cited as
`[@name]`: the citation becomes a link to its number. `enclosing`,
`chapterSep`, `refName` and `addPageNr` shape the numbers and the links;
`header-includes` gets the CSS or the LaTeX packages.

A port of cysouw/count-para (Copyright © 2021, 2023 Michael Cysouw; ISC;
the repository's CC0-1.0, see ../LICENSE).
"""

import re
from dataclasses import dataclass, field

from panir import (
    Cite,
    Context,
    Filter,
    Header,
    Image,
    Inline,
    Link,
    Meta,
    MetaBlocks,
    MetaList,
    MetaMap,
    MetaValue,
    Pandoc,
    Para,
    RawBlock,
    RawInline,
    Space,
    Span,
    main,
    stringify,
)

# (as the original's, its spaces at the ends of lines too)
CSS = """\x20
<!-- CSS added by lua-filter 'count-para' -->
<style>
.paragraph-number {\x20
  float: left;
  margin-left: -5em;
  width: 4.5em;
  text-align: right;
  color: grey;
  font-size: x-small;
  padding-top: 5px;
}
</style>
    """

LATEX = [
    "\\usepackage{xcolor}",
    "\\usepackage{marginnote}",
    "\\reversemarginpar",
    "\\newcommand{\\paragraphnumber}[1]{\\marginnote{\\color{lightgray}\\tiny{#1}}[0pt]}",
]


@dataclass
class State:
    # the options
    reset_at_chapter: bool = False
    enclosing: str = "[]"
    chapter_sep: str = "."
    ref_name: str = "paragraph "
    add_page_nr: bool = True
    # the numbering
    count: int = 0
    chapter: int = 0
    numbers: dict[str, str] = field(default_factory=dict)  # by the names given


state = State()


def byte(text: str, i: int) -> str:
    """The text's `i`th byte, as Lua's `sub` takes it and pandoc reads it
    back: an ASCII character, else U+FFFD (or nothing past the end)."""
    return text.encode()[i : i + 1].decode(errors="replace")


formatting, settings, numbering, references = Filter(), Filter(), Filter(), Filter()


@formatting.on_meta
def add_formatting(meta: Meta, ctx: Context) -> None:
    fmt = ctx.format or ""
    new: list[MetaValue] = []
    if "html" in fmt:
        new.append(MetaBlocks(RawBlock("html", CSS)))
    if "latex" in fmt:
        new += [MetaBlocks(RawBlock("tex", tex)) for tex in LATEX]
    # The original appends to what is there as to a Lua table: a list grows,
    # a map's entries are lost (the table becomes a list of the new ones),
    # anything else fails; nothing there is an empty list, even with nothing
    # to add.
    match meta.get("header-includes"):
        case None:
            meta["header-includes"] = new
        case MetaList(values):
            values.extend(new)
        case MetaMap() if new:
            meta["header-includes"] = new
        case other if new:
            raise ValueError(f"count-para: header-includes is {type(other).__name__}, not a list")


@settings.on_meta
def read_settings(meta: Meta, ctx: Context) -> None:
    if (value := meta.get("resetAtChapter")) is not None:
        state.reset_at_chapter = bool(value)  # (Lua's truth: set and not false)
    if (value := meta.get("enclosing")) is not None:
        state.enclosing = stringify(value)
    if (value := meta.get("chapterSep")) is not None:
        state.chapter_sep = stringify(value)
    if (value := meta.get("refName")) is not None:
        state.ref_name = stringify(value)
        if "latex" in (ctx.format or "") and state.ref_name == "#":
            state.ref_name = "\\#"
    if (value := meta.get("addPageNr")) is not None:
        state.add_page_nr = bool(value)


@numbering.on(Pandoc)
def count_paragraphs(doc: Pandoc, ctx: Context) -> None:
    latex = "latex" in (ctx.format or "")
    for block in doc.blocks:  # (the body's own paragraphs only)
        if isinstance(block, Header) and block.level == 1 and "unnumbered" not in block.classes[:1]:
            if state.reset_at_chapter:
                state.chapter += 1
                state.count = 0
        # (an empty paragraph fails, as in the original: it has no first inline)
        if not isinstance(block, Para) or isinstance(block.content[0], Image):
            continue
        state.count += 1
        number = (
            f"{state.chapter}{state.chapter_sep}{state.count}"
            if state.reset_at_chapter
            else str(state.count)
        )
        e = state.enclosing
        shown = f"{e}{number}{e}" if len(e.encode()) == 1 else f"{byte(e, 0)}{number}{byte(e, 1)}"

        # a name given at the start: anywhere in the first inline's text
        name = None
        if found := re.search(r"\{#(.*)\}", stringify(block.content[0]), re.DOTALL):
            name = found[1]
            state.numbers[name] = number
            del block.content[0]
            if isinstance(block.content[0], Space):  # (nothing left fails, as in the original)
                del block.content[0]

        if latex:  # after the first word, to keep the number on its page
            tex = f"\\paragraphnumber{{{shown}}}"
            if name is not None:
                tex = f"\\hypertarget{{{name}}}{{\n{tex}\\label{{{name}}}}}"
            block.content.insert(1, RawInline("tex", tex))
        else:
            block.content[0:0] = [
                Span(shown, identifier=number, classes=["paragraph-number"]),
                Space(),
            ]


@references.on(Cite)
def cross_reference(cite: Cite, ctx: Context) -> Inline | None:
    name = cite.citations[0].id
    number = state.numbers.get(name)
    if number is None:
        return None
    if "latex" in (ctx.format or ""):
        tex = f"{state.ref_name}\\hyperlink{{{name}}}{{{number}}}"
        if state.add_page_nr:
            tex += f" on page~\\pageref{{{name}}}"
        return RawInline("tex", tex)
    return Link(state.ref_name + number, url=f"#{number}")


if __name__ == "__main__":
    # header-includes, the options, the numbers, then the citations (the
    # metadata's too)
    main([formatting, settings, numbering, references])
