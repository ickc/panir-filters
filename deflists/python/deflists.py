#!/usr/bin/env python3
"""Definition lists to bullet lists, each term in bold before its
definitions: a pandoc filter, for Markdown without definition lists.

A port of pandocfilters' examples/deflists.py (Copyright (c) 2013, John
MacFarlane; BSD-3-Clause, see ../LICENSE).
"""

from panir import BulletList, DefinitionList, Filter, Para, Strong

f = Filter()


@f.on(DefinitionList)
def deflists(dl: DefinitionList) -> BulletList:
    return BulletList(*([Para(Strong(*term)), *(b for d in defs for b in d)] for term, defs in dl.content))


if __name__ == "__main__":
    f.main()
