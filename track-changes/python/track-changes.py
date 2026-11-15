#!/usr/bin/env python3
"""Tracked changes (Word's, as pandoc reads them with `--track-changes=all`):
accepted, rejected, or shown: a pandoc filter.

The metadata's `trackChanges` (`accept`, the default, `reject` or `all`)
decides. Accepting keeps insertions and drops deletions and comments;
rejecting the reverse; `all` shows them in HTML (`ins`, `del`, `mark`) and
LaTeX (the `changes` package, in `header-includes`, with a colour per
author), and in Word and ODT keeps them, a comment's `¶` a line break.

A port of pandoc/lua-filters' track-changes (Copyright (c) 2017-2021
pandoc Lua filters contributors; MIT, see ../LICENSE).
"""

import re
from collections.abc import Callable

from panir import (
    Context,
    Filter,
    Inline,
    LineBreak,
    MetaBlocks,
    MetaString,
    MetaValue,
    Pandoc,
    RawBlock,
    RawInline,
    Span,
    Str,
    stringify,
)

HEADER = r"""
\makeatletter
\PassOptionsToPackage{textsize=scriptsize}{todonotes}
\PassOptionsToPackage{markup=underlined,authormarkup=none,commentmarkup=todo}{changes}
\usepackage{changes}
\@ifpackagelater{changes}{2018/11/03}{%
}{%
  \usepackage{todonotes}
  \setremarkmarkup{\todo[color=Changes@Color#1!20]{\sffamily\textbf{#1:}~#2}}
}%
\makeatother
\definecolor{auth1}{HTML}{4477AA}
\definecolor{auth2}{HTML}{117733}
\definecolor{auth3}{HTML}{999933}
\definecolor{auth4}{HTML}{CC6677}
\definecolor{auth5}{HTML}{AA4499}
\definecolor{auth6}{HTML}{332288}
\setlength{\marginparwidth}{3cm}
\newcommand{\note}[2][]{\added[#1,remark={#2}]{}}
\newcommand\hlnotesingle{%
  \bgroup
  \expandafter\def\csname sout\space\endcsname{\bgroup \ULdepth =-.8ex \ULset}%
  \markoverwith{\textcolor{yellow}{\rule[-.5ex]{.1pt}{2.5ex}}}%
  \ULon}
\newcommand\hlnote[1]{\let\helpcmd\hlnotesingle\parhelp#1\par\relax\relax}
\long\def\parhelp#1\par#2\relax{%
  \helpcmd{#1}\ifx\relax#2\else\par\parhelp#2\relax\fi%
}

\makeatletter
\newcommand\ifmoving{%
  \ifx\protect\@unexpandable@protect
      \expandafter\@firstoftwo
  \else
      \expandafter\@secondoftwo
  \fi
}

\newcommand{\gobbletwo}[2][]{\@bsphack\@esphack}
\newcommand{\gobbleone}[1][]{\@bsphack\@esphack}

\let\oldadded\added
\let\olddeleted\deleted
\let\oldhlnote\hlnote
\let\oldnote\note
\renewcommand{\added}{\ifmoving{\gobbleone}{\oldadded}}
\renewcommand{\deleted}{\ifmoving{\gobbletwo}{\olddeleted}}
\renewcommand{\hlnote}{\ifmoving{}{\oldhlnote}}
\renewcommand{\note}{\ifmoving{\gobbletwo}{\oldnote}}
\makeatother
"""

TEX = {"comment-start": "\\note", "insertion": "\\added", "deletion": "\\deleted"}
HTML = {"comment-start": "mark", "insertion": "ins", "deletion": "del"}
TITLES = {"dr", "mr", "ms", "mrs", "prof", "mx", "sir"}

authors: dict[str, str] = {}  # by initials, those of the changes shown in LaTeX


def initials(name: str) -> str:
    """A name's initials (ASCII letters and digits: Lua's %w), titles left
    out; a name of one word as it is."""
    if " " not in name:
        return name
    words = re.findall(r"[A-Za-z0-9']+", name)
    return "".join(w[0].upper() for w in words if w.lower() not in TITLES)


def pilcrows(line: Callable[[], Inline]) -> Filter:
    """A comment's `¶` (a paragraph break in Word) as `line()`."""
    lines = Filter()

    @lines.on(Str)
    def pilcrow(s: Str) -> Inline | None:
        return line() if s.text == "¶" else None

    return lines


def author(span: Span) -> str:
    """The author's initials, the author kept by them."""
    name = span.attributes["author"]  # (the original fails without one too)
    authors[initials(name)] = name
    return initials(name)


def to_tex(span: Span) -> Inline | None:
    kind = span.classes[0] if span.classes else ""
    if kind in TEX:
        s = f"{TEX[kind]}[id={author(span)}]{{"
        if "comment-start" in span.classes:
            s += stringify(span.walk(pilcrows(lambda: Str("\\newline")))) + "}\\hlnote{"
        else:
            s += stringify(span.content) + "}"
        return RawInline("latex", s)
    if "comment-end" in span.classes:
        return RawInline("latex", "}")
    return None


def to_html(span: Span) -> Inline | None:
    kind = span.classes[0] if span.classes else ""
    if kind in HTML:
        author(span)
        s = f"<{HTML[kind]}"
        for key, value in span.attributes:
            s += f' {key if key == "date" else "data-" + key}="{value}"'
        if "comment-start" in span.classes:
            s += f' data-id="{span.identifier}"'  # (Lua's "" is true: always)
            s += f' title="{stringify(span.walk(pilcrows(lambda: Str("&#10;"))))}">'
        else:
            s += f">{stringify(span.content)}</{HTML[kind]}>"
        return RawInline("html", s)
    if "comment-end" in span.classes:
        return RawInline("html", "</mark>")
    return None


def to_word(span: Span) -> Span | None:
    return span.walk(pilcrows(LineBreak)) if "comment-start" in span.classes else None


def resolve(span: Span, accept: bool) -> list[Inline] | None:
    """Accepting or rejecting: comments go, an insertion becomes its content
    (accepted) or goes, a deletion the reverse; a span of both is an
    insertion."""
    if "comment-start" in span.classes or "comment-end" in span.classes:
        return []
    if "insertion" in span.classes:
        return list(span.content) if accept else []
    if "deletion" in span.classes:
        return [] if accept else list(span.content)
    return None


def header_includes(existing: MetaValue | None) -> list[MetaValue]:
    """`header-includes` with the `changes` package and a colour for each
    author: the original wraps what is there in a list, a list too (its test
    for a list, of `.t`, fails since pandoc 2.17)."""
    out: list[MetaValue] = [existing] if existing is not None else []
    out.append(MetaBlocks(RawBlock("latex", HEADER)))
    for i, (inits, name) in enumerate(sorted(authors.items()), 1):
        definition = f"\\definechangesauthor[name={{{name}}}, color=auth{i}]{{{inits}}}"
        out.append(MetaBlocks(RawBlock("latex", definition)))
    return out


f = Filter()


@f.on(Pandoc)
def track_changes(doc: Pandoc, ctx: Context) -> None:
    # (the original reads the reader's option first, as pandoc 2 named it:
    # since pandoc 3 never there, so the metadata decides)
    mode = stringify(doc.meta.pop("trackChanges", MetaString("accept")))
    fmt = ctx.format
    changes = Filter()
    if mode == "all":
        if fmt in ("html", "html4", "html5"):
            changes.on(Span)(to_html)
        elif fmt in ("latex", "tex", "context"):
            changes.on(Span)(to_tex)
        elif fmt in ("docx", "odt"):
            changes.on(Span)(to_word)
    else:
        changes.on(Span)(lambda span: resolve(span, accept=mode != "reject"))
    doc.blocks.walk(changes)  # (not the metadata)
    if mode == "all" and fmt in ("latex", "tex", "context"):
        doc.meta["header-includes"] = header_includes(doc.meta.get("header-includes"))


if __name__ == "__main__":
    f.main()
