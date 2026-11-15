#!/usr/bin/env julia
# LaTeX page breaks to the output format's: a pandoc filter.
#
# A raw TeX block holding only `\newpage` or `\pagebreak` becomes the
# format's page break (a raw block, or a paragraph of a form feed in
# formats without one); with `pagebreak.break-on.form-feed`, so does a
# paragraph of a form feed alone, and with `break-on.plaintext-command`, a
# paragraph of the command as text. `html-class` and `odt-style` (or
# `$PANDOC_PAGEBREAK_HTML_CLASS`, `$PANDOC_PAGEBREAK_ODT_STYLE`) style the
# HTML and ODT ones; the configuration is removed.
#
# A port of pandoc-ext/pagebreak (Copyright © 2017-2025 Benct Philip
# Jonsson, Albert Krewinkel; ISC; the repository's MIT licence, see
# ../LICENSE).

using Panir

const NEWPAGE = r"^\\(?:newpage|pagebreak)\{?\}?$"

# A text option: the metadata's, else the environment's. (The top-level
# field: the original reads `html-class`, not `pagebreak.html-class` as its
# README says.)
function option(meta, key, variable)
    value = get(meta, key, nothing)
    truthy(value) ? stringify(value) : get(ENV, variable, "")
end

"A page break in the format."
function pagebreak(format, meta)
    occursin("asciidoc", format) && return RawBlock("asciidoc", "<<<\n\n")
    format == "context" && return RawBlock("context", "\\page")
    format == "docx" && return RawBlock("openxml", "<w:p><w:r><w:br w:type=\"page\"/></w:r></w:p>")
    occursin("epub", format) && return RawBlock("html", "<p style=\"page-break-after: always;\"> </p>")
    if occursin("html", format)
        class = option(meta, "html-class", "PANDOC_PAGEBREAK_HTML_CLASS")
        return RawBlock("html", isempty(class) ? "<div style=\"page-break-after: always;\"></div>" : "<div class=\"$class\"></div>")
    end
    occursin("latex", format) && return RawBlock("tex", "\\newpage{}")
    occursin("ms", format) && return RawBlock("ms", ".bp")
    if occursin("odt", format)
        style = option(meta, "odt-style", "PANDOC_PAGEBREAK_ODT_STYLE")
        return RawBlock("opendocument", "<text:p text:style-name=\"$(isempty(style) ? "Pagebreak" : style)\"/>")
    end
    occursin("typst", format) && return RawBlock("typst", "#pagebreak()\n\n")
    Para(Str("\f"))  # a form feed, in any other format
end

function page_breaks(doc::Pandoc, ctx)
    format = something(ctx.format, "")
    config = pop!(doc.meta, "pagebreak", nothing)
    if config == MetaBool(true) || lookup(config, "break-on") == MetaBool(true)
        error("pagebreak: a configuration of true")  # (Lua indexes a boolean)
    end
    raw = pagebreak(format, doc.meta)
    raw_tex = !endswith(format, "tex")  # (in LaTeX, raw TeX stays as it is)
    form_feed = truthy(lookup(config, "break-on", "form-feed"))
    plaintext = truthy(lookup(config, "break-on", "plaintext-command"))
    marker(b::RawBlock) = raw_tex && occursin("tex", b.format) && occursin(NEWPAGE, b.text)
    marker(p::Para) = length(p.content) == 1 && p.content[1] isa Str &&
        (form_feed && p.content[1].text == "\f" || plaintext && occursin(NEWPAGE, p.content[1].text))
    breaks(b::Union{RawBlock, Para}) = marker(b) ? deepcopy(raw) : nothing  # (one page break, in many places)
    walk!(breaks, doc)  # the metadata's blocks too
    nothing
end

run_filter(page_breaks)
