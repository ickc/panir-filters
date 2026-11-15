# Every Julia port's cases in one Julia process, for `run.py test`: a new
# process per case compiles Panir's walk again each time (minutes for all
# the cases). Each case runs its port's script anew, in a module of its
# own, so that a port's state (a counter, a flag) starts afresh as in a
# process of its own; the script's `run_filter` hands its function over
# (`Panir.handoff`, as a host running filter scripts does) instead of
# reading stdin.
#
# Reads lines `scripts<TAB>format<TAB>environment<TAB>document` (the scripts
# of a chain, run in turn, separated by the unit separator, \x1f; the case's
# environment variables, `KEY=VALUE` separated by \x1f, set while it runs;
# the document as pandoc's JSON, on one line); writes a line per case,
# `output<TAB>document` or `error<TAB>message` (the message's newlines as
# \n).

using Panir

function run_case(scripts, format, json)
    doc = Panir.parse(json)
    for script in scripts
        handed = Panir.handoff(() -> Base.include(Module(), script))
        handed === nothing && error("$script calls no run_filter")
        f, traverse = handed
        doc = walk!(f, doc; traverse, format = Panir.Conversion([format], Dict{String, String}()))
        get(ENV, "CHECK_UNIQUE", "") == "1" && check_unique(doc)  # (run.py's switch)
    end
    Panir.serialize(doc)
end

for line in eachline(stdin)
    scripts, format, environment, json = split(line, '\t'; limit = 4)
    variables = [String(k) => String(v) for (k, v) in split.(split(environment, '\x1f'; keepempty = false), '='; limit = 2)]
    result = try
        "output\t" * withenv(variables...) do
            run_case(String.(split(scripts, '\x1f')), String(format), String(json))
        end
    catch e
        "error\t" * replace(sprint(showerror, e), "\n" => "\\n")
    end
    println(result)
    flush(stdout)
end
