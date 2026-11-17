#!/usr/bin/env julia
# A code block with an `include` attribute takes the file's text: a pandoc
# filter.
#
# `startLine` and `endLine` (or `start-line`, `end-line`) take a range of
# lines, the first giving `startFrom` (for `numberLines`); `dedent=n` removes
# the spaces among each line's first n bytes. The attributes it reads are
# removed. A file that can't be opened leaves the block empty, with a
# message.
#
# A port of pandoc/lua-filters' include-code-files (© 2020 Bruno BEAUFILS;
# MIT, see ../LICENSE).

using Panir

"Text as Lua's string functions take it for an integer argument."
function lua_integer(text, what)
    n = lua_number(text)
    n === nothing && error("bad argument to '$what' (number expected, got string)")
    isinteger(n) || error("bad argument to '$what' (number has no integer representation)")
    Int(n)
end

"Lua's `s:sub(i, j)`: bytes i to j, counted from 1, negative from the end."
function lua_sub(s::AbstractVector{UInt8}, i, j = -1)
    n = length(s)
    from = max(i < 0 ? n + i + 1 : i, 1)
    to = min(j < 0 ? n + j + 1 : j, n)
    from <= to ? s[from:to] : UInt8[]
end

"The spaces among the line's first n bytes removed (Lua's strings are bytes: an é is two)."
function dedent(line, n)
    k = lua_integer(n, "sub")
    [filter(!=(UInt8(' ')), lua_sub(line, 1, k)); lua_sub(line, k + 1)]
end

"Lua's `fh:lines(\"L\")`: each line with its newline (`\\n`)."
function lines(data::Vector{UInt8})
    out = Vector{UInt8}[]
    start = 1
    for i in findall(==(UInt8('\n')), data)
        push!(out, data[start:i])
        start = i + 1
    end
    start <= length(data) && push!(out, data[start:end])
    out
end

need(n) = n === nothing ? error("attempt to compare nil with number") : n

function transclude(cb::CodeBlock)
    attr = cb.attr
    path = get(attr, "include", nothing)
    path === nothing && return nothing
    content = UInt8[]
    # (Lua opens a directory, then fails to read it; Julia reads it as nothing)
    isdir(path) && error("$path: a directory")
    data = try
        read(path)
    catch
        println(stderr, "Cannot open file $path | Skipping includes")
        nothing
    end
    if data !== nothing
        # hyphenated attributes to camel case
        for (camel, hyphen) in (("startLine", "start-line"), ("endLine", "end-line"))
            value = get(attr, hyphen, nothing)
            value === nothing && continue
            attr[camel] = value
            delete!(attr, hyphen)
        end
        start = 1
        first = get(attr, "startLine", nothing)
        if first !== nothing
            attr["startFrom"] = first
            start = lua_number(first)
        end
        for (number, raw) in enumerate(lines(data))
            n = get(attr, "dedent", nothing)
            line = n === nothing ? raw : dedent(raw, n)
            if number >= need(start)
                last = get(attr, "endLine", nothing)
                (last === nothing || number <= need(lua_number(last))) && append!(content, line)
            end
        end
    end
    foreach(key -> delete!(attr, key), ("include", "startLine", "endLine", "dedent"))
    cb.text = String(content)
    nothing
end

run_filter(transclude)
