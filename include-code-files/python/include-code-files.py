#!/usr/bin/env python3
"""A code block with an `include` attribute takes the file's text: a pandoc
filter.

`startLine` and `endLine` (or `start-line`, `end-line`) take a range of
lines, the first giving `startFrom` (for `numberLines`); `dedent=n` removes
the spaces among each line's first n bytes. The attributes it reads are
removed. A file that can't be opened leaves the block empty, with a
message.

A port of pandoc/lua-filters' include-code-files (© 2020 Bruno BEAUFILS;
MIT, see ../LICENSE).
"""

import re
import sys

from panir import CodeBlock, Filter, main


def lua_number(text: str) -> float | None:
    """A number as Lua's `tonumber` reads text: decimal or hexadecimal,
    blanks around it; None if not one."""
    text = text.strip(" \t\n\v\f\r")
    if re.fullmatch(r"[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?", text):
        return int(text) if text.lstrip("+-").isdigit() else float(text)
    if re.fullmatch(r"[+-]?0[xX][0-9a-fA-F]+", text):
        return int(text, 16)
    return None


def lua_integer(text: str, what: str) -> int:
    """Text as Lua's string functions take it for an integer argument."""
    n = lua_number(text)
    if n is None:
        raise TypeError(f"bad argument to '{what}' (number expected, got string)")
    if n != int(n):
        raise ValueError(f"bad argument to '{what}' (number has no integer representation)")
    return int(n)


def lua_sub(s: bytes, i: int, j: int = -1) -> bytes:
    """Lua's `s:sub(i, j)`: bytes i to j, counted from 1, negative from the
    end."""
    n = len(s)
    i = max(n + i + 1 if i < 0 else i, 1)
    j = min(n + j + 1 if j < 0 else j, n)
    return s[i - 1 : j] if i <= j else b""


def dedent(line: bytes, n: str) -> bytes:
    """The spaces among the line's first n bytes removed (Lua's strings are
    bytes: an é is two)."""
    k = lua_integer(n, "sub")
    return lua_sub(line, 1, k).replace(b" ", b"") + lua_sub(line, k + 1)


def lines(data: bytes) -> list[bytes]:
    """Lua's `fh:lines("L")`: each line with its newline (`\\n`)."""
    parts = data.split(b"\n")
    return [p + b"\n" for p in parts[:-1]] + ([parts[-1]] if parts[-1] else [])


def need(n: float | None) -> float:
    if n is None:
        raise TypeError("attempt to compare nil with number")
    return n


f = Filter()


@f.on(CodeBlock)
def transclude(cb: CodeBlock) -> None:
    attrs = cb.attributes
    path = attrs.get("include")
    if path is None:
        return
    content = b""
    try:
        with open(path, "rb") as fh:
            data = fh.read()
    except IsADirectoryError:
        raise  # (Lua opens a directory, then fails to read it)
    except OSError:
        print(f"Cannot open file {path} | Skipping includes", file=sys.stderr)
    else:
        # hyphenated attributes to camel case
        for camel, hyphen in (("startLine", "start-line"), ("endLine", "end-line")):
            if (value := attrs.get(hyphen)) is not None:
                attrs[camel] = value
                del attrs[hyphen]
        start: float | None = 1
        if (first := attrs.get("startLine")) is not None:
            attrs["startFrom"] = first
            start = lua_number(first)
        for number, line in enumerate(lines(data), 1):
            if (n := attrs.get("dedent")) is not None:
                line = dedent(line, n)
            if number >= need(start):
                last = attrs.get("endLine")
                if last is None or number <= need(lua_number(last)):
                    content += line
    for key in ("include", "startLine", "endLine", "dedent"):
        attrs.pop(key, None)
    cb.text = content.decode(errors="replace")


if __name__ == "__main__":
    main(f)
