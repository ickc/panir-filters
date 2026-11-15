#!/usr/bin/env python3
"""Normalise the authors and institutes of a document's metadata: a pandoc
filter.

`author` and `institute` become lists of maps, each with a `name` and an
`id`; an author's `institute` becomes the numbers of its institutes in
`institute` (each given an `index`), which collects the top-level ones
and those given in an author. An author or institute may be a name, a map
with a `name`, or a map of one entry, its key the name and its value the
rest (or the name); text with commas is a list of names; an author refers
to an institute by identifier, by name or by number.

A port of pandoc/lua-filters' scholarly-metadata (Copyright (c) 2017-2021
Albert Krewinkel, Robert Winkler; ISC; the repository's MIT licence, see
../LICENSE).
"""

import copy
import re

from panir import (
    Filter,
    Meta,
    MetaBool,
    MetaInlines,
    MetaList,
    MetaMap,
    MetaString,
    MetaValue,
    Str,
    items,
    stringify,
)

# A named object, as the original's Lua table: plain while it is worked on.
Named = dict[str, MetaValue]

LUA_SPACE = " \t\n\v\f\r"  # Lua's %s (and its tonumber's)

f = Filter()


def number(text: str) -> float | None:
    """A number as Lua's `tonumber` reads a string (decimal only), else None."""
    if re.fullmatch(r"[ \t\n\v\f\r]*[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?[ \t\n\v\f\r]*", text):
        return float(text)
    return None


def as_list(value: MetaValue | None) -> list[MetaValue]:
    """A list as the original takes one: a list's items, text with commas
    split there (each a string), anything else alone (nothing if missing)."""
    if isinstance(value, MetaInlines):
        names = [name.strip(LUA_SPACE) for name in stringify(value).split(",")]
        if len(names) >= 2:
            return [MetaString(name) for name in names]
    return items(value)


def named(obj: MetaValue) -> Named:
    """An author or institute as a map with a name and an identifier."""
    match obj:
        case MetaInlines():
            return {"name": obj, "id": MetaString(stringify(obj))}
        case MetaString() | MetaBool():
            text = stringify(obj)  # (Lua's tostring)
            return {"name": MetaInlines(Str(text)), "id": MetaString(text)}
        case MetaMap(entries) if "name" in entries:
            out = dict(entries)
            out["id"] = MetaString(stringify(out.get("id") or out["name"]))
            return out
        case MetaMap(entries) if len(entries) == 1:
            ((key, value),) = entries.items()
            out: Named
            match value:
                case MetaString() | MetaInlines():
                    out = {"name": value}
                case MetaMap(more):
                    out = dict(more)
                    if not out.get("name"):
                        out["name"] = MetaInlines(Str(key))
                case _:
                    raise ValueError(f"scholarly-metadata: {key}: neither a name nor a map")
            out["id"] = MetaString(stringify(out["id"]) if out.get("id") else key)
            return out
    # a map of none or several entries; a list or blocks, which the original
    # names after Lua's tostring of them (NOTES.md)
    raise ValueError(f"scholarly-metadata: not a named object: {stringify(obj)}")


def ident(obj: Named) -> str:
    return stringify(obj["id"])  # (a string's text)


def find(institutes: list[Named], key: str) -> Named | None:
    """The first institute with that identifier."""
    return next((x for x in institutes if ident(x) == key), None)


def resolve(refs: list[MetaValue], institutes: list[Named]) -> list[Named]:
    """An author's institutes: by number (a string's), by identifier, or new."""
    out = []
    for ref in refs:
        n = number(ref.text) if isinstance(ref, MetaString) else None
        if n is not None and n.is_integer() and 1 <= n <= len(institutes):
            out.append(institutes[int(n) - 1])
        else:
            out.append(find(institutes, stringify(ref)) or named(ref))
    return out


def merge(institutes: list[Named], inst: Named) -> None:
    """Add an institute, or what it has the one of its identifier lacks."""
    same = find(institutes, ident(inst))
    if same is None:
        institutes.append(inst)
        return
    for key, value in inst.items():
        if not same.get(key):
            same[key] = copy.deepcopy(value)  # (inst may stay too: a second top-level one)


@f.on_meta
def canonicalize(meta: Meta) -> None:
    institutes = [named(x) for x in as_list(meta.get("institute"))]
    authors = [named(x) for x in as_list(meta.get("author"))]
    theirs = [resolve(as_list(author.get("institute")), institutes) for author in authors]
    for inst in (inst for insts in theirs for inst in insts):
        merge(institutes, inst)
    numbers: dict[str, str] = {}  # each identifier's first institute
    for i, inst in enumerate(institutes, 1):
        inst["index"] = MetaInlines(Str(str(i)))
        numbers.setdefault(ident(inst), str(i))
    for author, insts in zip(authors, theirs):
        author["institute"] = MetaList(*(MetaString(numbers[ident(x)]) for x in insts))
    meta["author"] = authors
    meta["institute"] = institutes


if __name__ == "__main__":
    f.main()
