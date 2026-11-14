#!/usr/bin/env julia
# Emphasis shown in capitals: a pandoc filter. An emphasis is replaced by
# its content, upper-cased (its text: code, link URLs and the like stay as
# they are).
#
# A port of pandocfilters' examples/deemph.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

caps(s::Str) = (s.text = uppercase(s.text); nothing)

deemph(emph::Emph) = walk!(caps, emph.content)

# pandocfilters' order: what replaces an emphasis is not filtered itself,
# only what is in it, so an emphasis directly in another stays one
run_filter(deemph; traverse = :topdown)
