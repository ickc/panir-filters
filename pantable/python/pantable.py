#!/usr/bin/env python3
"""Code blocks of class `table`, CSV with a YAML header of options, made
tables: a pandoc filter.

The block holds CSV, after a YAML header of options (between `---` and
`---` or `...`, starting the block): `caption`, `short-caption`,
`alignment` (a letter a column among L, R, C, D), `alignment-cells` (a
word of letters a row), `width` (a number or fraction a column, or D),
`table-width` (then the columns without a width get one from the length of
their text), `ms` (rows of the head, of each body's head and rows, of the
foot), `ns-head` (row head columns of each body), `markdown` (cells,
captions and attributes read as markdown), `fancy-table` (with markdown:
the first column marks the head, bodies and foot, with attributes; a
cell's first line its span and attributes), `csv-kwargs` (Python's
csv.reader's; `\\t` is a tab), `format` (csv only). The document's metadata
under `pantable` gives defaults, a header's keys replacing them.

The file a block's `include` attribute names is panclude's to include
(`pandoc -L panclude.lua`, or libpandoc's `panclude` step, run before this
filter): a block still carrying the attribute is left as it is.

The header is read by pandoc, as a metadata block (`-f markdown-smart`;
panir's `yaml_filter`), the markdown of the cells too: through `libpandoc`
(the pandoc the filter runs in under pandocpy or `libpandoc.convert`, or
libpandoc as a library), else the `pandoc` command, a process a text. The
CSV is Python's csv module's, as the original's.

A port of ickc/pantable's `pantable` filter, 0.15.0 with the include
attribute (© 2016-2026 Kolen Cheung; BSD-3-Clause, see ../LICENSE).
"""

import csv
import io
import re
import subprocess
import sys
from itertools import accumulate
from typing import Literal

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
    MetaBlocks,
    MetaInlines,
    MetaMap,
    MetaString,
    MetaValue,
    Options,
    Pandoc,
    Para,
    Plain,
    RawInline,
    Row,
    Span,
    Str,
    Table,
    TableBody,
    TableFoot,
    TableHead,
    blocks_of,
    inlines_of,
    loads,
    lua_number,
    main,
    stringify,
    to_data,
    yaml_filter,
)

try:
    import libpandoc  # pandoc's reader: the header, and markdown cells
except ImportError:
    libpandoc = None

Align = Literal["AlignDefault", "AlignLeft", "AlignRight", "AlignCenter"]
ALIGN: tuple[Align, ...] = ("AlignDefault", "AlignLeft", "AlignRight", "AlignCenter")


def warn(message: str) -> None:
    print(f"pantable: {message}", file=sys.stderr)


class EmptyTable(Exception):
    pass


def read(texts: list[str], ctx: Context, fmt: str) -> list[Pandoc]:
    """Texts read by pandoc, a document each: in one call through libpandoc,
    the conversion's way, else the `pandoc` command, a process a text. Tabs
    are expanded, as the original's `pandoc` command does (a read with a
    conversion keeps them)."""
    if libpandoc is not None:
        return libpandoc.read_many(texts, ctx.conversion, {"from": fmt, "preserve-tabs": False})
    command = ["pandoc", "-f", fmt, "-t", "json"]
    return [loads(subprocess.run(command, input=t, stdout=subprocess.PIPE, text=True, check=True).stdout) for t in texts]


# The options, from the metadata pandoc read: a key as written or with `_`
# for `-` (the hyphenated one if both); a value of another shape than the
# option's is its default, with a warning.


def get(options: Options, key: str, shape: str):
    """An option's value as `Options.text`, `.boolean`, `.list` or `.map`
    reads it, `.get` raw; None if missing or of another shape."""
    key = key if key in options else key.replace("-", "_")
    if key not in options:
        return None
    value = getattr(options, shape)(key)
    if value is None:
        warn(f"option {key} with value {to_data(options[key])} has an invalid type: the default")
    return value


def fraction(text: object) -> float | None:
    """A number as Lua reads one, or two with `/` between (`1/3`); None
    for anything else. A zero denominator fails, as the original."""
    if not isinstance(text, str):
        return None
    top, slash, bottom = text.partition("/")
    a, b = lua_number(top), lua_number(bottom) if slash else 1
    return None if a is None or b is None else a / b


def whole(text: object) -> int | None:
    """A whole number as Lua reads one (`3`, `3.0`); None for anything else."""
    number = lua_number(text) if isinstance(text, str) else None
    return int(number) if number is not None and float(number).is_integer() else None


def integers(values: list | None, key: str) -> list[int] | None:
    """Whole numbers (`ms`, `ns-head`), else the default."""
    out = [whole(x) for x in values or []]
    if values is None or None not in out:
        return None if values is None else out  # type: ignore[return-value]
    warn(f"option {key} with value {values} has an invalid type: the default")
    return None


def words(text: str) -> list[str]:
    """Words of text: split at spaces, tabs and line breaks."""
    return re.findall(r"[^ \t\n\r]+", text)


def kwarg(value: MetaValue) -> object:
    """A csv-kwargs value: a boolean, or text (raw TeX's too: pandoc reads
    `\\t` so), `\\t` a tab, a whole number an integer (`quoting: 3`)."""
    if isinstance(value, MetaInlines):
        text = "".join(x.text if isinstance(x, RawInline) else stringify(x) for x in value.content)
    else:
        text = to_data(value)
    return "\t" if text == "\\t" else text if whole(text) is None else whole(text)


class Settings:
    def __init__(self, options: Options):
        if "include" in options:
            warn("the include key is not read: name the file with the include attribute, and run panclude first")
        width, kwargs = get(options, "width", "list"), get(options, "csv-kwargs", "get")
        if kwargs is not None and not isinstance(kwargs, MetaMap):
            warn(f"option csv-kwargs with value {to_data(kwargs)} has an invalid type: the default")
        self.caption: MetaValue | None = get(options, "caption", "get")
        self.short_caption: MetaValue | None = get(options, "short-caption", "get")
        self.alignment: str = get(options, "alignment", "text") or ""
        self.alignment_cells = words(get(options, "alignment-cells", "text") or "")
        self.width: list | None = None if width is None else [fraction(w) for w in width]
        self.table_width = fraction(get(options, "table-width", "text"))
        self.ms = integers(get(options, "ms", "list"), "ms")
        self.ns_head = integers(get(options, "ns-head", "list"), "ns-head")
        self.markdown = bool(get(options, "markdown", "boolean"))
        self.fancy_table = bool(get(options, "fancy-table", "boolean"))
        self.csv_kwargs = {k: kwarg(v) for k, v in kwargs.content.items()} if isinstance(kwargs, MetaMap) else {}
        self.format: str = get(options, "format", "text") or "csv"

    def normalize(self, m: int, n: int) -> None:
        """What is wrong beyond the type, for a table of m rows and n columns."""
        total = 0.0
        if self.width is not None:
            widths: list = [None] * n
            for i, w in enumerate(self.width[:n]):
                if w is not None and w >= 0.0:
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


def load_csv(data: str, kwargs: dict) -> list[list[str]]:
    """The rows, as long as the longest; EmptyTable."""
    rows = list(csv.reader(io.StringIO(data, newline=""), **kwargs))
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
    """Each text's blocks, read by pandoc as markdown, in one call; an empty
    text is none."""
    todo = [t for t in texts if t]
    docs = iter(read(todo, ctx, "markdown") if todo else [])
    return [next(docs).blocks if t else [] for t in texts]


def attr_of(blocks: list[Block]) -> Attr:
    """The attributes of `[]{...}` read: its span's; else none."""
    if blocks:
        first = getattr(blocks[0], "content", [None])[0]
        if isinstance(first, Span):
            return first.attr
        warn(f"can't read the attributes {blocks}: none")
    return Attr()


def caption(long: MetaValue | None, short: MetaValue | None, markdown: bool) -> Caption:
    """The caption: with markdown, as pandoc read the header (the original
    reads its text as markdown); else its text, one `Str`, as the original's
    (a plain table's caption is an empty paragraph). Not text: none, with a
    warning; a short caption of blocks too."""
    long, short = text_or_none("caption", long), text_or_none("short-caption", short)
    if not markdown:
        return Caption(Para(Str(to_data(long) if long else "")), short=[Str(to_data(short) if short else "")])
    if isinstance(short, MetaBlocks):
        warn("a short-caption of blocks: none")
        short = None
    short_inlines = inlines_of(short) if short else []
    inlines = inlines_of(long) if long and not isinstance(long, MetaBlocks) else []
    blocks = blocks_of(long) if isinstance(long, MetaBlocks) else [Para(*inlines)] if inlines else []
    return Caption(*blocks, short=short_inlines or None)


def text_or_none(key: str, value: MetaValue | None) -> MetaValue | None:
    if value is None or isinstance(value, (MetaString, MetaInlines, MetaBlocks)):
        return value
    warn(f"option {key} with value {to_data(value)} has an invalid type: the default")
    return None


f = Filter()


@f.on(Pandoc)
def tables(doc: Pandoc, ctx: Context) -> Pandoc:
    def headers(texts: list[str]) -> list[Pandoc]:
        return read(texts, ctx, "markdown-smart")

    return yaml_filter(doc, "table", "pantable", headers, lambda options, data, block: table(options, data, block, ctx))


def table(options: Options, data: str, block: CodeBlock, ctx: Context) -> Table | list | None:
    if block.attr.attributes.get("include") is not None:
        warn("a code block with an include attribute: left as it is (run panclude first)")
        return None
    settings = Settings(options)
    try:
        if settings.format != "csv":
            raise ValueError(f"Unknown format: {settings.format}")
        rows = load_csv(data.strip("\n"), settings.csv_kwargs)  # (blank lines around the data dropped)
    except EmptyTable:
        warn("table is empty: deleted")
        return []
    if settings.markdown:
        grid, ms, block_attrs, row_attrs, cell_attrs = markdown_cells(rows, settings.fancy_table)
    else:
        grid, ms, block_attrs, row_attrs, cell_attrs = Grid(rows), None, None, [], []  # type: ignore[arg-type]
    m, n = len(grid.texts), len(grid.texts[0])
    settings.normalize(m, n)
    ms = ms or settings.ms or [1, 0, m - 1, 0]
    # the table has the rows of its blocks: with fancy-table, the rows after
    # the last marker are dropped (as the original, which sizes it by ms)
    m = sum(ms)
    grid.texts = grid.texts[:m]
    row_attrs = row_attrs[:m]
    ns_head = settings.ns_head or [0] * (len(ms) // 2 - 1)
    widths: list[float | None] = settings.width or [None] * n
    if settings.table_width is not None:
        widths = auto_width(grid, widths, settings.table_width)
    spec = [ColSpec(a, ColWidth(w) if w is not None else None) for a, w in zip(aligns(settings.alignment, n), widths)]
    lines = settings.alignment_cells[:m]
    cell_aligns = [aligns(line, n) for line in lines] + [aligns("", n) for _ in range(m - len(lines))]

    anchors = [(i, j) for i in range(m) for j in range(n) if grid.is_at(i, j)]
    if settings.markdown:
        # every text of the table in one call: cells and their attributes,
        # rows' and blocks' attributes
        texts = [*(grid.texts[i][j] or "" for i, j in anchors), *(cell_attrs[i][j] for i, j in anchors), *row_attrs,
                 *(block_attrs or [""] * (len(ms) // 2 + 1))]
        read = read_markdown(texts, ctx)
        k = len(anchors)
        contents = {a: plain(blocks) for a, blocks in zip(anchors, read[:k])}
        attrs = {a: attr_of(blocks) for a, blocks in zip(anchors, read[k:2 * k])}
        rows_attr = [attr_of(blocks) for blocks in read[2 * k:2 * k + m]]
        blocks_attr = [attr_of(blocks) for blocks in read[2 * k + m:]]
    else:
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
    return Table(caption(settings.caption, settings.short_caption, settings.markdown), spec, head, bodies, foot,
                 attr=attr)


def plain(blocks: list[Block]) -> list[Block]:
    """A cell of one paragraph is plain text."""
    match blocks:
        case [Para() as para]:
            return [Plain(*para.content)]
    return blocks


if __name__ == "__main__":
    main(f)
