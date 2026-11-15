#!/usr/bin/env python3
"""Quotation marks by the document's language: a pandoc filter.

Each quotation (`Quoted`) becomes its content between the marks of the
language: `quot-marks` (four marks, as text of four characters or a list
of four), else those of `quot-lang`, else of `lang` (an RFC 5646 code: the
language and region, then the language alone, then one of its regions).
`quot-marks-by-lang` adds languages or changes them. Without any, the
quotations stay; a bad field or an unknown language is an error.

A port of pandoc/lua-filters' pandoc-quotes.lua (Copyright (c) 2018 Odin
Kroeger; MIT, see ../LICENSE).
"""

import re

from panir import (
    Filter,
    Inline,
    Meta,
    MetaInlines,
    MetaList,
    MetaMap,
    MetaValue,
    Quoted,
    QuoteType,
    Str,
    main,
    stringify,
)

# primary left, primary right, secondary left, secondary right: from
# <https://en.wikipedia.org/w/index.php?title=Quotation_mark&oldid=836731669>
QUOT_MARKS_BY_LANG: dict[str, tuple[str, ...]] = {
    "de-CH": ("«", "»", "‹", "›"),
    "en-CA": ("“", "”", "‘", "’"),
    "en-GB": ("‘", "’", "“", "”"),
    "en-UK": ("‘", "’", "“", "”"),
    "en-US": ("“", "”", "‘", "’"),
    "it-CH": ("«", "»", "‹", "›"),
    "ko-KR": ("“", "”", "‘", "’"),
    "pt-BR": ("“", "”", "‘", "’"),
    "ar": ("”", "“", "’", "‘"),
    "bo": ("「", "」", "『", "』"),
    "bs": ("”", "”", "’", "’"),
    "cn": ("「", "」", "『", "』"),
    "cs": ("„", "“", "‚", "‘"),
    "cy": ("‘", "’", "“", "”"),
    "da": ("»", "«", "›", "‹"),
    "de": ("„", "“", "‚", "‘"),
    "el": ("«", "»", "“", "”"),
    "en": ("“", "”", "‘", "’"),
    "eo": ("“", "”", "‘", "’"),
    "es": ("«", "»", "“", "”"),
    "et": ("„", "“", "‚", "‘"),
    "fa": ("«", "»", "‹", "›"),
    "fi": ("”", "”", "’", "’"),
    "fil": ("“", "”", "‘", "’"),
    "fr": ("«", "»", "‹", "›"),
    "ga": ("“", "”", "‘", "’"),
    "gd": ("‘", "’", "“", "”"),
    "gl": ("«", "»", "‹", "›"),
    "he": ("“", "”", "‘", "’"),
    "hi": ("“", "”", "‘", "’"),
    "hr": ("„", "“", "‚", "‘"),
    "hu": ("„", "”", "»", "«"),
    "ia": ("“", "”", "‘", "’"),
    "id": ("“", "”", "‘", "’"),
    "is": ("„", "“", "‚", "‘"),
    "it": ("«", "»", "“", "”"),
    "ja": ("「", "」", "『", "』"),
    "jbo": ("lu", "li'u", "lu", "li'u"),
    "ka": ("„", "“", "‚", "‘"),
    "khb": ("《", "》", "〈", "〉"),
    "kk": ("«", "»", "‹", "›"),
    "km": ("«", "»", "‹", "›"),
    "ko": ("《", "》", "〈", "〉"),
    "lo": ("«", "»", "‹", "›"),
    "lt": ("„", "“", "‚", "‘"),
    "lv": ("„", "“", "‚", "‘"),
    "mk": ("„", "“", "’", "‘"),
    "mn": ("«", "»", "‹", "›"),
    "mt": ("“", "”", "‘", "’"),
    "nl": ("„", "”", "‚", "’"),
    "no": ("«", "»", "«", "»"),
    "pl": ("„", "”", "»", "«"),
    "ps": ("«", "»", "‹", "›"),
    "pt": ("«", "»", "“", "”"),
    "rm": ("«", "»", "‹", "›"),
    "ro": ("„", "”", "«", "»"),
    "ru": ("«", "»", "„", "“"),
    "sk": ("„", "“", "‚", "‘"),
    "sl": ("„", "“", "‚", "‘"),
    "sq": ("„", "“", "‚", "‘"),
    "sr": ("„", "“", "’", "’"),
    "sv": ("”", "”", "’", "’"),
    "tdd": ("「", "」", "『", "』"),
    "th": ("“", "”", "‘", "’"),
    "thi": ("「", "」", "『", "』"),
    "ti": ("«", "»", "‹", "›"),
    "tr": ("«", "»", "‹", "›"),
    "ug": ("«", "»", "‹", "›"),
    "uk": ("«", "»", "„", "“"),
    "uz": ("«", "»", "„", "“"),
    "vi": ("“", "”", "‘", "’"),
    "wen": ("„", "“", "‚", "‘"),
}


class QuotesError(ValueError):
    """The original warns, then fails as its warning does (NOTES.md)."""


def quotation_marks(value: MetaValue, field: str) -> tuple[str, ...]:
    """Four marks: text of four characters, or a list of four."""
    match value:
        case MetaInlines():
            marks = tuple(stringify(value))
        case MetaList(items):
            marks = tuple(map(stringify, items))
        case _:
            raise QuotesError(f"{field}: neither a string nor a list")
    if len(marks) != 4:
        raise QuotesError(f"{field}: not four quotation marks")
    return marks


def quot_marks(meta: Meta) -> tuple[str, ...] | None:
    by_lang = dict(QUOT_MARKS_BY_LANG)
    match meta.get("quot-marks-by-lang"):
        case MetaMap(entries):
            for lang, marks in entries.items():
                by_lang[lang] = quotation_marks(
                    marks, f'metadata field "quot-marks-by-lang": lang "{lang}"'
                )
        case other if other:
            raise QuotesError('metadata field "quot-marks-by-lang": not a map')
    if marks := meta.get("quot-marks"):
        return quotation_marks(marks, 'metadata field "quot-marks"')
    field = meta.get("quot-lang") or meta.get("lang")
    if not field:
        return None
    lang = stringify(field)
    # the language and region, the language, a region of the language
    if found := by_lang.get(lang):
        return found
    language = re.match("[A-Za-z]+", lang)  # Lua's %a
    if language is None:
        raise QuotesError(f"{lang}: unknown language.")
    # a region: the original's Lua pattern `^lang-`, where `-` repeats the last
    # letter, takes any key starting with the language but its last letter
    # (the first in this table, where Lua's order is random: NOTES.md)
    region = next((v for k, v in by_lang.items() if k.startswith(language[0][:-1])), None)
    if found := by_lang.get(language[0]) or region:
        return found
    raise QuotesError(f"{language[0]}: unknown language.")


configure, quotes = Filter(), Filter()
marks: tuple[str, ...] | None = None


@configure.on_meta
def read(meta: Meta) -> None:
    global marks
    marks = quot_marks(meta)


@quotes.on(Quoted)
def insert_marks(quoted: Quoted) -> list[Inline] | None:
    if marks is None:
        return None
    left, right = marks[:2] if quoted.quote_type == QuoteType.DoubleQuote else marks[2:]
    return [Str(left), *quoted.content, Str(right)]


if __name__ == "__main__":
    main([configure, quotes])  # the metadata, then every quotation, inner ones first
