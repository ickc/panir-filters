# Every Julia port's cases in one Julia process, for `run.py test`: a new
# process per case compiles Panir's walk again each time (minutes for all
# the cases). Each case runs its port's script anew, in a module of its
# own, so that a port's state (a counter, a flag) starts afresh as in a
# process of its own; the script's `run_filter` hands its function over
# (`Panir.handoff`, as a host running filter scripts does) instead of
# reading stdin.
#
# Reads lines `scripts<TAB>format<TAB>environment<TAB>options<TAB>document`
# (the scripts of a chain, run in turn, separated by the unit separator,
# \x1f; the case's environment variables, `KEY=VALUE` separated by \x1f, set
# while it runs; its conversion's options, as JSON, or nothing; the document
# as pandoc's JSON, on one line); writes a line per case,
# `output<TAB>document` or `error<TAB>message` (the message's newlines as
# \n).

using Panir

# A module of its own for a script, with `include` and `eval`, as `Main` has
# them for a script run by `julia` (auto-lang includes its table). Its code
# is compiled least (`compile = min`: run by the interpreter, mostly): a
# script's code is new in every case's module, so compiling it took most of
# each case's time (amsthm's 0.9 s; 0.45 s so), for documents of a few lines.
# Inference still runs (an empty comprehension's type depends on it:
# `infer = false` changed what deflists makes), so the script does what it
# does compiled; Panir's own code, compiled once, runs as a host runs it.
function fresh_module()
    m = Core.eval(Main, Expr(:module, true, gensym(:case), Expr(:block)))
    Core.eval(m, :(Base.Experimental.@compiler_options compile = min))
    m
end

function run_case(scripts, format, options, json)
    doc = Panir.parse(json)
    conversion = Panir.Conversion(; format, options = isempty(options) ? nothing : Panir.JSON.parse(options))
    for script in scripts
        handed = Panir.handoff(() -> Base.include(fresh_module(), script))
        handed === nothing && error("$script calls no run_filter")
        f, traverse = handed
        doc = walk!(f, doc; traverse, format = conversion)
        get(ENV, "CHECK_UNIQUE", "") == "1" && check_unique(doc)  # (run.py's switch)
    end
    Panir.serialize(doc)
end

for line in eachline(stdin)
    scripts, format, environment, options, json = split(line, '\t'; limit = 5)
    variables = [String(k) => String(v) for (k, v) in split.(split(environment, '\x1f'; keepempty = false), '='; limit = 2)]
    result = try
        "output\t" * withenv(variables...) do
            run_case(String.(split(scripts, '\x1f')), String(format), String(options), String(json))
        end
    catch e
        "error\t" * replace(sprint(showerror, e), "\n" => "\\n")
    end
    println(result)
    flush(stdout)
end
