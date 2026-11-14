#!/usr/bin/env python3
"""RST-like list tables in Markdown: a pandoc filter.

A div of class `list-table` (or `list-table-body`, without a head)
holding a bullet list of bullet lists becomes a table: each item a row,
each item of an item a cell. A paragraph before the list is the caption.
An empty span opening a cell gives the cell its attributes (`colspan`,
`rowspan`, `align`), one alone before a row's list gives the row its own;
a table in an item starts a new body with its rows. The div's attributes
`header-rows`, `header-cols`, `aligns` (`l,c,r,d`) and `widths` shape the
table.

A port of pandoc-ext/list-table (Copyright (c) 2021 Martin Fischer,
2023-2024 Pandoc Extensions contributors; MIT, see ../LICENSE).
"""

import re

from panir import (
    Alignment,
    Attr,
    Block,
    BulletList,
    Caption,
    Cell,
    ColSpec,
    ColWidth,
    Div,
    Filter,
    Para,
    Plain,
    Row,
    Span,
    Table,
    TableBody,
    TableFoot,
    TableHead,
)

ALIGNMENTS = {
    "d": Alignment.AlignDefault,
    "l": Alignment.AlignLeft,
    "r": Alignment.AlignRight,
    "c": Alignment.AlignCenter,
}

f = Filter()


class ListTableError(ValueError):
    """The original also shows the offending block as markdown: that needs
    pandoc's writer (libpandoc), so here only its kind is named."""


def number(text: str | None) -> float | None:
    """A number as Lua's `tonumber` reads one (decimal only), else None."""
    if text is not None and re.fullmatch(r"\s*[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?\s*", text):
        return float(text)
    return None


def skip_data_pos(block: Block) -> Block:
    """The block itself, or the first in a div the sourcepos extension
    wrapped it in."""
    if isinstance(block, Div) and "data-pos" in block.attributes:
        return block.content[0]
    return block


def pop(attr: Attr, key: str) -> str | None:
    """Remove an attribute, returning its value."""
    value = attr.attributes.get(key)
    if key in attr.attributes:
        del attr.attributes[key]
    return value


def new_cell(blocks: list[Block]) -> Cell:
    blocks = [skip_data_pos(b) for b in blocks]
    attr = Attr()
    # pandoc has no attributes on list items: an empty span opening the
    # first block holds the cell's
    first = getattr(blocks[0], "content", None) if blocks else None
    if first and isinstance(first[0], Span) and not first[0].content:
        attr = first.pop(0).attr
    col_span = pop(attr, "colspan") or 1
    row_span = pop(attr, "rowspan") or 1
    align = ALIGNMENTS.get(pop(attr, "align") or "", Alignment.AlignDefault)
    return Cell(blocks, align, int(row_span), int(col_span), attr=attr)


def row_attr(block: Block) -> Attr:
    """The attributes of a row: a block holding one empty span."""
    content = getattr(block, "content", None)
    if content is None:
        raise ListTableError("expected list item to have row attrs")
    if len(content) != 1:
        raise ListTableError("expected row attrs to contain only one inline")
    if not isinstance(content[0], Span):
        raise ListTableError("expected row attrs to contain a span")
    if content[0].content:
        raise ListTableError("expected row attrs span to be empty")
    return content[0].attr


def colspecs(attr: Attr, columns: int) -> list[ColSpec]:
    specs = [ColSpec() for _ in range(columns)]
    aligns = pop(attr, "aligns")
    if aligns is not None:
        given = [a for a in aligns.split(",") if a]
        if unknown := [a for a in given if a not in ALIGNMENTS]:
            raise ListTableError(f"unknown column alignment {unknown[0]}")
        if len(given) > columns:
            raise ListTableError("more alignments than columns")
        for spec, a in zip(specs, given):
            spec.alignment = ALIGNMENTS[a]
    widths = pop(attr, "widths")
    if widths is not None:
        given = [w for w in map(number, widths.split(",")) if w is not None]
        if len(given) != len([w for w in widths.split(",") if w]) or len(given) < columns:
            raise ListTableError(f"invalid widths: {widths}")
        total = sum(given)
        for spec, w in zip(specs, given):
            spec.width = ColWidth(w / total)
    return specs


@f.on(Div)
def list_table(div: Div) -> Table | None:
    kind = div.classes[0] if div.classes else None
    if kind not in ("list-table", "list-table-body"):
        return None
    content = [skip_data_pos(b) for b in div.content]
    caption = []
    if content and isinstance(content[0], Para):
        caption = [Plain(*content.pop(0).content)]
    if not content:
        return None  # an empty div, or a caption alone, stays as it is
    match content[0]:
        case BulletList(items):
            pass
        case other:
            raise ListTableError(f"expected bullet list, found {type(other).__name__}")

    bodies = [TableBody([])]
    for item in items:
        attr = Attr()
        if len(item) > 1:
            attr = row_attr(skip_data_pos(item[0]))
            item = item[1:]
        if len(item) != 1:
            raise ListTableError("expected item to contain only one block")
        match skip_data_pos(item[0]):
            case Table() as table:
                # its rows are a body of their own
                if table.caption.content or table.caption.short:
                    raise ListTableError("table bodies can't have captions (they'd be ignored)")
                if len(table.bodies) != 1:
                    raise ListTableError("table bodies can't contain other table bodies")
                if bodies[-1].body:
                    bodies.append(TableBody([]))
                bodies[-1].attr = table.attr
                bodies[-1].body.extend(table.bodies[0].body)
            case BulletList(cells):
                bodies[-1].body.append(Row(*map(new_cell, cells), attr=attr))
            case other:
                raise ListTableError(f"expected bullet list, found {type(other).__name__}")

    del div.classes[0]
    rows = bodies[0].body
    header_rows = number(pop(div.attr, "header-rows"))
    if header_rows is None:
        header_rows = 1 if kind == "list-table" else 0
    header_cols = number(pop(div.attr, "header-cols")) or 0
    columns = sum(cell.col_span for cell in rows[0].cells) if rows else 0
    specs = colspecs(div.attr, columns)
    head = rows[: max(0, int(header_rows))]
    del rows[: len(head)]
    for body in bodies:
        body.row_head_columns = int(header_cols)
        header_cols = 0  # the first body's only
    return Table(
        Caption(*caption, short=[]),
        specs,
        TableHead(*head),
        [body for body in bodies if body.body],
        TableFoot(),
        attr=div.attr,
    )


if __name__ == "__main__":
    f.main()
