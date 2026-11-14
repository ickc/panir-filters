#!/usr/bin/env python3
"""Affiliations and correspondence as blocks before the body: a pandoc
filter, run after scholarly-metadata (whose `author` and `institute` it
reads).

The body starts with a note on equal contributors (if any), the
affiliations, numbered, and the corresponding authors (`correspondence`
and an `email`), linked; `author` becomes one line of names with their
marks (in LaTeX, a list, one each), and `institute` goes.

A port of pandoc/lua-filters' author-info-blocks (Copyright © 2017–2021
Albert Krewinkel; ISC; the repository's MIT licence, see ../LICENSE).
"""

import copy
from collections.abc import Callable, Sequence
from typing import TypeGuard

from panir import (
    Context,
    Filter,
    Inline,
    LineBreak,
    Link,
    MetaBool,
    MetaInlines,
    MetaList,
    MetaMap,
    MetaValue,
    Pandoc,
    Para,
    RawInline,
    Space,
    Str,
    Superscript,
    stringify,
)

Map = dict[str, MetaValue]

f = Filter()


def given(value: MetaValue | None) -> TypeGuard[MetaValue]:
    """Whether Lua counts the value: anything but missing or false."""
    return value is not None and value != MetaBool(False)


def items(value: MetaValue | None) -> list[MetaValue]:
    if not isinstance(value, MetaList):
        raise ValueError(f"author-info-blocks: expected a list, got {type(value).__name__}")
    return value.content


def maps(value: MetaValue | None) -> list[Map]:
    """`author` or `institute`, as scholarly-metadata leaves them."""
    out = []
    for x in items(value):
        if not isinstance(x, MetaMap):
            raise ValueError(f"author-info-blocks: expected a map, got {type(x).__name__}")
        out.append(x.content)
    return out


def inlines(value: MetaValue) -> list[Inline]:
    """A name or an email: inlines (the original fails on anything else)."""
    if not isinstance(value, MetaInlines):
        raise ValueError(f"author-info-blocks: expected inlines, got {type(value).__name__}")
    return value.content


def intercalate(lists: Sequence[Sequence[Inline]], sep: Callable[[], list[Inline]]) -> list[Inline]:
    out: list[Inline] = []
    for i, xs in enumerate(lists):
        if i:
            out += sep()  # a new separator each time: a node is in one place
        out += xs
    return out


def corresponding(author: Map) -> bool:
    return given(author.get("correspondence")) and given(author.get("email"))


@f.on(Pandoc)
def info_blocks(doc: Pandoc, ctx: Context) -> None:
    def mark(name: str) -> list[Inline]:
        latex = ctx.format == "latex"
        if name == "corresponding_author":
            return [RawInline("latex", "*")] if latex else [Str("✉")]
        return [RawInline("latex", "$\\dagger{}$")] if latex else [Str("*")]

    def author_line(author: Map) -> list[Inline]:
        """The author's name, with its marks as a superscript."""
        marks = [mark("equal_contributor")] if given(author.get("equal_contributor")) else []
        marks += [[Str(stringify(i))] for i in items(author.get("institute"))]
        if corresponding(author):
            marks.append(mark("corresponding_author"))
        return [*inlines(author["name"]), Superscript(*intercalate(marks, lambda: [Str(",")]))]

    authors = maps(doc.meta.get("author"))
    institute = doc.meta.pop("institute", None)
    institutes = maps(institute) if institute is not None else []

    body = []
    if any(given(a.get("equal_contributor")) for a in authors):
        body.append(
            Para(
                Superscript(*mark("equal_contributor")),
                Space(),
                Str("These authors contributed equally to this work."),
            )
        )
    lines = [
        [Superscript(Str(str(i))), Space(), *inlines(inst["name"])]
        for i, inst in enumerate(institutes, 1)
    ]
    body.append(Para(*intercalate(lines, lambda: [LineBreak()])))
    # the name is in the author line too: a copy here
    links = [
        [
            Link(
                *copy.deepcopy(inlines(a["name"])),
                Space(),
                Str("<"),
                *inlines(a["email"]),
                Str(">"),
                url="mailto:" + stringify(a["email"]),
            )
        ]
        for a in authors
        if corresponding(a)
    ]
    if links:
        body.append(
            Para(
                Superscript(*mark("corresponding_author")),
                Space(),
                Str("Correspondence:"),
                Space(),
                *intercalate(links, lambda: [Str(","), Space()]),
            )
        )
    doc.blocks = [*body, *doc.blocks]

    names = [author_line(a) for a in authors]
    if "latex" in (ctx.format or ""):  # (the marks above ask for "latex" exactly)
        doc.meta["author"] = MetaList(*(MetaInlines(*name) for name in names))
        return
    if not names:
        raise ValueError("author-info-blocks: no authors")  # (the original fails too)
    line = intercalate(names[:-1], lambda: [Str(","), Space()])
    if len(names) > 1:
        line += [Str(","), Space(), Str("and"), Space()]
    doc.meta["author"] = MetaInlines(*line, *names[-1])


if __name__ == "__main__":
    f.main()
