#!/usr/bin/env node
// Quotation marks by the document's language: a pandoc filter.
//
// Each quotation (`Quoted`) becomes its content between the marks of the
// language: `quot-marks` (four marks, as text of four characters or a list
// of four), else those of `quot-lang`, else of `lang` (an RFC 5646 code: the
// language and region, then the language alone, then one of its regions).
// `quot-marks-by-lang` adds languages or changes them. Without any, the
// quotations stay; a bad field or an unknown language is an error.
//
// A port of pandoc/lua-filters' pandoc-quotes.lua (Copyright (c) 2018 Odin
// Kroeger; MIT, see ../LICENSE).

import { given, Str, stringify, truthy, type Meta, type MetaValue } from "panir";
import { runFilter } from "panir/node";

type Marks = readonly string[]; // primary left, right, secondary left, right

// from <https://en.wikipedia.org/w/index.php?title=Quotation_mark&oldid=836731669>
const QUOT_MARKS_BY_LANG: { readonly [lang: string]: Marks } = {
  "de-CH": ["«", "»", "‹", "›"],
  "en-CA": ["“", "”", "‘", "’"],
  "en-GB": ["‘", "’", "“", "”"],
  "en-UK": ["‘", "’", "“", "”"],
  "en-US": ["“", "”", "‘", "’"],
  "it-CH": ["«", "»", "‹", "›"],
  "ko-KR": ["“", "”", "‘", "’"],
  "pt-BR": ["“", "”", "‘", "’"],
  ar: ["”", "“", "’", "‘"],
  bo: ["「", "」", "『", "』"],
  bs: ["”", "”", "’", "’"],
  cn: ["「", "」", "『", "』"],
  cs: ["„", "“", "‚", "‘"],
  cy: ["‘", "’", "“", "”"],
  da: ["»", "«", "›", "‹"],
  de: ["„", "“", "‚", "‘"],
  el: ["«", "»", "“", "”"],
  en: ["“", "”", "‘", "’"],
  eo: ["“", "”", "‘", "’"],
  es: ["«", "»", "“", "”"],
  et: ["„", "“", "‚", "‘"],
  fa: ["«", "»", "‹", "›"],
  fi: ["”", "”", "’", "’"],
  fil: ["“", "”", "‘", "’"],
  fr: ["«", "»", "‹", "›"],
  ga: ["“", "”", "‘", "’"],
  gd: ["‘", "’", "“", "”"],
  gl: ["«", "»", "‹", "›"],
  he: ["“", "”", "‘", "’"],
  hi: ["“", "”", "‘", "’"],
  hr: ["„", "“", "‚", "‘"],
  hu: ["„", "”", "»", "«"],
  ia: ["“", "”", "‘", "’"],
  id: ["“", "”", "‘", "’"],
  is: ["„", "“", "‚", "‘"],
  it: ["«", "»", "“", "”"],
  ja: ["「", "」", "『", "』"],
  jbo: ["lu", "li'u", "lu", "li'u"],
  ka: ["„", "“", "‚", "‘"],
  khb: ["《", "》", "〈", "〉"],
  kk: ["«", "»", "‹", "›"],
  km: ["«", "»", "‹", "›"],
  ko: ["《", "》", "〈", "〉"],
  lo: ["«", "»", "‹", "›"],
  lt: ["„", "“", "‚", "‘"],
  lv: ["„", "“", "‚", "‘"],
  mk: ["„", "“", "’", "‘"],
  mn: ["«", "»", "‹", "›"],
  mt: ["“", "”", "‘", "’"],
  nl: ["„", "”", "‚", "’"],
  no: ["«", "»", "«", "»"],
  pl: ["„", "”", "»", "«"],
  ps: ["«", "»", "‹", "›"],
  pt: ["«", "»", "“", "”"],
  rm: ["«", "»", "‹", "›"],
  ro: ["„", "”", "«", "»"],
  ru: ["«", "»", "„", "“"],
  sk: ["„", "“", "‚", "‘"],
  sl: ["„", "“", "‚", "‘"],
  sq: ["„", "“", "‚", "‘"],
  sr: ["„", "“", "’", "’"],
  sv: ["”", "”", "’", "’"],
  tdd: ["「", "」", "『", "』"],
  th: ["“", "”", "‘", "’"],
  thi: ["「", "」", "『", "』"],
  ti: ["«", "»", "‹", "›"],
  tr: ["«", "»", "‹", "›"],
  ug: ["«", "»", "‹", "›"],
  uk: ["«", "»", "„", "“"],
  uz: ["«", "»", "„", "“"],
  vi: ["“", "”", "‘", "’"],
  wen: ["„", "“", "‚", "‘"],
};

/** The original warns, then fails as its warning does (NOTES.md). */
class QuotesError extends Error {}

/** Four marks: text of four characters, or a list of four. */
function quotationMarks(value: MetaValue, field: string): Marks {
  let marks: Marks;
  if (value.t === "MetaInlines") marks = [...stringify(value)];
  else if (value.t === "MetaList") marks = value.content.map((x) => stringify(x));
  else throw new QuotesError(`${field}: neither a string nor a list`);
  if (marks.length !== 4) throw new QuotesError(`${field}: not four quotation marks`);
  return marks;
}

function configure(meta: Meta): Marks | undefined {
  const byLang = new Map(Object.entries(QUOT_MARKS_BY_LANG));
  const custom = meta["quot-marks-by-lang"];
  if (custom?.t === "MetaMap") {
    for (const [lang, marks] of Object.entries(custom.content)) {
      byLang.set(lang, quotationMarks(marks, `metadata field "quot-marks-by-lang": lang "${lang}"`));
    }
  } else if (truthy(custom)) {
    throw new QuotesError('metadata field "quot-marks-by-lang": not a map');
  }
  const marks = given(meta["quot-marks"]);
  if (marks) return quotationMarks(marks, 'metadata field "quot-marks"');
  const field = given(meta["quot-lang"]) ?? given(meta.lang);
  if (!field) return undefined;
  // the language and region, the language, a region of the language: the
  // original's Lua pattern `^lang-`, where `-` repeats the last letter, takes
  // any key starting with the language but its last letter (the first in this
  // table, where Lua's order is random: NOTES.md)
  const lang = stringify(field);
  const language = /^[A-Za-z]+/.exec(lang)?.[0]; // Lua's %a
  const found = byLang.get(lang)
    ?? (language === undefined ? undefined
      : byLang.get(language) ?? [...byLang].find(([k]) => k.startsWith(language.slice(0, -1)))?.[1]);
  if (found === undefined) throw new QuotesError(`${language ?? lang}: unknown language.`);
  return found;
}

let marks: Marks | undefined;

await runFilter([
  { Meta: (meta) => { marks = configure(meta); } },
  {
    Quoted: (quoted) => {
      if (marks === undefined) return;
      const [left, right] = quoted.quoteType === "DoubleQuote" ? marks.slice(0, 2) : marks.slice(2);
      return [Str(left), ...quoted.content, Str(right)];
    },
  },
]);
