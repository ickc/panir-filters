#!/usr/bin/env julia
# Upper-case all regular text: a pandoc filter. Code, link URLs and the
# like are not text, and stay as they are.
#
# A port of pandocfilters' examples/caps.py (Copyright (c) 2013, John
# MacFarlane; BSD-3-Clause, see ../LICENSE).

using Panir

caps(s::Str) = (s.text = uppercase(s.text); nothing)

run_filter(caps)
