#!/usr/bin/env python3
"""Several bibliographies, one a topic: a pandoc filter.

The metadata's `bibliography` maps topics to files; a div `#refs-TOPIC`
(or `refs_TOPIC`, `refsTOPIC`) gets the bibliography of that topic's file,
of every work the document cites. The document's citations are resolved
with all the files together.

citeproc is pandoc's (`pandoc.utils.citeproc` in the original): run here
with `libpandoc.convert` of the document's JSON with `citeproc`, in this
process: the pandoc the filter runs in under pandocpy or
`libpandoc.convert`, or libpandoc as a library under plain pandoc. Without
libpandoc it fails, saying so.

A port of pandoc-ext/multibib (Copyright © 2018-2024 Albert Krewinkel;
MIT, see ../LICENSE).
"""

import copy
import re
import sys
import warnings

from panir import Cite, Context, Div, Filter, Meta, MetaList, MetaMap, MetaString, Pandoc, Para, dumps, loads, main, stringify

try:
    import libpandoc
except ImportError:
    sys.exit("multibib: needs pandoc's citeproc, libpandoc (pip install libpandoc)")

# What citeproc reads of the metadata, copied for a topic's bibliography.
FIELDS = ["bibliography", "references", "csl", "citation-style", "link-citations", "citation-abbreviations",
          "lang", "suppress-bibliography", "reference-section-title", "notes-after-punctuation", "nocite"]


def citeproc(doc: Pandoc) -> Pandoc:
    """pandoc's citeproc on a document, its warnings silenced."""
    with warnings.catch_warnings():
        warnings.simplefilter("ignore", libpandoc.PandocWarning)
        return loads(libpandoc.convert(dumps(doc), from_="json", to="json", citeproc=True))


class State:
    cites: list[Cite]  # every citation of the document
    meta = Meta()  # the document's metadata, as the first pass saw it
    refs: Div | None = None  # the bibliography's div, as citeproc made it


state = State()
start, collect, resolve, topics = Filter(), Filter(), Filter(), Filter()


@start.on(Pandoc)
def reset(doc: Pandoc) -> None:
    """A new document (a host may run these filters on many: Lua loads the
    original anew for each)."""
    state.cites = []


@collect.on(Cite)
def collect_cite(cite: Cite) -> None:
    state.cites.append(copy.deepcopy(cite))


@collect.on_meta
def collect_meta(meta: Meta) -> None:
    state.meta = copy.deepcopy(meta)


@resolve.on(Pandoc)
def resolve_doc_citations(doc: Pandoc) -> Pandoc:
    """The citations resolved with every bibliography at once."""
    given = doc.meta.get("bibliography")
    # (Lua's pandoc.utils.type: a map is a "table"; its values in any order)
    files = [MetaString(stringify(v)) for v in given.content.values()] if isinstance(given, MetaMap) else []
    doc.meta["bibliography"] = MetaList(*files)  # (MetaList([...]) is a list in a list)
    doc.blocks.append(Div(identifier="refs"))
    doc = citeproc(doc)
    state.refs = doc.blocks.pop()
    # (the original restores `orig_bib`, which it never set: the field goes)
    doc.meta.pop("bibliography", None)
    return doc


@topics.on(Div)
def create_topic_bibliography(div: Div, ctx: Context) -> Div | None:
    m = re.fullmatch(r"refs[-_]?([-_A-Za-z0-9]*)", div.identifier)
    bibliography = state.meta.get("bibliography")
    # (a map's entry; Lua indexes a list or text by a name to nothing)
    bibfile = bibliography.content.get(m[1]) if m and isinstance(bibliography, MetaMap) else None
    if bibfile is None:
        return None
    meta = Meta({k: copy.deepcopy(state.meta[k]) for k in FIELDS if k in state.meta})
    meta["bibliography"] = copy.deepcopy(bibfile)
    res = citeproc(Pandoc(Para(*copy.deepcopy(state.cites)), Div(identifier="refs"), meta=meta))
    div.content = res.blocks[1].content
    assert state.refs is not None
    div.classes = list(dict.fromkeys(state.refs.classes))
    div.attributes = copy.deepcopy(state.refs.attributes)
    if ctx.format in ("odt", "docx"):
        # pandoc gives Word's bibliography this style
        div.attributes["custom-style"] = div.attributes.get("custom-style") or "Bibliography"
    return div


if __name__ == "__main__":
    main([start, collect, resolve, topics])
