#!/usr/bin/env node
// Tracked changes (Word's, as pandoc reads them with `--track-changes=all`):
// accepted, rejected, or shown: a pandoc filter.
//
// The metadata's `trackChanges` (`accept`, the default, `reject` or `all`)
// decides. Accepting keeps insertions and drops deletions and comments;
// rejecting the reverse; `all` shows them in HTML (`ins`, `del`, `mark`) and
// LaTeX (the `changes` package, in `header-includes`, with a colour per
// author), and in Word and ODT keeps them, a comment's `¶` a line break.
//
// A port of pandoc/lua-filters' track-changes (Copyright (c) 2017-2021
// pandoc Lua filters contributors; MIT, see ../LICENSE).

import {
  applyFilter, getAttribute, LineBreak, MetaBlocks, MetaList, MetaString, RawBlock, RawInline, Str, stringify,
  type Filter, type Inline, type MetaValue, type Span,
} from "panir";
import { runFilter } from "panir/node";

const HEADER = String.raw`
\makeatletter
\PassOptionsToPackage{textsize=scriptsize}{todonotes}
\PassOptionsToPackage{markup=underlined,authormarkup=none,commentmarkup=todo}{changes}
\usepackage{changes}
\@ifpackagelater{changes}{2018/11/03}{%
}{%
  \usepackage{todonotes}
  \setremarkmarkup{\todo[color=Changes@Color#1!20]{\sffamily\textbf{#1:}~#2}}
}%
\makeatother
\definecolor{auth1}{HTML}{4477AA}
\definecolor{auth2}{HTML}{117733}
\definecolor{auth3}{HTML}{999933}
\definecolor{auth4}{HTML}{CC6677}
\definecolor{auth5}{HTML}{AA4499}
\definecolor{auth6}{HTML}{332288}
\setlength{\marginparwidth}{3cm}
\newcommand{\note}[2][]{\added[#1,remark={#2}]{}}
\newcommand\hlnotesingle{%
  \bgroup
  \expandafter\def\csname sout\space\endcsname{\bgroup \ULdepth =-.8ex \ULset}%
  \markoverwith{\textcolor{yellow}{\rule[-.5ex]{.1pt}{2.5ex}}}%
  \ULon}
\newcommand\hlnote[1]{\let\helpcmd\hlnotesingle\parhelp#1\par\relax\relax}
\long\def\parhelp#1\par#2\relax{%
  \helpcmd{#1}\ifx\relax#2\else\par\parhelp#2\relax\fi%
}

\makeatletter
\newcommand\ifmoving{%
  \ifx\protect\@unexpandable@protect
      \expandafter\@firstoftwo
  \else
      \expandafter\@secondoftwo
  \fi
}

\newcommand{\gobbletwo}[2][]{\@bsphack\@esphack}
\newcommand{\gobbleone}[1][]{\@bsphack\@esphack}

\let\oldadded\added
\let\olddeleted\deleted
\let\oldhlnote\hlnote
\let\oldnote\note
\renewcommand{\added}{\ifmoving{\gobbleone}{\oldadded}}
\renewcommand{\deleted}{\ifmoving{\gobbletwo}{\olddeleted}}
\renewcommand{\hlnote}{\ifmoving{}{\oldhlnote}}
\renewcommand{\note}{\ifmoving{\gobbletwo}{\oldnote}}
\makeatother
`;

const TEX: Record<string, string> = { "comment-start": "\\note", insertion: "\\added", deletion: "\\deleted" };
const HTML: Record<string, string> = { "comment-start": "mark", insertion: "ins", deletion: "del" };
const TITLES = new Set(["dr", "mr", "ms", "mrs", "prof", "mx", "sir"]);

const authors = new Map<string, string>(); // by initials, those of the changes shown in LaTeX

/** A name's initials (ASCII letters and digits: Lua's %w), titles left out;
 * a name of one word as it is. */
function initials(name: string): string {
  if (!name.includes(" ")) return name;
  const words = name.match(/[A-Za-z0-9']+/g) ?? [];
  return words.filter((w) => !TITLES.has(w.toLowerCase())).map((w) => w[0]!.toUpperCase()).join("");
}

/** A comment's `¶` (a paragraph break in Word) as `line()`. */
const pilcrows = (line: () => Inline): Filter => ({ Str: (s) => (s.text === "¶" ? line() : undefined) });

/** The author's initials, the author kept by them. */
function author(span: Span): string {
  const name = getAttribute(span.attr, "author");
  if (name === undefined) throw new Error("track-changes: a change without an author"); // (the original fails too)
  authors.set(initials(name), name);
  return initials(name);
}

function toTex(span: Span): Inline | undefined {
  const kind = span.attr.classes[0] ?? "";
  const classes = span.attr.classes;
  if (kind in TEX) {
    const text = classes.includes("comment-start")
      ? `${stringify(applyFilter(span, pilcrows(() => Str("\\newline"))))}}\\hlnote{`
      : `${stringify(span.content)}}`;
    return RawInline("latex", `${TEX[kind]}[id=${author(span)}]{${text}`);
  }
  if (classes.includes("comment-end")) return RawInline("latex", "}");
}

function toHtml(span: Span): Inline | undefined {
  const kind = span.attr.classes[0] ?? "";
  if (kind in HTML) {
    author(span);
    let s = `<${HTML[kind]}`;
    for (const [key, value] of span.attr.attributes) s += ` ${key === "date" ? key : `data-${key}`}="${value}"`;
    if (span.attr.classes.includes("comment-start")) {
      s += ` data-id="${span.attr.identifier}"`; // (Lua's "" is true: always)
      s += ` title="${stringify(applyFilter(span, pilcrows(() => Str("&#10;"))))}">`;
    } else {
      s += `>${stringify(span.content)}</${HTML[kind]}>`;
    }
    return RawInline("html", s);
  }
  if (span.attr.classes.includes("comment-end")) return RawInline("html", "</mark>");
}

const toWord = (span: Span) => (span.attr.classes.includes("comment-start") ? applyFilter(span, pilcrows(LineBreak)) : undefined);

/** Accepting or rejecting: comments go, an insertion becomes its content
 * (accepted) or goes, a deletion the reverse; a span of both is an insertion. */
function resolve(span: Span, accept: boolean): Inline[] | undefined {
  const classes = span.attr.classes;
  if (classes.includes("comment-start") || classes.includes("comment-end")) return [];
  if (classes.includes("insertion")) return accept ? span.content : [];
  if (classes.includes("deletion")) return accept ? [] : span.content;
}

/** `header-includes` with the `changes` package and a colour for each author:
 * the original wraps what is there in a list, a list too (its test for a
 * list, of `.t`, fails since pandoc 2.17). */
function headerIncludes(existing: MetaValue | undefined): MetaValue {
  const out: MetaValue[] = existing === undefined ? [] : [existing];
  out.push(MetaBlocks([RawBlock("latex", HEADER)]));
  [...authors].sort(([a], [b]) => (a < b ? -1 : a > b ? 1 : 0)).forEach(([inits, name], i) => {
    out.push(MetaBlocks([RawBlock("latex", `\\definechangesauthor[name={${name}}, color=auth${i + 1}]{${inits}}`)]));
  });
  return MetaList(out);
}

await runFilter({
  Pandoc: (doc, ctx) => {
    // (the original reads the reader's option first, as pandoc 2 named it:
    // since pandoc 3 never there, so the metadata decides)
    const mode = stringify(doc.meta.trackChanges ?? MetaString("accept"));
    delete doc.meta.trackChanges;
    const format = ctx.format ?? "";
    const tex = ["latex", "tex", "context"].includes(format);
    let Span: ((span: Span) => Inline | Inline[] | undefined) | undefined;
    if (mode !== "all") {
      Span = (span) => resolve(span, mode !== "reject");
    } else if (["html", "html4", "html5"].includes(format)) Span = toHtml;
    else if (tex) Span = toTex;
    else if (["docx", "odt"].includes(format)) Span = toWord;
    if (Span) applyFilter(doc.blocks, { Span }); // (not the metadata)
    if (mode === "all" && tex) doc.meta["header-includes"] = headerIncludes(doc.meta["header-includes"]);
  },
});
