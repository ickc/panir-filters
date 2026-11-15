#!/usr/bin/env julia
# Tracked changes (Word's, as pandoc reads them with `--track-changes=all`):
# accepted, rejected, or shown: a pandoc filter.
#
# The metadata's `trackChanges` (`accept`, the default, `reject` or `all`)
# decides. Accepting keeps insertions and drops deletions and comments;
# rejecting the reverse; `all` shows them in HTML (`ins`, `del`, `mark`) and
# LaTeX (the `changes` package, in `header-includes`, with a colour per
# author), and in Word and ODT keeps them, a comment's `¶` a line break.
#
# A port of pandoc/lua-filters' track-changes (Copyright (c) 2017-2021
# pandoc Lua filters contributors; MIT, see ../LICENSE).

using Panir

# (the first newline after """ is not in the string, the second is)
const HEADER = raw"""

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
"""

const TEX = Dict("comment-start" => "\\note", "insertion" => "\\added", "deletion" => "\\deleted")
const HTML = Dict("comment-start" => "mark", "insertion" => "ins", "deletion" => "del")
const TITLES = Set(["dr", "mr", "ms", "mrs", "prof", "mx", "sir"])

const authors = Dict{String, String}()  # by initials, those of the changes shown in LaTeX

"A name's initials (ASCII letters and digits: Lua's %w), titles left out; a name of one word as it is."
function initials(name)
    occursin(' ', name) || return name
    words = [m.match for m in eachmatch(r"[A-Za-z0-9']+", name)]
    join(uppercase(first(w)) for w in words if !(lowercase(w) in TITLES))
end

"The author's initials, the author kept by them (the original fails without one too)."
function author(span)
    name = span.attr["author"]
    authors[initials(name)] = name
    initials(name)
end

"A comment's `¶` (a paragraph break in Word) as `line()`."
pilcrows(line) = s::Str -> s.text == "¶" ? line() : nothing

kind(span) = isempty(span.attr.classes) ? "" : first(span.attr.classes)
has(span, class) = class in span.attr.classes

function to_tex(span::Span)
    if haskey(TEX, kind(span))
        text = has(span, "comment-start") ?
            stringify(walk!(pilcrows(() -> Str("\\newline")), span)) * "}\\hlnote{" :
            stringify(span.content) * "}"
        RawInline("latex", "$(TEX[kind(span)])[id=$(author(span))]{$text")
    elseif has(span, "comment-end")
        RawInline("latex", "}")
    end
end

function to_html(span::Span)
    if haskey(HTML, kind(span))
        author(span)
        s = "<" * HTML[kind(span)]
        for (key, value) in span.attr.attributes
            s *= " $(key == "date" ? key : "data-" * key)=\"$value\""
        end
        if has(span, "comment-start")
            s *= " data-id=\"$(span.attr.identifier)\""  # (Lua's "" is true: always)
            s *= " title=\"$(stringify(walk!(pilcrows(() -> Str("&#10;")), span)))\">"
        else
            s *= ">$(stringify(span.content))</$(HTML[kind(span)])>"
        end
        RawInline("html", s)
    elseif has(span, "comment-end")
        RawInline("html", "</mark>")
    end
end

to_word(span::Span) = has(span, "comment-start") ? walk!(pilcrows(LineBreak), span) : nothing

"Accepting or rejecting: comments go, an insertion becomes its content (accepted) or goes, a deletion the reverse; a span of both is an insertion."
function resolve(span, accept)
    (has(span, "comment-start") || has(span, "comment-end")) && return Inline[]
    has(span, "insertion") && return accept ? span.content : Inline[]
    has(span, "deletion") && return accept ? Inline[] : span.content
    nothing
end

"""`header-includes` with the `changes` package and a colour for each author:
the original wraps what is there in a list, a list too (its test for a list,
of `.t`, fails since pandoc 2.17)."""
function header_includes(existing)
    out = existing === nothing ? MetaValue[] : MetaValue[existing]
    push!(out, MetaBlocks(RawBlock("latex", HEADER)))
    for (i, inits) in enumerate(sort!(collect(keys(authors))))
        push!(out, MetaBlocks(RawBlock("latex", "\\definechangesauthor[name={$(authors[inits])}, color=auth$i]{$inits}")))
    end
    out
end

function track_changes(doc::Pandoc, ctx)
    # (the original reads the reader's option first, as pandoc 2 named it:
    # since pandoc 3 never there, so the metadata decides)
    mode = stringify(pop!(doc.meta, "trackChanges", MetaString("accept")))
    format = ctx.format
    tex = format in ("latex", "tex", "context")
    changes = if mode == "all"
        format in ("html", "html4", "html5") ? to_html : tex ? to_tex : format in ("docx", "odt") ? to_word : nothing
    elseif mode == "reject"
        (span::Span) -> resolve(span, false)
    else
        (span::Span) -> resolve(span, true)
    end
    changes === nothing || walk!(changes, doc.blocks)  # (not the metadata)
    if mode == "all" && tex
        doc.meta["header-includes"] = header_includes(get(doc.meta, "header-includes", nothing))
    end
    nothing
end

run_filter(track_changes)
