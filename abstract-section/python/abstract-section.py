#!/usr/bin/env python3
"""A section headed "Abstract" moves into the metadata: a pandoc filter.

A section whose heading has the identifier `abstract`, at the highest
level met so far, in any list of blocks, runs to the next such heading or
a horizontal rule; it becomes the metadata's `abstract`, unless that is
set. `abstract-section.section-identifiers` lists other identifiers (each
its own field); the configuration is removed.

A port of pandoc-ext/abstract-section (Copyright © 2017-2023 Albert
Krewinkel; MIT, see ../LICENSE).
"""

from panir import (
    Block,
    Filter,
    Header,
    HorizontalRule,
    Meta,
    MetaBlocks,
    MetaBool,
    MetaInlines,
    MetaList,
    MetaString,
    Pandoc,
    lookup,
    stringify,
)

f = Filter()


def section_identifiers(meta: Meta) -> set[str]:
    """`abstract-section.section-identifiers`, as the original reads it."""
    config = meta.pop("abstract-section", None)
    if config == MetaBool(True):  # (a string, a list... has no field in Lua; true fails)
        raise ValueError("abstract-section: the configuration is true, not a map")
    match lookup(config, "section-identifiers"):
        case MetaList(items) | MetaInlines(items) | MetaBlocks(items) if items:
            return {stringify(x) for x in items}  # (each inline of text too)
        case MetaString(text) if text:
            return set()  # (Lua takes its length, then finds no items)
        case MetaBool(True):
            raise ValueError("abstract-section: section-identifiers is true, not a list")
    return {"abstract"}


@f.on(Pandoc)
def abstract_section(doc: Pandoc) -> None:
    identifiers = section_identifiers(doc.meta)
    collected: dict[str, list[Block]] = {}
    toplevel = 6  # the highest level of heading met so far

    sections = Filter()

    @sections.on_blocks
    def collect(blocks: list[Block]) -> list[Block]:
        nonlocal toplevel
        body = []
        section = None  # the identifier of the section being collected
        for block in blocks:
            if isinstance(block, Header) and block.level <= toplevel:
                toplevel = block.level
                if block.identifier in identifiers:
                    section = block.identifier
                    collected[section] = []
                else:
                    section = None
                    body.append(block)
            elif section is None:
                body.append(block)
            elif isinstance(block, HorizontalRule):
                section = None
            else:
                collected[section].append(block)
        return body

    doc.blocks.walk(sections)  # the body's lists, inner ones first
    for key, blocks in collected.items():
        if key in identifiers and blocks and not doc.meta.get(key):
            doc.meta[key] = MetaBlocks(*blocks)


if __name__ == "__main__":
    f.main()
