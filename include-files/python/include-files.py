#!/usr/bin/env python3
"""Code blocks of class `include` replaced by the files they name: a pandoc
filter.

Each line of the block names a file (`//` starts a comment); each is read
by pandoc, as markdown or as the block's `format`, with the document's
reader options, and its blocks (not its metadata) take the block's place.
Headings shift by `shift-heading-level-by`, or, with `include-auto` in the
metadata, by the level of the last heading before the block. Included
files may include others, named relative to their own directory; relative
links, images and `include` attributes (include-code-files') in an
included file are rewritten relative to the including one.

Reading the files is Python's; parsing them is pandoc's (`pandoc.read` in
the original): `libpandoc.read`, in this process: the pandoc the filter
runs in under pandocpy or `libpandoc.convert`, or libpandoc as a library
when pandoc runs this as a JSON filter. Without libpandoc it fails, saying
so.

A port of pandoc-ext/include-files (© 2019–2021 Albert Krewinkel; MIT,
see ../LICENSE).
"""

import os
import re
import sys

from panir import Block, CodeBlock, Context, Filter, Header, Image, Link, Meta, main
from panir import lua_number

try:
    import libpandoc
except ImportError:
    sys.exit("include-files: needs pandoc's reader, libpandoc (pip install libpandoc), to read the files it includes")


# pandoc.path's functions, as pandoc has them (Haskell's System.FilePath,
# POSIX): `normalize` drops `.` and doubled slashes but keeps `..`, unlike
# os.path.normpath.


def is_relative(path: str) -> bool:
    return not path.startswith("/")


def filename(path: str) -> str:
    return path.rpartition("/")[2]


def directory(path: str) -> str:
    head = path.rpartition("/")[0] + "/" if "/" in path else ""
    stripped = head.rstrip("/")
    if not stripped and head:
        return "/"  # (the root)
    return stripped or ("." if path else "")


def join(a: str, b: str) -> str:
    if b.startswith("/") or not a:
        return b
    if not b:
        return a
    return a + b if a.endswith("/") else f"{a}/{b}"


def normalize(path: str) -> str:
    root = path.startswith("/")
    rest = path.lstrip("/")
    parts = [p for p in rest.split("/") if p and p != "."]
    out = ("/" if root else "") + "/".join(parts)
    if not out:
        return "."
    is_dir = rest.endswith("/") or (rest.endswith(".") and rest[:-1].endswith("/"))
    return out + "/" if is_dir and not out.endswith("/") else out


class State:
    auto = False  # (include-auto)
    last_level = 0  # the level of the last heading met


state = State()
configure = Filter()


@configure.on_meta
def get_vars(meta: Meta) -> None:
    # (for each document: a host may run this filter on many, where Lua
    # loads the original anew for each)
    state.auto = bool(meta.get("include-auto"))
    state.last_level = 0


def update_last_level(header: Header) -> None:
    state.last_level = header.level


def update_contents(blocks: list[Block], shift_by: float | None, include_path: str) -> list[Block]:
    """Headings shifted; relative paths taken to the including file's
    directory."""
    update = Filter()

    @update.on(Header)
    def shift(header: Header) -> None:
        if shift_by is not None:
            level = header.level + shift_by
            if level != int(level):  # (Lua's error, as pandoc's Header takes an integer)
                raise ValueError(f"shift-heading-level-by: {shift_by}: number has no integer representation")
            header.level = int(level)

    @update.on(Link)
    def link(link: Link) -> None:
        target = link.url
        if is_relative(target) and not filename(target).startswith("#") and not re.search(r"https?://", target):
            link.url = normalize(join(include_path, target))

    @update.on(Image)
    def image(image: Image) -> None:
        if is_relative(image.url):
            image.url = normalize(join(include_path, image.url))

    @update.on(CodeBlock)
    def code(cb: CodeBlock) -> None:
        path = cb.attributes.get("include")
        if path is not None and is_relative(path):
            cb.attributes["include"] = normalize(join(include_path, path))

    return update(blocks)  # (a list of blocks, walked in place)


def transclusion(base: str) -> Filter:
    """The filter including files, for blocks of a file in `base` (the
    original changes the working directory to it: here the names are
    joined to it, as the working directory is the process's)."""
    f = Filter()
    f.on(Header)(update_last_level)

    @f.on(CodeBlock)
    def transclude(cb: CodeBlock, ctx: Context) -> list[Block] | None:
        if "include" not in cb.classes:
            return None
        fmt = cb.attributes.get("format") or "markdown"
        shift_input = cb.attributes.get("shift-heading-level-by")
        if shift_input is not None:
            shift_by = lua_number(shift_input)
        else:
            shift_by = state.last_level if state.auto else 0
        before = state.last_level
        blocks: list[Block] = []
        for line in cb.text.split("\n"):
            if not line or line.startswith("//"):
                continue
            path = join(base, line)
            try:
                with open(path, "rb") as fh:
                    text = fh.read()
            except IsADirectoryError:
                raise  # (Lua opens a directory, and pandoc fails to read nothing)
            except OSError:
                print(f"Cannot open file {line} | Skipping includes", file=sys.stderr)
                continue
            contents = libpandoc.read(text, options=libpandoc.read_options(ctx.conversion, fmt)).blocks
            state.last_level = 0
            # (the walk of the included blocks knows this one's conversion)
            contents.walk(transclusion(join(base, directory(line))))
            state.last_level = before
            blocks += update_contents(contents, shift_by, directory(line))
        return blocks

    return f


if __name__ == "__main__":
    main([configure, transclusion("")])
