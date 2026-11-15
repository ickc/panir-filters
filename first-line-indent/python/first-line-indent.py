#!/usr/bin/env python3
"""First-line indents, removed after quotes, lists and the like: a pandoc
filter.

A paragraph right after a block quote, a list, a code block or a rule
(`first-line-indent.remove-after`, `dont-remove-after`) gets `\\noindent` in
LaTeX, or a marker div before it in HTML, unless it starts with
`\\indent` or `\\noindent` itself. The metadata's `indent` is set true, and
`header-includes` gets the CSS (HTML) or the indent's `size` (LaTeX).

A port of pandoc/lua-filters' first-line-indent (Copyright (c) 2021
Julien Dutant; MIT, see ../LICENSE).
"""

import re
from dataclasses import dataclass, field

from panir import (
    Block,
    Context,
    Filter,
    Meta,
    MetaBlocks,
    MetaBool,
    MetaInlines,
    MetaList,
    MetaValue,
    Pandoc,
    Para,
    RawBlock,
    RawInline,
    blocks_of,
    items,
    lookup,
    main,
    stringify,
)

CSS = """  <style>
    p {
      text-indent: SIZE;
      margin: 0;
    }
    header + p {
      text-indent: ;
    }
    :is(h1, h2, h3, h4, h5, h6) + p {
      text-indent: 0;
    }
    :is(div.no-first-line-indent-after) + p {
      text-indent: 0;
    }
    :is(div.first-line-indent-after) + p {
      text-indent: SIZE;
    }
  </style>
"""

DIVS = {
    "indent": '<div class="first-line-indent-after"></div>',
    "noindent": '<div class="no-first-line-indent-after"></div>',
}


@dataclass
class Options:
    set_metadata_variable: bool = True
    set_header_includes: bool = True
    auto_remove: bool = True
    remove_after: list[str] = field(
        default_factory=lambda: [
            "BlockQuote",
            "BulletList",
            "CodeBlock",
            "DefinitionList",
            "HorizontalRule",
            "OrderedList",
        ]
    )
    size: str = "1em"


options = Options()  # read from the metadata by the first filter, used by the second


def command(kind: str) -> RawInline:
    """`\\indent ` or `\\noindent `, as the markdown reader makes them."""
    return RawInline("tex", f"\\{kind} ")


def names(value: MetaValue | None) -> list[str] | None:
    """Block types: text is one, a list's items each one; else none."""
    match value:
        case MetaInlines():
            return [stringify(value)]
        case MetaList(values):
            return [stringify(x) for x in values]
    return None


def lua_replacement(text: str, match: str) -> str:
    """`text` as Lua's `gsub` reads a replacement, for a pattern without
    captures: `%0` and `%1` are the match, `%%` a percent sign, any other
    `%` an error. (So a size of `5%` fails, as in the original.)"""

    def escape(m: re.Match[str]) -> str:
        if m[1] in ("0", "1"):
            return match
        if m[1] == "%":
            return "%"
        raise ValueError("first-line-indent: invalid use of '%' in replacement string")

    return re.sub(r"%(.?)", escape, text, flags=re.DOTALL)


configure, body = Filter(), Filter()


@configure.on_meta
def read(meta: Meta, ctx: Context) -> None:
    user = meta.get("first-line-indent")
    if user == MetaBool(True):
        raise ValueError("first-line-indent: a configuration of true")  # (Lua indexes a boolean)
    if user:
        if lookup(user, "set-metadata-variable") == MetaBool(False):
            options.set_metadata_variable = False
        if lookup(user, "set-header-includes") == MetaBool(False):
            options.set_header_includes = False
        if lookup(user, "auto-remove") == MetaBool(False):
            options.auto_remove = False
        options.remove_after += names(lookup(user, "remove-after")) or []
        if (blacklist := names(lookup(user, "dont-remove-after"))) is not None:
            options.remove_after = [x for x in options.remove_after if x not in blacklist]
        if (size := lookup(user, "size")) is not None:
            options.size = stringify(size)

    # The original changes the metadata, then returns it only if it set
    # `indent`: otherwise (Lua's `return nil`) nothing it changed counts,
    # but what fails as it changes it (header-includes) fails all the same.
    changes = options.set_metadata_variable and meta.get("indent") != MetaBool(False)
    fmt = ctx.format or ""
    header = None
    if not options.set_header_includes:
        pass
    elif "htm" in fmt:  # (Lua's pattern `html*`: `htm`, then any `l`s)
        header = RawBlock("html", CSS.replace("SIZE", lua_replacement(options.size, "SIZE")))
    elif "latex" in fmt and options.size != "1em":
        header = RawBlock("tex", f"\\setlength{{\\parindent}}{{{options.size}}}")
    # before what is there: a list's items, or one value
    existing = meta.get("header-includes")
    includes = header and MetaBlocks(header, *blocks_of(MetaList(*(items(existing) if existing else []))))
    if changes:
        meta["indent"] = True
        if includes:
            meta["header-includes"] = includes


def markup(kind: str, para: Para, fmt: str) -> list[Block]:
    """The paragraph with its first line indented or not (`kind`)."""
    if "latex" in fmt:
        if para.content and para.content[0] in (command("indent"), command("noindent")):
            para.content[0] = command(kind)
        else:
            para.content.insert(0, command(kind))
        return [para]
    if "htm" in fmt:
        return [RawBlock("html", DIVS[kind]), para]
    return [para]


@body.on(Pandoc)
def indents(doc: Pandoc, ctx: Context) -> None:
    fmt = ctx.format or ""
    blocks: list[Block] = []
    after = False  # whether the block before removes an indent
    for block in doc.blocks:
        if not isinstance(block, Para):
            after = type(block).__name__ in options.remove_after
            blocks.append(block)
            continue
        first = block.content[0] if block.content else None
        if explicit := next((k for k in ("indent", "noindent") if first == command(k)), None):
            blocks += markup(explicit, block, fmt)
        elif options.auto_remove and after:
            blocks += markup("noindent", block, fmt)
        else:
            blocks.append(block)
        after = False
    doc.blocks = blocks


if __name__ == "__main__":
    main([configure, body])  # the options and the metadata, then the body
