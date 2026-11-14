# Every Julia port's cases in one Julia process, for `run.py test`: a new
# process per case compiles Panir's walk again each time (minutes for all
# the cases). Each case runs its port's script anew, in a module of its
# own, so that a port's state (a counter, a flag) starts afresh as in a
# process of its own; the script's `run_filter` hands its function over
# (`Panir.handoff`, as a host running filter scripts does) instead of
# reading stdin.
#
# Reads lines `script<TAB>format<TAB>document` (the document as pandoc's
# JSON, on one line); writes a line per case, `output<TAB>document` or
# `error<TAB>message` (the message's newlines as \n).

using Panir

function run_case(script, format, json)
    handed = Panir.handoff(() -> Base.include(Module(), script))
    handed === nothing && error("$script calls no run_filter")
    f, traverse = handed
    doc = Panir.parse(json)
    doc = walk!(f, doc; traverse, format = Panir.Conversion([format], Dict{String, String}()))
    get(ENV, "CHECK_UNIQUE", "") == "1" && check_unique(doc)  # (run.py's switch)
    Panir.serialize(doc)
end

for line in eachline(stdin)
    script, format, json = split(line, '\t'; limit = 3)
    result = try
        "output\t" * run_case(String(script), String(format), String(json))
    catch e
        "error\t" * replace(sprint(showerror, e), "\n" => "\\n")
    end
    println(result)
    flush(stdout)
end
