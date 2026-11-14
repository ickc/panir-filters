#!/usr/bin/env julia
# Normalise the authors and institutes of a document's metadata: a pandoc
# filter.
#
# `author` and `institute` become lists of maps, each with a `name` and an
# `id`; an author's `institute` becomes the numbers of its institutes in
# `institute` (each given an `index`), which collects the top-level ones
# and those given in an author. An author or institute may be a name, a map
# with a `name`, or a map of one entry, its key the name and its value the
# rest (or the name); text with commas is a list of names; an author refers
# to an institute by identifier, by name or by number.
#
# A port of pandoc/lua-filters' scholarly-metadata (Copyright (c) 2017-2021
# Albert Krewinkel, Robert Winkler; ISC; the repository's MIT licence, see
# ../LICENSE).

using Panir

# A named object, as the original's Lua table (a reference, as a Lua table).
const Named = Dict{String, MetaValue}

const LUA_SPACE = (' ', '\t', '\n', '\v', '\f', '\r')  # Lua's %s (and its tonumber's)

"Whether Lua counts the value: anything but missing or false."
given(::Nothing) = false
given(v::MetaBool) = v.value
given(::MetaValue) = true

"A number as Lua's `tonumber` reads a string (decimal only), else `nothing`."
number(text) = occursin(r"^[ \t\n\v\f\r]*[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?[ \t\n\v\f\r]*$", text) ?
               parse(Float64, strip(text, LUA_SPACE)) : nothing

"A list as the original takes one: a list's items, text with commas split there
(each a string), anything else alone (nothing if missing)."
as_list(::Nothing) = MetaValue[]
as_list(v::MetaList) = copy(v.content)
function as_list(v::MetaInlines)
    names = [strip(name, LUA_SPACE) for name in split(stringify(v), ',')]
    length(names) >= 2 ? MetaValue[MetaString(name) for name in names] : MetaValue[v]
end
as_list(v::MetaValue) = MetaValue[v]

"An author or institute as a map with a name and an identifier."
named(obj::MetaInlines) = Named("name" => obj, "id" => MetaString(stringify(obj)))
function named(obj::Union{MetaString, MetaBool})
    text = stringify(obj)  # (Lua's tostring)
    Named("name" => MetaInlines([Str(text)]), "id" => MetaString(text))
end
function named(obj::MetaMap)
    entries = obj.content
    if haskey(entries, "name")
        out = copy(entries)
        out["id"] = MetaString(stringify(given(get(out, "id", nothing)) ? out["id"] : out["name"]))
        return out
    elseif length(entries) == 1
        ((key, value),) = entries
        out = attributes(key, value)
        out["id"] = given(get(out, "id", nothing)) ? MetaString(stringify(out["id"])) : MetaString(key)
        return out
    end
    error("scholarly-metadata: not a named object: $(stringify(obj))")
end
# a list or blocks, which the original names after Lua's tostring of them (NOTES.md)
named(obj::MetaValue) = error("scholarly-metadata: not a named object: $(stringify(obj))")

"The rest of a named object of one entry: its value, a name or a map."
attributes(_, value::Union{MetaString, MetaInlines}) = Named("name" => value)
function attributes(key, value::MetaMap)
    out = copy(value.content)
    given(get(out, "name", nothing)) || (out["name"] = MetaInlines([Str(key)]))
    out
end
attributes(key, ::MetaValue) = error("scholarly-metadata: $key: neither a name nor a map")

ident(x::Named) = stringify(x["id"])  # (a string's text)
find_institute(institutes, key) = findfirst(x -> ident(x) == key, institutes)

"An author's institutes: by number (a string's), by identifier, or new."
function resolve(refs, institutes)
    map(refs) do ref
        n = ref isa MetaString ? number(ref.text) : nothing
        n !== nothing && isinteger(n) && 1 <= n <= length(institutes) && return institutes[Int(n)]
        i = find_institute(institutes, stringify(ref))
        i === nothing ? named(ref) : institutes[i]
    end
end

"Add an institute, or what it has the one of its identifier lacks."
function add_institute!(institutes, inst)
    i = find_institute(institutes, ident(inst))
    i === nothing && return push!(institutes, inst)
    for (key, value) in inst
        # a copy: inst may stay too (a second top-level one, referred to by number)
        given(get(institutes[i], key, nothing)) || (institutes[i][key] = deepcopy(value))
    end
end

function canonicalize(meta::Panir.Meta)
    institutes = Named[named(x) for x in as_list(get(meta, "institute", nothing))]
    authors = Named[named(x) for x in as_list(get(meta, "author", nothing))]
    theirs = [resolve(as_list(get(author, "institute", nothing)), institutes) for author in authors]
    foreach(inst -> add_institute!(institutes, inst), Iterators.flatten(theirs))
    numbers = Dict{String, String}()  # each identifier's first institute
    for (i, inst) in enumerate(institutes)
        inst["index"] = MetaInlines([Str(string(i))])
        get!(numbers, ident(inst), string(i))
    end
    for (author, insts) in zip(authors, theirs)
        author["institute"] = [numbers[ident(x)] for x in insts]
    end
    meta["author"] = authors
    meta["institute"] = institutes
    nothing
end

run_filter(canonicalize)
