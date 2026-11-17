#!/usr/bin/env python3
"""Code blocks of class `table`, CSV with YAML options, made tables: a
pandoc filter.

The block holds CSV, and YAML blocks (each between `---` and `---` or
`...`) of options: `caption`, `short-caption`, `alignment` (a letter a
column among L, R, C, D), `alignment-cells` (a line of letters a row),
`width` (a number or fraction a column, or D), `table-width` (then the
columns without a width get one from the length of their text),
`ms` (rows of the head, of each body's head and rows, of the foot),
`ns-head` (row head columns of each body), `markdown` (cells, captions
and attributes read as markdown), `fancy-table` (with markdown: the first
column marks the head, bodies and foot, with attributes; a cell's first
line its span and attributes), `include` (the CSV from a file, relative to
the working directory, `include-encoding` its encoding), `csv-kwargs`
(Python's csv.reader's), `format` (csv only).

Reading the CSV and the YAML is Python's (its csv module, PyYAML, as the
original); the markdown of the cells is pandoc's, read with `libpandoc`:
the pandoc the filter runs in under pandocpy or `libpandoc.convert`, or
libpandoc as a library when pandoc runs this as a JSON filter, all of a
table's texts in one call (`read_many`). Without libpandoc a table with
markdown fails, saying so; the others don't need it.

A port of ickc/pantable's `pantable` filter, 0.15.0 (© 2016-2026 Kolen
Cheung; BSD-3-Clause, see ../LICENSE).
"""

import csv
import io
import re
import sys
from dataclasses import dataclass, field, fields
from fractions import Fraction
from itertools import accumulate
from typing import Literal

import yaml
from yaml.scanner import ScannerError

from panir import (
    Attr,
    Block,
    Caption,
    Cell,
    CodeBlock,
    ColSpec,
    ColWidth,
    Context,
    Filter,
    Para,
    Plain,
    Row,
    Span,
    Str,
    Table,
    TableBody,
    TableFoot,
    TableHead,
    main,
)

try:
    import libpandoc  # pandoc's reader, for markdown cells
except ImportError:
    libpandoc = None

# panflute's choice of PyYAML's loader: libyaml's where it is built
Loader = getattr(yaml, "CSafeLoader", yaml.SafeLoader)
Align = Literal["AlignDefault", "AlignLeft", "AlignRight", "AlignCenter"]
ALIGN: tuple[Align, ...] = ("AlignDefault", "AlignLeft", "AlignRight", "AlignCenter")


def warn(message: str) -> None:
    print(f"pantable: {message}", file=sys.stderr)


class EmptyTable(Exception):
    pass


# The code block: YAML and data, as panflute's yaml_filter (strict_yaml)
# splits it: at lines of three or more dots or dashes; a chunk starting
# with --- opens YAML (even one of data), --- or ... closes it.

SEPARATOR = re.compile(r"^([.]{3,}|[-]{3,})$", re.MULTILINE)


def split(text: str) -> tuple[dict, str] | None:
    """The options and the data; None for YAML that can't be scanned."""
    options: dict = {}
    data = []
    raw = True
    for chunk in SEPARATOR.split(text):
        chunk = chunk.strip("\n")
        if not chunk:
            continue
        if raw:
            if chunk.startswith("---"):
                raw = False
            else:
                data.append(chunk)
        elif chunk.startswith(("---", "...")):
            raw = True
        else:
            try:
                options.update(yaml.load(chunk, Loader=Loader))  # a map, or a list of pairs
            except ScannerError:
                warn("malformed YAML block: left as it is")
                return None
    return options, "\n".join(data)


# The options: a value of the wrong type is made the right one as Python
# makes it (`str(1.5)`, `list("12")`, `bool("false")`), else the default.


def text(value: object) -> str:
    return value if isinstance(value, str) else str(value)


def maybe_list(value: object) -> list | None:
    return value if value is None or isinstance(value, list) else list(value)  # type: ignore[call-overload]


def maybe_fraction(value: object) -> float | None:
    if value is None or isinstance(value, float):
        return value
    return float(Fraction(value))  # type: ignore[arg-type]


def mapping(value: object) -> dict:
    return value if isinstance(value, dict) else dict(value)  # type: ignore[call-overload]


@dataclass
class Options:
    short_caption: str = ""
    caption: str = ""
    alignment: str = ""
    alignment_cells: str = ""
    width: list | None = None
    table_width: float | None = None
    header: bool = True  # (read, and never used: as the original)
    ms: list[int] | None = None
    ns_head: list[int] | None = None
    markdown: bool = False
    fancy_table: bool = False
    include: str = ""
    include_encoding: str = ""
    format: str = "csv"
    csv_kwargs: dict = field(default_factory=dict)

    @classmethod
    def of(cls, given: dict) -> "Options":
        cast = {"width": maybe_list, "ms": maybe_list, "ns_head": maybe_list, "table_width": maybe_fraction,
                "header": bool, "markdown": bool, "fancy_table": bool, "csv_kwargs": mapping}
        names = {f.name for f in fields(cls)}
        options = cls()
        # keys with - or _ alike, the later winning
        for name, value in {str(k).replace("-", "_"): v for k, v in given.items()}.items():
            if name not in names:
                continue
            try:
                setattr(options, name, cast.get(name, text)(value))
            except (ValueError, TypeError):
                warn(f"option {name.replace('_', '-')} with value {value} has an invalid type: the default")
        for name in ("ms", "ns_head"):
            value = getattr(options, name)
            try:
                if value is not None:
                    setattr(options, name, [int(x) for x in value])
            except (ValueError, TypeError):
                warn(f"option {name.replace('_', '-')} with value {value} has an invalid type: the default")
                setattr(options, name, None)
        return options

    def normalize(self, m: int, n: int) -> None:
        """What is wrong beyond the type, for a table of m rows and n columns."""
        total = 0.0
        if self.width is not None:
            widths: list = ["D"] * n
            for i, w in enumerate(self.width[:n]):
                try:
                    w = float(Fraction(w))
                except (ValueError, TypeError):
                    continue
                if w >= 0.0:
                    widths[i] = w
                    total += w
            self.width = widths
        if self.table_width is not None and self.table_width < total:
            warn(f"table-width smaller than the sum of the widths, {total}: dropped")
            self.table_width = None
        ms = self.ms
        if ms is not None and (len(ms) < 4 or len(ms) % 2 or any(x < 0 for x in ms) or sum(ms) != m):
            warn(f"ms {ms}: too short, of odd length, negative or not adding up to the rows, {m}: dropped")
            self.ms = None
        bodies = 1 if self.ms is None else len(self.ms) // 2 - 1
        ns = self.ns_head
        if ns is not None and (len(ns) != bodies or any(x > n for x in ns)):
            warn(f"ns-head {ns}: not one a body, {bodies}, or more than the columns, {n}: dropped")
            self.ns_head = None


def aligns(letters: str, n: int) -> list[Align]:
    """A row of alignments, by the original's arithmetic on each letter's
    code (L, R, C, D, and anything else somewhere); all default if one isn't
    ASCII."""
    letters = letters.strip().upper()
    if not letters.isascii():
        warn(f"non-ASCII character in {letters}: the default")
        return [ALIGN[0]] * n
    out: list[Align] = [ALIGN[(ord(c) - 3) % 11 % 6 % 4] for c in letters[:n]]
    default: list[Align] = [ALIGN[0]] * (n - len(out))
    return out + default


# The data


def load_csv(data: str, options: Options) -> list[list[str]]:
    """The rows, as long as the longest; FileNotFoundError, EmptyTable."""
    if options.include:
        try:
            with open(options.include, encoding=options.include_encoding or None, newline="") as f:
                rows = list(csv.reader(f, **options.csv_kwargs))
        except FileNotFoundError as e:
            raise FileNotFoundError(f"include path {options.include} not found.") from e
    else:
        rows = list(csv.reader(io.StringIO(data, newline=""), **options.csv_kwargs))
    if not any(cell.strip() for row in rows for cell in row):
        raise EmptyTable
    n = max(map(len, rows))
    return [row + [""] * (n - len(row)) for row in rows]


class Grid:
    """A table's texts, cells spanning others: each covered cell holds its
    span's text and anchor, as the original's array does (a later span
    overwrites an earlier one's)."""

    def __init__(self, rows: list[list[str | None]]):
        self.texts = rows
        self.spans: dict[tuple[int, int], tuple[int, int, int, int]] = {}  # (rows, columns, anchor)

    def put(self, text: str, rows: int, cols: int, i: int, j: int) -> None:
        for i_ in range(i, i + rows):
            for j_ in range(j, j + cols):
                self.texts[i_][j_] = text
                if (rows, cols) != (1, 1):
                    self.spans[i_, j_] = (rows, cols, i, j)

    def shape(self, i: int, j: int) -> tuple[int, int]:
        return self.spans.get((i, j), (1, 1, i, j))[:2]

    def is_at(self, i: int, j: int) -> bool:
        rows, cols, i_, j_ = self.spans.get((i, j), (1, 1, i, j))
        return (rows, cols) == (1, 1) or (i_, j_) == (i, j)


CELL_LINE = re.compile(r"^(\([0-9, ]+\))?({[^{}]*})?$")
FANCY = re.compile(r"^({[^{}]*})?? ?(---|===|___)? ?({[^{}]*})?$")


def markdown_cells(rows: list[list[str]], fancy: bool):
    """The cells of a table of markdown: a cell's first line may be its span,
    `(rows, columns)`, and attributes, `{...}`; with `fancy`, the first
    column marks the ends of the head (===), bodies' heads (---), bodies
    (___) and the foot (===), with attributes. The grid, the rows of each
    block (or None), the attributes of each block, of each row, of each
    cell (markdown: `[]{...}`, or "")."""
    m, n = len(rows), len(rows[0]) - fancy
    grid = Grid([[None] * n for _ in range(m)])
    cell_attrs = [[""] * n for _ in range(m)]
    for i in range(m):
        for j in range(n):
            if grid.texts[i][j] is not None:  # (in a span)
                continue
            string = rows[i][j + fancy]
            lines = string.splitlines()
            attr, shape, content = "", (1, 1), string
            if lines and (found := CELL_LINE.match(lines[0])):
                span, braces = found.groups("")
                try:
                    size = tuple(int(x.strip()) for x in span[1:-1].split(",")) if span else (1, 1)
                except ValueError:
                    warn(f"invalid cell shape {span}: ignored")
                else:
                    if len(size) != 2 or size[0] <= 0 or size[1] <= 0:
                        warn(f"invalid cell shape {size}: ignored")
                    elif size[0] + i > m or size[1] + j > n:
                        warn(f"a cell overflowing the table: its attributes ignored: {string}")
                    else:
                        attr = f"[]{braces}" if braces else ""
                        shape, content = size, "\n".join(lines[1:])
            cell_attrs[i][j] = attr
            grid.put(content, *shape, i, j)
    row_attrs = [""] * m
    ms = block_attrs = None
    if fancy:
        markers = []  # (row, marker, the block's attributes)
        for i, row in enumerate(rows):
            if not row[0].strip():
                continue
            if found := FANCY.match(row[0]):
                if found[2]:
                    markers.append((i, found[2], found[1] or ""))
                if found[3]:
                    row_attrs[i] = f"[]{found[3]}"
            else:
                warn(f"can't read the fancy-table cell {row[0]}: ignored")
        if markers:
            ms, block_attrs = fancy_blocks(markers, rows)
    return grid, ms, block_attrs, row_attrs, cell_attrs


def fancy_blocks(markers: list[tuple[int, str, str]], rows: list[list[str]]):
    """`ms` and the blocks' attributes from the markers."""
    sizes = [b - a for a, b in zip([-1] + [i for i, _, _ in markers], [i for i, _, _ in markers])]
    head = markers[0][1] == "==="
    foot = len(markers) > 1 and markers[-1][1] == "==="
    bodies: list[dict[str, tuple[int, str]]] = []
    for k in range(int(head), len(markers) - int(foot)):
        i, marker, attr = markers[k]
        if marker == "___":
            if bodies and "body" not in bodies[-1]:
                bodies[-1]["body"] = (sizes[k], attr)
            else:
                bodies.append({"body": (sizes[k], attr)})
        elif marker == "---":
            bodies.append({"head": (sizes[k], attr)})
        else:
            warn(f"a fancy-table row neither head nor foot: ignored: {rows[i][0]}")
    ms = [sizes[0] if head else 0]
    attrs = [f"[]{markers[0][2]}" if head and markers[0][2] else ""]
    for body in bodies:
        attr = ""
        for part in ("head", "body"):
            size, a = body.get(part, (0, ""))
            ms.append(size)
            attr = a or attr  # (the body's rows' attributes win)
        attrs.append(f"[]{attr}" if attr else "")
    ms.append(sizes[-1] if foot else 0)
    attrs.append(f"[]{markers[-1][2]}" if foot and markers[-1][2] else "")
    return ms, attrs


def text_width(string: str) -> int:
    """The longest line's characters, and 3 (as pandoc's widths)."""
    lines = string.splitlines()
    return max(map(len, lines)) + 3 if lines else 3


def auto_width(grid: Grid, widths: list[float | None], table_width: float) -> list[float | None]:
    """The columns without a width get one, by the length of their text, so
    that the widths add up to `table_width`; a cell spanning columns gives
    what its first column lacks to the next."""
    n = len(widths)
    found: list[list] = [[] for _ in range(n)]
    for i, row in enumerate(grid.texts):
        for j in range(n):
            if grid.is_at(i, j):
                w, cols = text_width(row[j] or ""), grid.shape(i, j)[1]
                found[j].append((w, cols) if cols > 1 else w)
    lengths = []
    for j in range(n):
        longest = max(w for w in found[j] if isinstance(w, int))
        lengths.append(longest)
        for w in found[j]:
            if isinstance(w, tuple) and w[0] > longest:
                found[j + 1].append((w[0] - longest, w[1] - 1) if w[1] > 2 else w[0] - longest)
    unset = [j for j in range(n) if widths[j] is None]
    total = sum(lengths[j] for j in unset)
    out = list(widths)
    if total > 0:
        scale = (table_width - sum(w for w in widths if w is not None)) / total
        for j in unset:
            out[j] = lengths[j] * scale
    else:
        for j in unset:
            out[j] = 0.0
    return out


# The table


def read_markdown(texts: list[str], ctx: Context) -> list[list[Block]]:
    """Each text's blocks, read by pandoc as markdown, the conversion's way,
    in one call; an empty text is none. Tabs are expanded, as the original's
    `pandoc` command does (a read with a conversion keeps them)."""
    if libpandoc is None:
        raise RuntimeError("pantable: markdown cells need pandoc's reader, libpandoc (pip install libpandoc)")
    todo = [t for t in texts if t]
    docs = iter(libpandoc.read_many(todo, ctx.conversion, {"from": "markdown", "preserve-tabs": False}) if todo else [])
    return [next(docs).blocks if t else [] for t in texts]


def attr_of(blocks: list[Block]) -> Attr:
    """The attributes of `[]{...}` read: its span's; else none."""
    if blocks:
        first = getattr(blocks[0], "content", [None])[0]
        if isinstance(first, Span):
            return first.attr
        warn(f"can't read the attributes {blocks}: none")
    return Attr()


f = Filter()


@f.on(CodeBlock)
def table(block: CodeBlock, ctx: Context) -> Table | list | None:
    if "table" not in block.classes:
        return None
    parsed = split(block.text)
    if parsed is None:
        return None
    options = Options.of(parsed[0])
    try:
        if options.format != "csv":
            raise ValueError(f"Unknown format: {options.format}")
        rows = load_csv(parsed[1], options)
    except FileNotFoundError as e:
        warn(f"{e} The code block is left as it is.")
        return None
    except EmptyTable:
        warn("table is empty: deleted")
        return []
    if options.markdown:
        grid, ms, block_attrs, row_attrs, cell_attrs = markdown_cells(rows, options.fancy_table)
    else:
        grid, ms, block_attrs, row_attrs, cell_attrs = Grid(rows), None, None, [], []  # type: ignore[arg-type]
    m, n = len(grid.texts), len(grid.texts[0])
    options.normalize(m, n)
    ms = ms or options.ms or [1, 0, m - 1, 0]
    # the table has the rows of its blocks: with fancy-table, the rows after
    # the last marker are dropped (as the original, which sizes it by ms)
    m = sum(ms)
    grid.texts = grid.texts[:m]
    row_attrs = row_attrs[:m]
    ns_head = options.ns_head or [0] * (len(ms) // 2 - 1)
    widths: list[float | None] = [None if w == "D" else float(w) for w in options.width or ["D"] * n]
    if options.table_width is not None:
        widths = auto_width(grid, widths, options.table_width)
    spec = [ColSpec(a, ColWidth(w) if w is not None else None) for a, w in zip(aligns(options.alignment, n), widths)]
    lines = options.alignment_cells.strip().splitlines()[:m]
    cell_aligns = [aligns(line, n) for line in lines] + [aligns("", n) for _ in range(m - len(lines))]

    anchors = [(i, j) for i in range(m) for j in range(n) if grid.is_at(i, j)]
    if options.markdown:
        # every text of the table in one call: captions, cells and their
        # attributes, rows' and blocks' attributes
        texts = [options.caption, options.short_caption, *(grid.texts[i][j] or "" for i, j in anchors),
                 *(cell_attrs[i][j] for i, j in anchors), *row_attrs, *(block_attrs or [""] * (len(ms) // 2 + 1))]
        read = read_markdown(texts, ctx)
        caption = Caption(*read[0], short=read[1][0].content if read[1] else None)  # type: ignore[attr-defined]
        k = 2 + len(anchors)
        contents = {a: plain(blocks) for a, blocks in zip(anchors, read[2:k])}
        attrs = {a: attr_of(blocks) for a, blocks in zip(anchors, read[k:k + len(anchors)])}
        k += len(anchors)
        rows_attr = [attr_of(blocks) for blocks in read[k:k + m]]
        blocks_attr = [attr_of(blocks) for blocks in read[k + m:]]
    else:
        caption = Caption(Para(Str(options.caption)), short=[Str(options.short_caption)])
        contents = {a: [Plain(Str(grid.texts[a[0]][a[1]] or ""))] for a in anchors}
        attrs = {a: Attr() for a in anchors}
        rows_attr = [Attr() for _ in range(m)]
        blocks_attr = [Attr() for _ in range(len(ms) // 2 + 1)]

    def row(i: int) -> Row:
        cells = [Cell(contents[i, j], cell_aligns[i][j], *grid.shape(i, j), attr=attrs[i, j])
                 for j in range(n) if grid.is_at(i, j)]
        return Row(*cells, attr=rows_attr[i])

    # the rows of each block
    ends = list(accumulate(ms))[:-1]
    parts = [[row(i) for i in range(a, b)] for a, b in zip([0, *ends], [*ends, m])]
    head = TableHead(*parts[0], attr=blocks_attr[0])
    bodies = [TableBody(parts[2 + 2 * k], parts[1 + 2 * k], ns, attr=blocks_attr[1 + k]) for k, ns in enumerate(ns_head)]
    foot = TableFoot(*parts[-1], attr=blocks_attr[-1])
    attr = Attr(block.attr.identifier, [c for c in block.classes if c != "table"], block.attr.attributes)
    return Table(caption, spec, head, bodies, foot, attr=attr)


def plain(blocks: list[Block]) -> list[Block]:
    """A cell of one paragraph is plain text."""
    match blocks:
        case [Para() as para]:
            return [Plain(*para.content)]
    return blocks


if __name__ == "__main__":
    main(f)
