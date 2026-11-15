#!/usr/bin/env python3
"""amsthm's theorem and proof environments for any output format: a pandoc
filter.

The metadata declares the environments (`amsthm: {plain: [Theorem:
[Lemma]], definition: [Definition], ...}`, with options); a div of one's
class is one. In LaTeX (and beamer) they are amsthm's own: `\\newtheorem`s
in `header-includes`, `\\begin{...}...\\end{...}` around each div, and
citations of their identifiers `\\ref`s. In any other format the filter
numbers them as amsthm would (counters shared, within sections), sets
their headings and bodies in their style's fonts, ends proofs with a
symbol, and makes citations of them links.

A div's `info` is its note, `Theorem 1 (note)`, which the original reads as
markdown with pandoc. This port has no markdown reader: a note of plain
text (words of letters and digits, a hyphen inside a word, and `,;:?!`
after one, single spaces between) is read as pandoc reads it; any other
is kept as its text, with a warning.

A port of ickc/pandoc-amsthm's amsthm.lua (Copyright (c) 2016-2026, Kolen
Cheung; BSD-3-Clause, see ../LICENSE).
"""

import copy
import math
import re
import sys
from collections.abc import Mapping
from dataclasses import dataclass, field
from typing import Any

from panir import (
    Block,
    Cite,
    CitationMode,
    Context,
    Div,
    Emph,
    Filter,
    Header,
    Inline,
    LineBreak,
    Link,
    Math,
    Meta,
    MetaBlocks,
    MetaBool,
    MetaInlines,
    MetaList,
    MetaMap,
    MetaValue,
    Pandoc,
    Para,
    Plain,
    RawBlock,
    RawInline,
    SmallCaps,
    SoftBreak,
    Space,
    Span,
    Str,
    Strong,
    inlines,
    inlines_of,
    items,
    main,
    stringify,
    to_roman_numeral,
)

# LaTeX's sectioning units, by depth: parent_counter's values.
PARENT_COUNTERS = {"part": 0, "chapter": 1, "section": 2, "subsection": 3, "subsubsection": 4, "paragraph": 5, "subparagraph": 6}
# Document classes whose level-1 headings pandoc's LaTeX writer makes chapters.
CHAPTER_CLASSES = {"memoir", "book", "report", "scrreprt", "scrreport", "scrbook", "extreport", "extbook", "tufte-book"}
FONT_TYPES = {"bold": Strong, "italic": Emph, "smallcaps": SmallCaps}
FONT_LATEX = {"bold": "\\bfseries", "italic": "\\itshape", "smallcaps": "\\scshape"}
# Keys of the metadata that are options, not styles.
OPTION_KEYS = {"styles", "name_to_text", "parent_counter", "counter_depth", "counter_ignore_headings", "css", "swapnumbers", "qed_symbol"}
LATEX_LIKE = {"latex", "beamer"}
# Formats whose header-includes are HTML.
HTML_LIKE = {"html", "html4", "html5", "chunkedhtml", "epub", "epub2", "epub3", "revealjs", "slidy", "slideous", "dzslides", "s5"}
CSS = "<style>\n:where(.amsthm-qed) { float: right; }\n</style>"
BLANK = (Space, SoftBreak, LineBreak)
Font = type[Emph] | type[Strong] | type[SmallCaps]


def warn(message: str) -> None:
    print(f"[amsthm] warning: {message}", file=sys.stderr)


def lua_number(text: str) -> float | None:
    """A number as Lua's `tonumber` reads text: decimal or hexadecimal,
    blanks around it; None if not one."""
    text = text.strip(" \t\n\v\f\r")
    if re.fullmatch(r"[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?", text):
        return int(text) if text.lstrip("+-").isdigit() else float(text)
    if re.fullmatch(r"[+-]?0[xX][0-9a-fA-F]+", text):
        return int(text, 16)
    return None


# A note of plain text: words of letters and digits (anything not ASCII but
# a space), a hyphen inside one, and one of `,;:?!` after one, single
# spaces between. pandoc's markdown reader gives such a text as it is, a
# Str a word.
PLAIN = re.compile(r"(?:[A-Za-z0-9]|[^\x00-\x7f\x85\xa0\u1680\u2000-\u200a\u2028\u2029\u202f\u205f\u3000\ufeff])+")
PLAIN_NOTE = re.compile(rf"{PLAIN.pattern}(?:-{PLAIN.pattern})*[,;:?!]?(?: {PLAIN.pattern}(?:-{PLAIN.pattern})*[,;:?!]?)*")


def note_inlines(info: str) -> list[Inline]:
    """A div's note, as the original reads it with pandoc's markdown reader
    where it is plain text; otherwise its text, with a warning."""
    if not PLAIN_NOTE.fullmatch(info):
        warn(f"the note {info!r} has markup, which needs pandoc's markdown reader: kept as text")
    return inlines(info)


# ---------------------------------------------------------------------------
# Fonts: an element of one around each Str, and what that does to nesting


def wrap_strs(font: Font) -> Filter:
    """Each Str in the font; an element of the font in it undone for
    emphasis (`\\emph` toggles), flattened for the others (bold in bold is
    bold); citations and `\\ref`s the font applies to in it (`font_ref`);
    then neighbouring elements of the font in a block joined."""
    f = Filter()
    f.on(Str)(lambda s: font(s))

    if font is Emph:

        @f.on(font)
        def cancel(x: Inline) -> list[Inline]:
            out: list[Inline] = []
            for child in x.content:  # pyright: ignore[reportAttributeAccessIssue]
                if isinstance(child, font):
                    out += child.content  # pyright: ignore[reportAttributeAccessIssue]
                elif isinstance(child, BLANK):
                    out.append(child)
                else:
                    out.append(font(child))
            return out

    else:

        @f.on(font)
        def flatten(x: Inline) -> None:
            content = x.content  # pyright: ignore[reportAttributeAccessIssue]
            content[:] = [y for child in content for y in (child.content if isinstance(child, font) else [child])]

    ref = font_ref(font)
    f.on(Cite)(ref)
    f.on(RawInline)(ref)
    merge = merge_consecutive(font)
    for block in (Para, Plain, Header):
        f.on(block)(merge)
    return f


def merge_consecutive(font: Font):
    """Joins elements of the font next to each other, or with one Space
    between, in a block's inlines."""

    def merge(block: Block) -> None:
        content: list[Inline] = block.content  # pyright: ignore[reportAttributeAccessIssue]
        out: list[Inline] = []
        run: list[Inline] | None = None
        for i, x in enumerate(content):
            if isinstance(x, font):
                run = (run or []) + list(x.content)  # pyright: ignore[reportAttributeAccessIssue]
            elif run is not None and isinstance(x, Space) and i + 1 < len(content) and isinstance(content[i + 1], font):
                run.append(Space())
            else:
                if run is not None:
                    out.append(font(*run))
                    run = None
                out.append(x)
        if run is not None:
            out.append(font(*run))
        if len(out) != len(content):
            content[:] = out

    return merge


def font_ref(font: Font):
    """In LaTeX, the font of the text around a reference applies to what
    `\\ref` and an in-text citation print; `\\eqref` is upright, which
    undoes a shape (italic, small caps) but not bold. Those it applies to
    are put in the font, as LaTeX prints them."""
    shape = font is not Strong

    def ref(x: Cite | RawInline) -> Inline | None:
        if isinstance(x, Cite):
            mode = x.citations[0].mode if len(x.citations) == 1 else None
            if len(x.citations) > 1:
                mode = CitationMode.NormalCitation  # (`\eqref{a}, \eqref{b}`)
            if mode != CitationMode.AuthorInText and (shape or mode != CitationMode.NormalCitation):
                return None
        elif x.format != "tex" or not (re.fullmatch(r"\\ref\{.*\}", x.text, re.S) or (not shape and re.fullmatch(r"\\eqref\{.*\}", x.text, re.S))):
            return None
        return font(x)

    return ref


def in_font(content: list[Inline], font: list[str]) -> list[Inline]:
    """The inlines in the elements of a font such as ["bold", "italic"]."""
    for name in reversed(font):
        content = [FONT_TYPES[name](*content)]
    return content


def join_runs(runs: list[tuple[list[Inline], list[str]]]) -> list[Inline]:
    """Runs of inlines, each with its font, each stretch of one font in its
    elements once ("Theorem 1." one Strong, not three)."""
    out: list[Inline] = []
    content: list[Inline] = []
    font: list[str] | None = None
    for run, run_font in runs:
        if run_font != font:
            if font is not None:
                out += in_font(content, font)
            content, font = [], run_font
        content += run
    if font is not None:
        out += in_font(content, font)
    return out


def named(name: str, number: Inline) -> list[Inline]:
    """`name`, a no-break space, then `number`: "Theorem 1"."""
    return [*inlines(name), Str("\u00a0"), number]


# ---------------------------------------------------------------------------
# The environments


@dataclass
class Style:
    headfont: list[str]
    bodyfont: list[str]
    headpunct: str = "."
    headspace: str = " "  # " ", "newline", or a length (LaTeX)
    above: str = ""  # LaTeX only, as below and indent
    below: str = ""
    indent: str = ""


BUILTIN_STYLES = {
    "plain": Style(["bold"], ["italic"]),
    "definition": Style(["bold"], []),
    "remark": Style(["italic"], []),
}


@dataclass(eq=False)
class Theorem:
    style: str
    env_name: str
    text: str = ""
    parent_counter: str | None = None
    shared_counter: str | None = None
    numbered: bool = True

    def __post_init__(self) -> None:
        if self.env_name.endswith("*"):
            self.env_name, self.numbered = self.env_name[:-1], False
        self.text = self.text or self.env_name
        if self.parent_counter is not None and self.parent_counter not in PARENT_COUNTERS:
            warn(f"unsupported parent_counter {self.parent_counter}, ignoring")
            self.parent_counter = None
        if self.numbered and self.parent_counter is not None and self.shared_counter is not None:
            self.shared_counter = None

    def latex(self) -> str:
        if not self.numbered:
            return f"\\newtheorem*{{{self.env_name}}}{{{self.text}}}"
        if self.shared_counter is not None:
            return f"\\newtheorem{{{self.env_name}}}[{self.shared_counter}]{{{self.text}}}"
        parent = f"[{self.parent_counter}]" if self.parent_counter is not None else ""
        return f"\\newtheorem{{{self.env_name}}}{{{self.text}}}{parent}"

    @property
    def class_name(self) -> str:
        return self.env_name.replace(" ", "_")

    @property
    def counter_name(self) -> str:
        return self.shared_counter or self.env_name

    def header(self, options: "Options", identifier: str, info: str | None) -> list[Inline]:
        """The heading, as amsthm's `\\thmhead` writes it: the name and the
        number in the style's heading font, the number upright, the note in
        parentheses in the note font, and the punctuation; then the space
        after the heading."""
        style = options.styles[self.style]
        head = style.headfont
        number = None
        if self.numbered:
            name = self.counter_name
            options.theorem_counters[name] = options.theorem_counters.get(name, 0) + 1
            counter = options.counters.get(name, Counter())
            parts = []
            for level in range(counter.first, math.floor(counter.depth) + 1):
                n = options.header_counters[level]
                # \thepart is a Roman numeral, and empty before the first part
                parts.append((to_roman_numeral(n) if n > 0 else "") if level == options.part_level else str(n))
            number = ".".join([*parts, str(options.theorem_counters[name])])
            if identifier:
                options.identifiers[identifier] = number
                options.names[identifier] = self.text

        runs: list[tuple[list[Inline], list[str]]] = [(inlines(self.text), head)]
        if number and options.swapnumbers:
            # \swappedhead: the number first, in the heading font, then a tie
            runs.insert(0, ([Str(number), Str("\u00a0")], head))
        elif number:
            runs.append(([Space(), Str(number)], [f for f in head if f == "bold"]))  # upright
        if info:
            runs.append(([Space(), Str("("), *note_inlines(info), Str(")")], []))
        runs.append(([Str(style.headpunct)], head))
        return [*join_runs(runs), LineBreak() if style.headspace == "newline" else Space()]


class Proof(Theorem):
    def __init__(self, text: str | None) -> None:
        # (`name_to_text: {proof: ""}` is an empty name, which a theorem's
        # text takes as none: the proof's is then its environment's, "proof")
        super().__init__("proof", "proof", "Proof" if text is None else text, numbered=False)

    def header(self, options: "Options", identifier: str, info: str | None) -> list[Inline]:
        """"Proof.", in italics; or the note, in italics but its references,
        for the whole heading."""
        if not info:
            return [Emph(*inlines(self.text + ".")), Space()]
        # each Str in italics, emphasis in it upright, references as the
        # font applies to them; then neighbouring italics joined (in the
        # paragraph itself: a walk is of what is in it)
        para = Para(*note_inlines(info)).walk(wrap_strs(Emph))
        merge_consecutive(Emph)(para)
        return [*para.content, Emph(Str(".")), Space()]


@dataclass
class Counter:
    """The heading levels in a counter's number, `first` to `depth`; a
    heading at one of them restarts it."""

    first: int = 1
    depth: float = 0


@dataclass
class Writer:
    """The writer's options the original reads (`PANDOC_WRITER_OPTIONS`):
    panir has them as the conversion's options, in defaults-file keys,
    where its host knows them; a JSON filter is given none."""

    number_sections: bool = False
    top_level_division: str = "default"
    number_offset: list[int] = field(default_factory=list)
    secnumdepth: str | None = None  # (a variable, -V)

    @classmethod
    def of(cls, options: Mapping[str, Any] | None) -> "Writer":
        o = options or {}
        variables = o.get("variables") or {}
        depth = variables.get("secnumdepth")
        return cls(
            bool(o.get("number-sections", False)),
            str(o.get("top-level-division", "default")),
            list(o.get("number-offset") or []),
            None if depth is None else str(depth),
        )


@dataclass
class Options:
    css: bool
    styles: dict[str, Style]
    user_styles: list[str]
    swapnumbers: bool
    qed_symbol: str
    counters: dict[str, Counter]
    numbered_depth: float
    part_level: int | None
    proof_name: str | None
    theorems: list[Theorem]  # in order
    by_class: dict[str, Theorem]
    counter_depth: float
    ignored_headings: set[str]
    header_counters: dict[int, int]
    theorem_counters: dict[str, int] = field(default_factory=dict)
    identifiers: dict[str, str] = field(default_factory=dict)  # (LaTeX: "")
    names: dict[str, str] = field(default_factory=dict)
    unnumbered: set[str] = field(default_factory=set)
    warned: set[str] = field(default_factory=set)


def as_list(value: MetaValue | None) -> list[MetaValue]:
    """A value as a list: a single value a list of one."""
    return items(value)


def lua_pairs(value: MetaValue, what: str) -> dict[str, MetaValue]:
    """A map's entries, as Lua's `pairs`: a list's are its indices, which
    no name matches; text or a boolean fails, as in Lua."""
    match value:
        case MetaMap(content):
            return dict(content)
        case MetaList() | MetaInlines() | MetaBlocks():
            return {}
        case _:
            raise ValueError(f"amsthm: {what} is {type(value).__name__}, not a map")


def read_font(value: MetaValue | None, where: str) -> list[str]:
    font = []
    for v in as_list(value):
        name = stringify(v)
        if name in FONT_TYPES:
            font.append(name)
        elif name != "normal":
            warn(f"unknown font {name} in {where}; use bold, italic, smallcaps or normal")
    return font


def read_styles(value: MetaValue | None) -> tuple[dict[str, Style], list[str]]:
    """amsthm's styles, and those under `styles` (`\\newtheoremstyle`'s
    arguments; what is left out as for a theorem), with the names of the
    user's, sorted."""
    styles = dict(BUILTIN_STYLES)
    if not isinstance(value, MetaMap):
        return styles, []
    kept = []
    for name in sorted(value.content):
        spec = value.content[name]
        if name in OPTION_KEYS:
            warn(f"a style cannot be called {name}, which is an option of its own; ignoring it")
        elif name == "proof":
            warn("a style cannot be called proof, which lists the proof environments; ignoring it")
        elif not isinstance(spec, MetaMap):
            warn(f"style {name} is not a map of its settings; ignoring it")
        else:
            settings = spec.content

            def get(key: str, default: str) -> str:
                return stringify(settings[key]) if key in settings else default

            where = f"style {name}"
            styles[name] = Style(
                read_font(settings["headfont"], where) if "headfont" in settings else ["bold"],
                read_font(settings.get("bodyfont"), where),
                get("headpunct", "."),
                get("headspace", " "),
                get("above", ""),
                get("below", ""),
                get("indent", ""),
            )
            kept.append(name)
    return styles, kept


def top_level_division(meta: Meta, writer: Writer) -> str:
    """The LaTeX unit that level-1 headings are, as pandoc decides it."""
    if writer.top_level_division in PARENT_COUNTERS:
        return writer.top_level_division
    documentclass = stringify(meta["documentclass"]) if meta.get("documentclass") else None
    return "chapter" if documentclass in CHAPTER_CLASSES else "section"


def numbered_depth(meta: Meta, writer: Writer, top: str) -> float:
    """How many heading levels step their LaTeX counters: none without
    -N, else as deep as secnumdepth."""
    if not writer.number_sections:
        return 0
    value = meta.get("secnumdepth")
    if value is not None:
        text = stringify(value) if value else None  # (false: as if unset, but not the variable)
    else:
        text = writer.secnumdepth
    secnumdepth = lua_number(text) if text is not None else None
    return (5 if secnumdepth is None else secnumdepth) - PARENT_COUNTERS[top] + 2


def read_options(meta: Meta, writer: Writer) -> Options:
    config = meta.get("amsthm")
    # (an empty `amsthm:` is text, not a map: it defines nothing)
    opt = dict(config.content) if isinstance(config, MetaMap) else {}

    name_to_text = {k: stringify(v) for k, v in lua_pairs(opt["name_to_text"], "name_to_text").items()} if "name_to_text" in opt else {}
    # one unit for every environment, or a map of them by name
    parent_counters: dict[str, str] = {}
    parent_counter = None
    match opt.get("parent_counter"):
        case MetaMap(content):
            parent_counters = {k: stringify(v) for k, v in content.items()}
        case None:
            pass
        case other:
            parent_counter = stringify(other)

    theorems: list[Theorem] = []
    by_class: dict[str, Theorem] = {}

    def add(t: Theorem) -> None:
        theorems.append(t)
        by_class[t.class_name] = t

    styles, user_styles = read_styles(opt.get("styles"))
    for style in ["plain", "definition", "remark", *(s for s in user_styles if s not in BUILTIN_STYLES)]:
        for entry in as_list(opt.get(style)):
            if isinstance(entry, MetaMap):
                # a name, and the names sharing its counter (YAML's keys come
                # sorted, and so are they here)
                for key in sorted(entry.content):
                    add(Theorem(style, key, name_to_text.get(key, ""), parent_counters.get(key, parent_counter)))
                    value = entry.content[key]
                    for v in value.content if isinstance(value, MetaList) else [value]:
                        name = stringify(v)
                        add(Theorem(style, name, name_to_text.get(name, ""), shared_counter=key))
            else:
                name = stringify(entry)
                add(Theorem(style, name, name_to_text.get(name, ""), parent_counters.get(name, parent_counter)))
    proof = Proof(name_to_text.get("proof"))
    add(proof)

    top = top_level_division(meta, writer)
    # with parts at level 1: a part steps no other counter, and is in a
    # theorem's number only when theorems are numbered within parts
    part_level = 1 if top == "part" else None
    explicit = lua_number(stringify(opt["counter_depth"])) if opt.get("counter_depth") else None
    counters: dict[str, Counter] = {}
    counter_depth = explicit if explicit is not None else 0
    for t in theorems:
        if t.numbered and t.shared_counter is None:
            c = Counter(depth=explicit if explicit is not None else 0)
            if explicit is None and t.parent_counter:
                # within the heading level that LaTeX would number within
                c.depth = max(0, PARENT_COUNTERS[t.parent_counter] - PARENT_COUNTERS[top] + 1)
                if part_level and t.parent_counter != "part":
                    c.first = 2
            counters[t.counter_name] = c
            counter_depth = max(counter_depth, c.depth)
    for name in parent_counters:
        t = by_class.get(name.replace(" ", "_"))
        if t and t.shared_counter:
            warn(f"{name} shares the counter of {t.shared_counter}, so its parent_counter is {t.shared_counter}'s; ignoring the one given")

    ignored = {stringify(e) for e in as_list(opt.get("counter_ignore_headings"))} if opt.get("counter_ignore_headings") else set()
    # where pandoc starts numbering sections (--number-offset)
    header_counters = {i: (writer.number_offset[i - 1] if i <= len(writer.number_offset) else 0) for i in range(1, math.floor(counter_depth) + 1)}

    # the end-of-proof symbol as TeX math: the text of its math, raw TeX and
    # words
    qed_symbol = "\\Box"
    if "qed_symbol" in opt:
        parts: list[str] = []
        collect = Filter()
        collect.on(Math)(lambda m: parts.append(m.text))
        collect.on(RawInline)(lambda r: parts.append(r.text))
        collect.on(Str)(lambda s: parts.append(s.text))
        Para(*inlines_of(opt["qed_symbol"])).walk(collect)
        if parts:
            qed_symbol = " ".join(parts)

    return Options(
        css=stringify(opt["css"]) != "false" if "css" in opt else True,
        styles=styles,
        user_styles=user_styles,
        swapnumbers="swapnumbers" in opt and stringify(opt["swapnumbers"]) == "true",
        qed_symbol=qed_symbol,
        counters=counters,
        numbered_depth=numbered_depth(meta, writer, top),
        part_level=part_level,
        proof_name=name_to_text.get("proof"),
        theorems=theorems,
        by_class=by_class,
        counter_depth=counter_depth,
        ignored_headings=ignored,
        header_counters=header_counters,
    )


def options_latex(options: Options) -> str:
    lines = []
    if options.qed_symbol != "\\Box":
        lines.append(f"\\renewcommand{{\\qedsymbol}}{{\\ensuremath{{{options.qed_symbol}}}}}")
    # \newtheoremstyle{name}{above}{below}{body font}{indent}{heading font}
    #   {punctuation}{space after the heading}{heading layout}
    for name in options.user_styles:
        s = options.styles[name]

        def fonts(font: list[str]) -> str:
            # (\normalfont for none: an empty argument is the font around it)
            return "".join(FONT_LATEX[f] for f in font) or "\\normalfont"

        space = "\\newline" if s.headspace == "newline" else s.headspace
        lines.append(f"\\newtheoremstyle{{{name}}}{{{s.above}}}{{{s.below}}}{{{fonts(s.bodyfont)}}}{{{s.indent}}}{{{fonts(s.headfont)}}}{{{s.headpunct}}}{{{space}}}{{}}")
    if options.swapnumbers:
        lines.append("\\swapnumbers")  # before \newtheorem, which takes the order from it
    if options.proof_name is not None:
        # at the start of the document, after babel names the proof
        lines.append(f"\\AtBeginDocument{{\\renewcommand{{\\proofname}}{{{options.proof_name}}}}}")
    style = ""
    for t in options.theorems:
        if not isinstance(t, Proof):
            if t.style != style:
                style = t.style
                lines.append(f"\\theoremstyle{{{style}}}")
            lines.append(t.latex())
    return "\n".join(lines)


def add_header_include(meta: Meta, block: Block, at_start: bool = False) -> None:
    """A block in header-includes, whatever shape it has."""
    existing = items(meta.get("header-includes"))
    meta["header-includes"] = MetaList(*([MetaBlocks(block), *existing] if at_start else [*existing, MetaBlocks(block)]))


# ---------------------------------------------------------------------------
# The two passes


def find_theorem(options: Options, classes: list[str], warn_two: bool = False) -> Theorem | None:
    """The environment a div is, by its classes; none for one naming two."""
    found, others = None, []
    for c in classes:
        t = options.by_class.get(c)
        if t is not None and t is not found:
            if found is None:
                found = t
            elif c not in others:
                others.append(c)
    if others:
        if warn_two and found is not None:
            warn(f"multiple environments found: {found.class_name}, {', '.join(others)}; skipping")
        return None
    return found


def style_body(div: Div, options: Options, bodyfont: list[str]) -> None:
    """The div's content in its style's body font; environments in it are
    held out, as each has a style of its own."""
    held: list[Div] = []
    hold = Filter(traverse="topdown")

    @hold.on(Div)
    def out(d: Div, ctx: Context) -> Div | None:
        if find_theorem(options, d.classes):
            held.append(d)
            ctx.skip_children()
            return Div(attributes={"amsthm-held": str(len(held))})
        return None

    div.content.walk(hold)
    for font in bodyfont:
        div.content.walk(wrap_strs(FONT_TYPES[font]))
    if held:
        restore = Filter()

        @restore.on(Div)
        def back(d: Div) -> Div | None:
            # (any div with the attribute, the user's too: a copy each, as
            # each return is in Lua)
            i = lua_number(d.attributes.get("amsthm-held", ""))
            return copy.deepcopy(held[int(i) - 1]) if i is not None and i == int(i) and 1 <= i <= len(held) else None

        div.content.walk(restore)


def qed_math(options: Options) -> str:
    """amsthm's \\mathqed, \\quad\\qedsymbol (a space between where the
    symbol starts with a letter)."""
    symbol = options.qed_symbol
    return "\\quad" + (" " if re.match("[A-Za-z]", symbol) else "") + symbol


def qed_span(options: Options) -> Span:
    return Span(Math("InlineMath", qed_math(options)), classes=["amsthm-qed"])


def place_qedhere(div: Div, options: Options) -> bool:
    """amsthm's \\qedhere: the symbol where it is, \\mathqed in math, \\qed
    (after the spaces before it) in text; environments in this one keep
    theirs. Whether there was one."""
    placed = False
    f = Filter(traverse="topdown")

    @f.on(Div)
    def own(d: Div, ctx: Context) -> None:
        if find_theorem(options, d.classes):
            ctx.skip_children()

    @f.on(Math)
    def in_math(m: Math) -> None:
        nonlocal placed
        text, n = re.subn(r"\\qedhere(?![A-Za-z])", lambda _: qed_math(options), m.text)
        if n:
            placed = True
            m.text = text

    @f.on_inlines
    def in_text(xs: list[Inline]) -> list[Inline] | None:
        nonlocal placed
        out: list[Inline] = []
        changed = False
        for x in xs:
            if isinstance(x, RawInline) and x.format in ("tex", "latex") and re.fullmatch(r"[ \t\n\v\f\r]*\\qedhere[ \t\n\v\f\r]*", x.text):
                while out and isinstance(out[-1], BLANK):  # (\qed starts with \unskip)
                    out.pop()
                out.append(qed_span(options))
                placed = changed = True
            else:
                out.append(x)
        return out if changed else None

    div.content.walk(f)
    return placed


def with_inlines(block: Block | None) -> list[Inline] | None:
    """The inlines of a paragraph, plain text or heading."""
    return block.content if isinstance(block, (Para, Plain, Header)) else None


def environment(div: Div, options: Options) -> Div | None:
    """Not LaTeX: the heading, the body's font, the end of a proof."""
    theorem = find_theorem(options, div.classes, True)
    if theorem is None:
        return None
    identifier = div.identifier
    if not theorem.numbered and identifier:
        options.unnumbered.add(identifier)
    *title, space = theorem.header(options, identifier, div.attributes.get("info"))
    header = [Span(*title, classes=["amsthm-title"]), space]

    style = options.styles.get(theorem.style)
    if style and style.bodyfont:
        style_body(div, options, style.bodyfont)
    if (first := with_inlines(div.content[0] if div.content else None)) is not None:
        first[:0] = header
    else:
        div.content.insert(0, Para(*header))

    if isinstance(theorem, Proof) and not place_qedhere(div, options):
        # amsthm's \qed, \nobreak\hfill\quad\openbox: as math, \quad\Box
        # (the \hfill is the CSS's)
        if (last := with_inlines(div.content[-1] if div.content else None)) is not None:
            last.append(qed_span(options))
        else:
            div.content.append(Para(qed_span(options)))
    div.classes.append("amsthm")
    if f"amsthm-{theorem.style}" not in div.classes:
        div.classes.append(f"amsthm-{theorem.style}")
    return div


def reference_target(identifier: str, known: dict[str, str] | set[str]) -> tuple[str, bool] | None:
    """The environment a citation's id names, and whether to name it:
    `@Euler` is `@euler`, the environment's name before the number."""
    if identifier in known:
        return identifier, False
    lower = re.sub("^[A-Z]", lambda m: m[0].lower(), identifier)
    if lower != identifier and lower in known:
        return lower, True
    return None


def multi_ref(cite: Inline, known: dict[str, str]) -> list[str] | None:
    """The ids of `[@a; @b]`, each an environment's, none with a prefix or
    suffix (which the list would drop); else None, for citeproc."""
    if not isinstance(cite, Cite) or len(cite.citations) < 2:
        return None
    for c in cite.citations:
        if c.mode != CitationMode.NormalCitation or c.id not in known or c.prefix or c.suffix:
            return None
    return [c.id for c in cite.citations]


def ref_list(ids: list[str], one) -> list[Inline]:
    out: list[Inline] = []
    for i, identifier in enumerate(ids):
        if i:
            out += [Str(","), Space()]
        out += one(identifier)
    return out


def cite_id_mode(cite: Cite) -> tuple[str, CitationMode] | tuple[None, None]:
    if len(cite.citations) != 1:
        return None, None
    return cite.citations[0].id, cite.citations[0].mode


def warn_unnumbered(x: Cite | RawInline, options: Options) -> None:
    """Once per id, a reference to an unnumbered environment; and a citation
    of several that is left to citeproc."""
    ids: list[str] = []
    if isinstance(x, Cite):
        ids = [c.id for c in x.citations]
    elif x.format == "tex" and (m := re.fullmatch(r"\\([A-Za-z]+)\{(.*)\}", x.text, re.S)) and m[1] in ("ref", "eqref"):
        ids = [m[2]]
    for identifier in ids:
        target = reference_target(identifier, options.unnumbered)
        if target and target[0] not in options.warned:
            options.warned.add(target[0])
            warn(f"reference to the unnumbered environment {target[0]}; LaTeX prints the last number before it, other output leaves it unresolved")
    if isinstance(x, Cite) and len(x.citations) > 1:
        ours = others = named_ = affixed = False
        for c in x.citations:
            affixed = affixed or bool(c.prefix or c.suffix)
            if c.id in options.identifiers or c.id in options.unnumbered:
                ours = True
            elif reference_target(c.id, options.identifiers):
                named_ = True
            else:
                others = True
        if named_:
            warn(f"a reference to several environments cannot name them, and is left to citeproc: {stringify(x)}")
        elif ours and others:
            warn(f"a citation mixes environments with other keys, and is left to citeproc: {stringify(x)}")
        elif ours and affixed:
            warn(f"a reference to several environments cannot carry a prefix or suffix, and is left to citeproc: {stringify(x)}")


def ref_link(identifier: str, number: str, paren: bool, name: str | None = None) -> list[Inline]:
    """The number, a link to the environment, perhaps led by its name, in
    parentheses with `paren`."""
    link = Link(Str(number), url="#" + identifier)
    out: list[Inline] = named(name, link) if name else [link]
    return [Str("("), *out, Str(")")] if paren else out


def resolve(x: Cite | RawInline, options: Options) -> list[Inline] | None:
    """Not LaTeX: `[@id]`, `@id`, `\\ref{id}`, `\\eqref{id}` to numbers."""
    known = options.identifiers
    if (ids := multi_ref(x, known)) is not None:
        return ref_list(ids, lambda i: ref_link(i, known[i], True))
    if isinstance(x, Cite):
        identifier, mode = cite_id_mode(x)
        target = reference_target(identifier, known) if identifier is not None else None
        if target:
            identifier, is_named = target
            name = options.names.get(identifier) if is_named else None
            if mode in (CitationMode.NormalCitation, CitationMode.AuthorInText):
                return ref_link(identifier, known[identifier], mode == CitationMode.NormalCitation, name)
    elif x.format == "tex" and (m := re.fullmatch(r"\\([A-Za-z]+)\{(.*)\}", x.text, re.S)):
        if m[1] in ("ref", "eqref") and m[2] in known:
            return ref_link(m[2], known[m[2]], m[1] == "eqref")
    return None


def cite_to_ref(cite: Cite, options: Options) -> list[Inline] | None:
    """LaTeX: a citation of an environment to `\\ref` or `\\eqref`, perhaps
    led by its name (`Theorem~\\ref{euler}`)."""
    known = options.identifiers
    if (ids := multi_ref(cite, known)) is not None:
        return ref_list(ids, lambda i: [RawInline("latex", f"\\eqref{{{i}}}")])
    identifier, mode = cite_id_mode(cite)
    if identifier is None or mode not in (CitationMode.NormalCitation, CitationMode.AuthorInText):
        return None
    paren = mode == CitationMode.NormalCitation
    target = reference_target(identifier, known)
    if target is None:
        return None
    identifier, is_named = target
    name = options.names.get(identifier) if is_named else None
    if not name:
        return [RawInline("latex", f"\\{'eqref' if paren else 'ref'}{{{identifier}}}")]
    out = named(name, RawInline("latex", f"\\ref{{{identifier}}}"))
    return [Str("("), *out, Str(")")] if paren else out


def latex_environment(div: Div, options: Options) -> Div | None:
    """LaTeX: the div's own blocks between `\\begin{...}` and `\\end{...}`,
    sharing a line with the first and last paragraphs, in a bare div."""
    theorem = find_theorem(options, div.classes)
    if theorem is None:
        return None
    opening: list[Inline] = [RawInline("latex", f"\\begin{{{theorem.env_name}}}")]
    if info := div.attributes.get("info"):
        # (the original braces a note holding math, a citation, an image or
        # raw TeX, where a ] would end it: a note of text has none)
        opening += [RawInline("latex", "["), *note_inlines(info), RawInline("latex", "]")]
    if div.identifier:
        opening.append(RawInline("latex", f"\\label{{{div.identifier}}}"))
    closing = RawInline("latex", f"\\end{{{theorem.env_name}}}")
    content = div.content
    if content and isinstance(content[0], (Para, Plain)):
        content[0].content[:0] = [*opening, RawInline("latex", "\n")]
    else:
        content.insert(0, Plain(*opening))
    if isinstance(content[-1], (Para, Plain)):
        content[-1].content += [RawInline("latex", "\n"), closing]
    else:
        content.append(Plain(closing))
    return Div(*content)


@dataclass
class State:
    options: Options | None = None
    latex: bool = False


state = State()
first, second = Filter(traverse="topdown"), Filter(traverse="topdown")


@first.on(Pandoc)
def configure(doc: Pandoc, ctx: Context) -> None:
    state.latex = ctx.format in LATEX_LIKE
    options = state.options = read_options(doc.meta, Writer.of(ctx.conversion.options))
    if any(c.depth >= c.first and options.numbered_depth < c.first for c in options.counters.values()):
        warn("theorems are numbered within sections that are not numbered, so their numbers start with 0 as in LaTeX; pass -N (--number-sections)")
    if state.latex:
        # amsthm first, for the user's own \newtheoremstyle; the
        # environments last, to use those styles
        add_header_include(doc.meta, RawBlock("latex", "\\usepackage{amsthm}"), at_start=True)
        add_header_include(doc.meta, RawBlock("latex", options_latex(options)))
    elif ctx.format in HTML_LIKE and options.css:
        add_header_include(doc.meta, RawBlock("html", CSS))


@first.on(Header)
def count(h: Header) -> None:
    """\\section* and the like step no counter, nor do headings LaTeX
    doesn't number."""
    o = state.options
    assert o is not None
    if state.latex or h.level > o.counter_depth or h.level > o.numbered_depth or "unnumbered" in h.classes:
        return
    if stringify(h) in o.ignored_headings:
        return
    o.header_counters[h.level] = o.header_counters.get(h.level, 0) + 1
    if h.level != o.part_level:
        for level in range(h.level + 1, math.floor(o.counter_depth) + 1):
            o.header_counters[level] = 0
    # restart each counter numbered within this level or above it
    for name, c in o.counters.items():
        if c.first <= h.level <= c.depth:
            o.theorem_counters.pop(name, None)


@first.on(Div)
def first_div(d: Div) -> Div | None:
    assert state.options is not None
    if state.latex:
        # the ids, for the references before their environments
        theorem = find_theorem(state.options, d.classes, True)
        if theorem is not None and d.identifier:
            state.options.identifiers[d.identifier] = ""
            state.options.names[d.identifier] = theorem.text
            if not theorem.numbered:
                state.options.unnumbered.add(d.identifier)
        return None
    return environment(d, state.options)


@second.on(Div)
def second_div(d: Div) -> Div | None:
    assert state.options is not None
    return latex_environment(d, state.options) if state.latex else None


@second.on(Cite)
def cite(c: Cite) -> list[Inline] | None:
    assert state.options is not None
    warn_unnumbered(c, state.options)
    return cite_to_ref(c, state.options) if state.latex else resolve(c, state.options)


@second.on(RawInline)
def raw(r: RawInline) -> list[Inline] | None:
    assert state.options is not None
    warn_unnumbered(r, state.options)
    return None if state.latex else resolve(r, state.options)


if __name__ == "__main__":
    main([first, second])
