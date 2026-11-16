#!/usr/bin/env julia
# Code blocks of class `include` replaced by the files they name: a pandoc
# filter.
#
# Each line of the block names a file (`//` starts a comment); each is read
# by pandoc, as markdown or as the block's `format`, with the document's
# reader options, and its blocks (not its metadata) take the block's place.
# Headings shift by `shift-heading-level-by`, or, with `include-auto` in the
# metadata, by the level of the last heading before the block. Included
# files may include others, named relative to their own directory; relative
# links, images and `include` attributes (include-code-files') in an
# included file are rewritten relative to the including one.
#
# Reading the files is Julia's; parsing them is pandoc's (`pandoc.read` in
# the original): LibPandoc.jl's `read`, in this process: the pandoc the
# filter runs in under pandocjl or `LibPandoc.convert`, or LibPandoc as a
# library when pandoc runs this as a JSON filter.
#
# A port of pandoc-ext/include-files (© 2019–2021 Albert Krewinkel; MIT,
# see ../LICENSE).

using Panir
import LibPandoc

"A number as Lua's `tonumber` reads text, decimal or hexadecimal, blanks around it; `nothing` if not one."
function lua_number(text)
    t = strip(text, [' ', '\t', '\n', '\v', '\f', '\r'])
    occursin(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$", t) && return parse(Float64, t)
    m = match(r"^([+-]?)0[xX]([0-9a-fA-F]+)$", t)
    m === nothing ? nothing : (m[1] == "-" ? -1 : 1) * parse(Int, m[2]; base = 16)
end

# pandoc.path's functions, as pandoc has them (Haskell's System.FilePath,
# POSIX): `normalize` drops `.` and doubled slashes but keeps `..`, unlike
# Julia's normpath.
is_relative(path) = !startswith(path, '/')
filename(path) = last(split(path, '/'))
function directory(path)
    i = findlast('/', path)
    i === nothing && return isempty(path) ? "" : "."
    dir = rstrip(path[1:i], '/')
    isempty(dir) ? "/" : String(dir)
end
join_path(a, b) = startswith(b, '/') || isempty(a) ? b : isempty(b) ? a : endswith(a, '/') ? a * b : "$a/$b"
function normalize(path)
    rest = lstrip(path, '/')
    parts = filter(p -> !isempty(p) && p != ".", split(rest, '/'))
    out = (startswith(path, '/') ? "/" : "") * join(parts, '/')
    isempty(out) && return "."
    is_dir = endswith(rest, '/') || (endswith(rest, '.') && endswith(chop(rest), '/'))
    is_dir && !endswith(out, '/') ? out * "/" : out
end

const auto = Ref(false)  # (include-auto)
const last_level = Ref(0)  # the level of the last heading met

# (for each document: a host may run this filter on many, where Lua loads the
# original anew for each)
configure(meta::Panir.Meta) = (auto[] = truthy(get(meta, "include-auto", nothing)); last_level[] = 0; nothing)

"Headings shifted; relative paths taken to the including file's directory."
struct Update
    shift_by::Union{Real, Nothing}
    include_path::String
end

function (u::Update)(h::Header)
    u.shift_by === nothing && return nothing
    level = h.level + u.shift_by
    # (Lua's error, as pandoc's Header takes an integer)
    isinteger(level) || error("shift-heading-level-by: $(u.shift_by): number has no integer representation")
    h.level = Int(level)
    nothing
end

function (u::Update)(link::Link)
    url = link.target.url
    if is_relative(url) && !startswith(filename(url), '#') && !occursin(r"https?://", url)
        link.target.url = normalize(join_path(u.include_path, url))
    end
    nothing
end

(u::Update)(image::Image) =
    (is_relative(image.target.url) && (image.target.url = normalize(join_path(u.include_path, image.target.url))); nothing)

function (u::Update)(cb::CodeBlock)
    path = get(cb.attr, "include", nothing)
    path !== nothing && is_relative(path) && (cb.attr["include"] = normalize(join_path(u.include_path, path)))
    nothing
end

"""The filter including files, for blocks of a file in `base` (the original
changes the working directory to it: here the names are joined to it, as the
working directory is the process's)."""
struct Transclude
    base::String
end

(::Transclude)(h::Header) = (last_level[] = h.level; nothing)

function (t::Transclude)(cb::CodeBlock, ctx)
    "include" in cb.attr.classes || return nothing
    format = something(get(cb.attr, "format", nothing), "markdown")
    format = isempty(format) ? "markdown" : format
    shift_input = get(cb.attr, "shift-heading-level-by", nothing)
    shift_by = shift_input !== nothing ? lua_number(shift_input) : auto[] ? last_level[] : 0
    before = last_level[]
    blocks = Block[]
    for line in split(cb.text, '\n'; keepempty = false)
        startswith(line, "//") && continue
        path = join_path(t.base, line)
        # (Lua opens a directory, and pandoc fails to read nothing; Julia reads it as "")
        isdir(path) && error("$line: a directory")
        text = try
            read(path, String)
        catch
            println(stderr, "Cannot open file $line | Skipping includes")
            continue
        end
        contents = LibPandoc.read(text, ctx.conversion; from = format).blocks
        last_level[] = 0
        walk!(Transclude(join_path(t.base, directory(line))), contents; format = ctx.conversion)
        last_level[] = before
        append!(blocks, walk!(Update(shift_by, directory(line)), contents))
    end
    blocks
end

run_filter([configure, Transclude("")])
