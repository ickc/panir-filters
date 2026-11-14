#!/usr/bin/env julia
# Definition lists to bullet lists, each term in bold before its
# definitions: a pandoc filter, for Markdown without definition lists.
#
# A port of pandocfilters' examples/deflists.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

deflists(dl::DefinitionList) =
    BulletList([[Para(Strong(term)); reduce(vcat, defs; init = Block[])] for (term, defs) in dl.content])

run_filter(deflists)
