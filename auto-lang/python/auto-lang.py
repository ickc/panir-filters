#!/usr/bin/env python3
"""Spans with `lang` (and `dir`) around runs of a mapped Unicode script: a
pandoc filter.

The metadata maps scripts to languages (`auto-lang: {Hebrew: he, Han:
zh-Hant}`); each run of text in a mapped script becomes a span with the
language, and `dir=rtl` for a right-to-left script. Spaces, digits and
punctuation join a run that goes on after them; a character that
Script_Extensions ties to the run's script (`。` to Han) joins a run it
touches; a combining mark goes with the character before it. A span that
has a `lang` already is left alone, and what is in it.

A port of ickc/font's bin/auto-lang.lua (Copyright (c) 2026, Kolen Cheung;
BSD-3-Clause, see ../LICENSE), with its table of Unicode 15.0.0's scripts
(`script_ranges.py`, made by ../tables.py from the original's).
"""

import sys
from bisect import bisect_right
from dataclasses import dataclass
from enum import Enum
from functools import cache
from typing import TypeGuard

from panir import (
    Context,
    Emph,
    Filter,
    Inline,
    LineBreak,
    Link,
    Meta,
    MetaBlocks,
    MetaBool,
    MetaInlines,
    MetaList,
    MetaMap,
    Quoted,
    SmallCaps,
    SoftBreak,
    Space,
    Span,
    Str,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Underline,
    main,
    stringify,
)
from script_ranges import EXT, INHERITED, NAMES, RTL, STRONG, VERSION

languages: dict[str, str] = {}  # a mapped script's language

# Containers that carry nothing of their own: what is in them decides the run
# they belong to (a span with a `lang` aside). A note's content is a flow of
# its own; code, math and raw inlines are not prose.
Container = Emph | Link | Quoted | SmallCaps | Span | Strikeout | Strong | Subscript | Superscript | Underline


def transparent(x: Inline) -> TypeGuard[Container]:
    return isinstance(x, Container) and not (isinstance(x, Span) and "lang" in x.attributes)


def within(ranges: tuple[tuple[int, int], ...], cp: int) -> bool:
    i = bisect_right(ranges, (cp, sys.maxunicode + 1))
    return i > 0 and cp <= ranges[i - 1][1]


STRONG_FIRSTS = [first for first, _, _ in STRONG]


def own_script(cp: int) -> str | None:
    """The script of a code point's own Script property, if a script."""
    i = bisect_right(STRONG_FIRSTS, cp)
    if i and cp <= STRONG[i - 1][1]:
        return STRONG[i - 1][2]
    return None


class Mark(Enum):
    INHERITED = "inherited"  # Script=Inherited: goes with what precedes it


@cache
def classify(cp: int) -> str | frozenset[str] | Mark | None:
    """A code point's script (its own, mapped or not), the mapped scripts
    its Script_Extensions tie it to, `Mark.INHERITED`, or None: neutral
    (spaces, digits, punctuation). Its own script first if mapped; then
    the ties, even of a code point of an unmapped script (the Arabic-Indic
    digits continue a Thaana run)."""
    own = own_script(cp)
    if own in languages:
        return own
    tied = frozenset(s for s in languages if within(EXT.get(s, ()), cp))
    if tied:
        return tied
    if own:
        return own
    if within(INHERITED, cp):
        return Mark.INHERITED
    return None


@dataclass
class Piece:
    text: str
    script: str | None = None  # a run of a mapped script
    tied: frozenset[str] | None = None
    foreign: bool = False  # another script's own text


def tied_to(script: str, ties: frozenset[str] | None) -> bool:
    return ties is not None and script in ties


def leading_tied(script: str, ties: list[frozenset[str] | None]) -> int:
    """How many of the pending entries' ties, from the start, are to `script`."""
    n = 0
    while n < len(ties) and tied_to(script, ties[n]):
        n += 1
    return n


def trailing_tied(script: str, ties: list[frozenset[str] | None]) -> int:
    """Where the pending entries tied to `script` at the end start."""
    i = len(ties)
    while i and tied_to(script, ties[i - 1]):
        i -= 1
    return i


def split(text: str) -> list[Piece]:
    """A text as maximal runs of one mapped script, and what is between
    them: neutral text joins a run only if the run goes on after it; text
    tied to a run's script joins the run it touches. Tied text stays in
    pieces of one character (with its marks): it may belong to a run in a
    neighbouring inline, which only `group` sees."""
    pieces: list[Piece] = []
    run: Piece | None = None
    pending: list[Piece] = []  # each character since the run last took one

    def close() -> None:
        nonlocal run
        if run is not None:
            n = leading_tied(run.script or "", [p.tied for p in pending])
            run.text += "".join(p.text for p in pending[:n])
            del pending[:n]
            pieces.append(run)
            run = None

    def drain() -> None:
        pieces.extend(pending)
        pending.clear()

    for ch in text:
        match classify(ord(ch)):
            case str(script) if script in languages:
                if run is not None and run.script == script:
                    run.text += "".join(p.text for p in pending) + ch
                    pending.clear()
                else:
                    close()
                    first = trailing_tied(script, [p.tied for p in pending])
                    carried = "".join(p.text for p in pending[first:])
                    del pending[first:]
                    drain()
                    run = Piece(carried + ch, script=script)
            case str():
                # another script's own text ends the run, apart from the
                # neutral text around it
                close()
                drain()
                pieces.append(Piece(ch, foreign=True))
            case Mark.INHERITED:
                # with its base: the last pending character, or the run
                if pending:
                    pending[-1].text += ch
                elif run is not None:
                    run.text += ch
                else:
                    pending.append(Piece(ch))
            case tied:
                pending.append(Piece(ch, tied=tied))
    close()
    drain()

    merged: list[Piece] = []
    for p in pieces:
        last = merged[-1] if merged else None
        if last and not (last.script or p.script or last.tied or p.tied) and last.foreign == p.foreign:
            last.text += p.text
        else:
            merged.append(p)
    return merged


def span(script: str, inlines: list[Inline]) -> Span:
    attributes = [("lang", languages[script])] + ([("dir", "rtl")] if script in RTL else [])
    return Span(*inlines, attributes=attributes)


def list_script(inlines: list[Inline]) -> tuple[str | None, frozenset[str] | None] | None:
    """Which run a list of inlines belongs to, from the text in it: its one
    mapped script, or none and, if all it holds is tied text, the scripts
    all of it is tied to; None if it is opaque: it holds another script, or
    what isn't prose, or a span with a language."""
    found: str | None = None
    ties: frozenset[str] | None = None
    untied = False

    def scan(items: list[Inline]) -> bool:
        nonlocal found, ties, untied
        for x in items:
            match x:
                case Str(text):
                    for ch in text:
                        match classify(ord(ch)):
                            case str(script):
                                if script not in languages or (found and found != script):
                                    return False
                                found = script
                            case frozenset() as tied:
                                ties = tied if ties is None else ties & tied
                            case None:
                                untied = True
                            # (a mark goes with its base: it neither ties nor unties)
                case Space() | SoftBreak() | LineBreak():
                    untied = True
                case _ if transparent(x):
                    if not scan(x.content):
                        return False
                case _:
                    return False
        return True

    if not scan(inlines):
        return None
    if found:
        return found, None
    return None, (None if untied or not ties else ties)


def group(inlines: list[Inline]) -> list[Inline] | None:
    """The list with its runs in spans, a run going on across spaces and
    the containers of nothing but its script; None if there is none."""
    # A run with the whole list to itself takes the neutral text around it.
    whole = list_script(inlines)
    if whole and whole[0]:
        return [span(whole[0], inlines)]

    out: list[Inline] = []
    run: list[Inline] = []
    script: str | None = None
    pending: list[tuple[Inline, frozenset[str] | None]] = []  # since the run last took one
    changed = False

    def flush() -> None:
        out.extend(x for x, _ in pending)
        pending.clear()

    def close() -> None:
        nonlocal script, changed
        if script is not None:
            n = leading_tied(script, [t for _, t in pending])
            run.extend(x for x, _ in pending[:n])
            del pending[:n]
            out.append(span(script, run))
            run.clear()
            script, changed = None, True
        flush()

    def add(x: Inline, of: str | None, tied: frozenset[str] | None) -> None:
        nonlocal script
        if of is None:
            pending.append((x, tied))
        elif of == script:
            run.extend(y for y, _ in pending)
            pending.clear()
            run.append(x)
        else:
            # the first run, or another script's: the tied text at the end of
            # what is pending is the new run's
            first = trailing_tied(of, [t for _, t in pending])
            carried = [y for y, _ in pending[first:]]
            del pending[first:]
            close()
            script = of
            run.extend(carried)
            run.append(x)

    for x in inlines:
        match x:
            case Space() | SoftBreak():
                add(x, None, None)
            case Str(text):
                for piece in split(text):
                    if piece.foreign:
                        close()
                        out.append(Str(piece.text))
                    else:
                        add(Str(piece.text), piece.script, piece.tied)
            case _ if transparent(x):
                found = list_script(x.content)
                if found is None:
                    close()
                    out.append(x)
                else:
                    add(x, *found)
            case _:
                close()
                out.append(x)
    close()
    return out if changed else None


configure, tag = Filter(), Filter(traverse="topdown")


@configure.on_meta
def read_map(meta: Meta) -> None:
    """`auto-lang`, the scripts' languages. In a separate filter, as a
    filter's metadata function runs after its inlines'."""
    match meta.get("auto-lang"):
        case None | MetaBool(False):
            scripts = {}
        case MetaMap(content):
            scripts = content
        case MetaList(content) | MetaInlines(content) | MetaBlocks(content):
            # (Lua's `pairs` over a list: its indices, none a script's name)
            scripts = {str(i): None for i in range(1, len(content) + 1)}
        case other:
            raise ValueError(f"auto-lang: the map of scripts to languages is {type(other).__name__}")
    for script, language in scripts.items():
        if script in NAMES and language is not None:
            languages[script] = stringify(language)
        else:
            print(f"[WARNING] auto-lang: no Unicode script named {script} (Unicode {VERSION})", file=sys.stderr)
    classify.cache_clear()


@tag.on(Span)
def keep(span: Span, ctx: Context) -> None:
    """A language written by hand wins: the span and what is in it stay."""
    if "lang" in span.attributes:
        ctx.skip_children()


@tag.on_inlines
def runs(inlines: list[Inline]) -> list[Inline] | None:
    if languages:
        return group(inlines)
    return None


if __name__ == "__main__":
    main([configure, tag])
