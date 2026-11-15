#!/usr/bin/env julia
# Quotation marks by the document's language: a pandoc filter.
#
# Each quotation (`Quoted`) becomes its content between the marks of the
# language: `quot-marks` (four marks, as text of four characters or a list
# of four), else those of `quot-lang`, else of `lang` (an RFC 5646 code: the
# language and region, then the language alone, then one of its regions).
# `quot-marks-by-lang` adds languages or changes them. Without any, the
# quotations stay; a bad field or an unknown language is an error.
#
# A port of pandoc/lua-filters' pandoc-quotes.lua (Copyright (c) 2018 Odin
# Kroeger; MIT, see ../LICENSE).

using Panir

# primary left, primary right, secondary left, secondary right: from
# <https://en.wikipedia.org/w/index.php?title=Quotation_mark&oldid=836731669>
const QUOT_MARKS_BY_LANG = Dict{String, NTuple{4, String}}(
    "de-CH" => ("«", "»", "‹", "›"),
    "en-CA" => ("“", "”", "‘", "’"),
    "en-GB" => ("‘", "’", "“", "”"),
    "en-UK" => ("‘", "’", "“", "”"),
    "en-US" => ("“", "”", "‘", "’"),
    "it-CH" => ("«", "»", "‹", "›"),
    "ko-KR" => ("“", "”", "‘", "’"),
    "pt-BR" => ("“", "”", "‘", "’"),
    "ar" => ("”", "“", "’", "‘"),
    "bo" => ("「", "」", "『", "』"),
    "bs" => ("”", "”", "’", "’"),
    "cn" => ("「", "」", "『", "』"),
    "cs" => ("„", "“", "‚", "‘"),
    "cy" => ("‘", "’", "“", "”"),
    "da" => ("»", "«", "›", "‹"),
    "de" => ("„", "“", "‚", "‘"),
    "el" => ("«", "»", "“", "”"),
    "en" => ("“", "”", "‘", "’"),
    "eo" => ("“", "”", "‘", "’"),
    "es" => ("«", "»", "“", "”"),
    "et" => ("„", "“", "‚", "‘"),
    "fa" => ("«", "»", "‹", "›"),
    "fi" => ("”", "”", "’", "’"),
    "fil" => ("“", "”", "‘", "’"),
    "fr" => ("«", "»", "‹", "›"),
    "ga" => ("“", "”", "‘", "’"),
    "gd" => ("‘", "’", "“", "”"),
    "gl" => ("«", "»", "‹", "›"),
    "he" => ("“", "”", "‘", "’"),
    "hi" => ("“", "”", "‘", "’"),
    "hr" => ("„", "“", "‚", "‘"),
    "hu" => ("„", "”", "»", "«"),
    "ia" => ("“", "”", "‘", "’"),
    "id" => ("“", "”", "‘", "’"),
    "is" => ("„", "“", "‚", "‘"),
    "it" => ("«", "»", "“", "”"),
    "ja" => ("「", "」", "『", "』"),
    "jbo" => ("lu", "li'u", "lu", "li'u"),
    "ka" => ("„", "“", "‚", "‘"),
    "khb" => ("《", "》", "〈", "〉"),
    "kk" => ("«", "»", "‹", "›"),
    "km" => ("«", "»", "‹", "›"),
    "ko" => ("《", "》", "〈", "〉"),
    "lo" => ("«", "»", "‹", "›"),
    "lt" => ("„", "“", "‚", "‘"),
    "lv" => ("„", "“", "‚", "‘"),
    "mk" => ("„", "“", "’", "‘"),
    "mn" => ("«", "»", "‹", "›"),
    "mt" => ("“", "”", "‘", "’"),
    "nl" => ("„", "”", "‚", "’"),
    "no" => ("«", "»", "«", "»"),
    "pl" => ("„", "”", "»", "«"),
    "ps" => ("«", "»", "‹", "›"),
    "pt" => ("«", "»", "“", "”"),
    "rm" => ("«", "»", "‹", "›"),
    "ro" => ("„", "”", "«", "»"),
    "ru" => ("«", "»", "„", "“"),
    "sk" => ("„", "“", "‚", "‘"),
    "sl" => ("„", "“", "‚", "‘"),
    "sq" => ("„", "“", "‚", "‘"),
    "sr" => ("„", "“", "’", "’"),
    "sv" => ("”", "”", "’", "’"),
    "tdd" => ("「", "」", "『", "』"),
    "th" => ("“", "”", "‘", "’"),
    "thi" => ("「", "」", "『", "』"),
    "ti" => ("«", "»", "‹", "›"),
    "tr" => ("«", "»", "‹", "›"),
    "ug" => ("«", "»", "‹", "›"),
    "uk" => ("«", "»", "„", "“"),
    "uz" => ("«", "»", "„", "“"),
    "vi" => ("“", "”", "‘", "’"),
    "wen" => ("„", "“", "‚", "‘"),
)

"The original warns, then fails as its warning does (NOTES.md)."
struct QuotesError <: Exception
    msg::String
end
Base.showerror(io::IO, e::QuotesError) = print(io, "pandoc-quotes: ", e.msg)

"Four marks: text of four characters, or a list of four."
quotation_marks(v::MetaInlines, field) = four(string.(collect(stringify(v))), field)
quotation_marks(v::MetaList, field) = four(map(stringify, v.content), field)
quotation_marks(::MetaValue, field) = throw(QuotesError("$field: neither a string nor a list"))
four(marks, field) = length(marks) == 4 ? Tuple(marks) : throw(QuotesError("$field: not four quotation marks"))

function quot_marks(meta)
    by_lang = copy(QUOT_MARKS_BY_LANG)
    custom = get(meta, "quot-marks-by-lang", nothing)
    if custom isa MetaMap
        for (lang, marks) in custom.content
            by_lang[lang] = quotation_marks(marks, "metadata field \"quot-marks-by-lang\": lang \"$lang\"")
        end
    elseif truthy(custom)
        throw(QuotesError("metadata field \"quot-marks-by-lang\": not a map"))
    end
    marks = get(meta, "quot-marks", nothing)
    truthy(marks) && return quotation_marks(marks, "metadata field \"quot-marks\"")
    field = truthy(get(meta, "quot-lang", nothing)) ? meta["quot-lang"] : get(meta, "lang", nothing)
    truthy(field) || return nothing
    # the language and region, the language, a region of the language
    lang = stringify(field)
    haskey(by_lang, lang) && return by_lang[lang]
    m = match(r"^[A-Za-z]+", lang)  # Lua's %a
    m === nothing && throw(QuotesError("$lang: unknown language."))
    language = m.match
    haskey(by_lang, language) && return by_lang[language]
    # a region: the original's Lua pattern `^lang-`, where `-` repeats the last
    # letter, takes any key starting with the language but its last letter (the
    # first in this table, where Lua's order is random: NOTES.md)
    regions = [marks for (k, marks) in by_lang if startswith(k, chop(language))]
    isempty(regions) ? throw(QuotesError("$language: unknown language.")) : first(regions)
end

const marks = Ref{Any}(nothing)
configure(meta::Panir.Meta) = (marks[] = quot_marks(meta); nothing)
function insert_marks(quoted::Quoted)
    marks[] === nothing && return nothing
    left, right = quoted.quote_type == DoubleQuote ? marks[][1:2] : marks[][3:4]
    [Str(left); quoted.content; Str(right)]
end

run_filter([configure, insert_marks])  # the metadata, then every quotation, inner ones first
