#!/usr/bin/env python3
"""The test runner: each filter's ports against what its original made.

    python3 run.py test [FILTER...]            the cases (cases/expected.jsonl)
    python3 run.py diff --docs DOC... [FILTER...]
                                               each port against its original,
                                               on documents it wasn't written for
    python3 run.py expect [FILTER...]          regenerate cases/expected.jsonl
                                               with the original
    python3 run.py time FILTER --doc DOC       time and memory, original and ports

`--lang python,ts,rust,julia` picks languages (all by default). A port is
run as pandoc runs a JSON filter: the document on stdin, the output format
as the first argument, the document on stdout. Two documents are the same
when their JSON is ("the same AST"), `pandoc-api-version` aside. A filter
whose ORIGIN says `after: OTHER` runs after OTHER, always: the originals
one after the other, and each language's ports likewise.

A case may set environment variables (`NAME.env`, `KEY=VALUE` lines, kept
in `expected.jsonl` as `"env"`): the original and the ports run with them.
Its document may be read through another format (`NAME.via`, a format's
name: pandoc writes the markdown in it, then reads that, with `NAME.args`,
as track-changes' cases go through docx with `--track-changes=all`). It may
set the conversion's options (`NAME.opts`, a JSON object of pandoc's
defaults-file keys, kept in `expected.jsonl` as `"options"`): the writer's
options, which pandoc gives a Lua filter (`PANDOC_WRITER_OPTIONS`) but not a
JSON filter, and panir's conversion has where its host knows them. The
original runs with them (`pandoc -d NAME.opts`); the Python, TypeScript and
Julia ports are then run in process, as a host does (`host/`), with them in
`ctx.conversion.options`; the Rust ports, which run as JSON filters only,
without them.

`test` needs only the ports' toolchains; it runs the Julia ports' cases
in one Julia process (`julia-cases.jl`), the others a process per case.
`diff`, `expect` and `time` also run the originals: pandoc (`$PANDOC`) for
Lua ones, and for Python ones a Python with pandocfilters and panflute
(`$ORACLE_PYTHON`; see `oracle-requirements.txt`), whose output is taken
as pandoc reads it. Other variables: `$PANIR` (panir's checkout, default
`../panir`), `$PYTHON`, `$NODE`, `$JULIA`, `$TIMEOUT` (seconds a run may
take, 300). Rust ports are built (`cargo build --release`) before they
run.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shlex
import shutil
import subprocess
import sys
import tempfile
import threading
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent
LANGS = ("python", "ts", "rust", "julia")
env = os.environ.get


def filters(names: list[str]) -> list[Path]:
    found = sorted(p.parent.parent for p in ROOT.glob("*/upstream/ORIGIN"))
    if not names:
        return found
    by_name = {p.name: p for p in found}
    missing = [n for n in names if n not in by_name]
    if missing:
        sys.exit(f"no such filter: {', '.join(missing)}")
    return [by_name[n] for n in names]


def origin(folder: Path) -> dict[str, str]:
    """upstream/ORIGIN: `key: value` lines."""
    out = {}
    for line in (folder / "upstream" / "ORIGIN").read_text().splitlines():
        key, _, value = line.partition(":")
        if value:
            out[key.strip()] = value.strip()
    return out


def workdir(folder: Path) -> Path | None:
    """The directory a filter's original, ports and hosts run in, if its
    ORIGIN names one (`cwd: cases/files`): the files its cases name are
    there, relative to it."""
    cwd = origin(folder).get("cwd")
    return folder / cwd if cwd else None


def wasm_dirs(folder: Path) -> dict[str, str]:
    """What a filter's port built for wasm is given in a host: the
    directories its ORIGIN's `wasm-dirs` names (`wasm-dirs: .`, relative
    to its `cwd`; the chain's together), as `LIBPANDOC_WASM_DIRS`, a list
    as PATH's, which every host reads. None for a filter that names none:
    a wasm filter is given no directory by default, so its runs prove it
    needs none. A case may give more, its `NAME.env` setting the variable
    itself (include-files' `outside`)."""
    dirs = [d for f in chain(folder) for d in origin(f).get("wasm-dirs", "").split()]
    return {"LIBPANDOC_WASM_DIRS": os.pathsep.join(dict.fromkeys(dirs))} if dirs else {}


def needs_pandoc(folder: Path, case: dict | None = None) -> bool:
    """Whether a filter's ports call pandoc (`needs: pandoc` in its ORIGIN,
    round 2's and 3's), or, given a case, its ports do on that case
    (`NAME.needs` beside it, `"needs": "pandoc"` in `expected.jsonl`):
    then only `run.py host` runs them, `test` and `diff` leave them out."""
    return origin(folder).get("needs") == "pandoc" or (case or {}).get("needs") == "pandoc"


def chain(folder: Path) -> list[Path]:
    """The filters run for this one: those its ORIGIN's `after` names
    (each perhaps after others), then itself."""
    after = origin(folder).get("after")
    return [*chain(ROOT / after), folder] if after else [folder]


def ports(folder: Path, lang: str, options: bool = False) -> list[list[str]] | None:
    """The commands running a filter's port in a language, after the ports
    of the filters it runs after; None if one has no port there. With
    `options` (a case's), in process where a language can be."""
    cmds = [port(f, lang, options) for f in chain(folder)]
    return None if None in cmds else cmds  # type: ignore[return-value]


def port(folder: Path, lang: str, options: bool = False) -> list[str] | None:
    """The command running a filter's port in a language, if it has one."""
    name = folder.name
    # in process, as a host does (host/): for a case's writer options, and,
    # with CHECK_UNIQUE=1, panir's check that no node is in two places, on
    # what each port made (Rust's ownership needs none)
    hosted = options or env("CHECK_UNIQUE") == "1"
    if lang == "python":
        script = folder / "python" / f"{name}.py"
        wrap = [str(ROOT / "host" / "python.py")] if hosted else []
        return [env("PYTHON", "python3"), *wrap, str(script)] if script.exists() else None
    if lang == "ts":
        script = folder / "ts" / f"{name}.ts"
        wrap = ["--import", str(ROOT / "host" / "register.mjs")] if hosted else []
        return [env("NODE", "node"), *wrap, str(script)] if script.exists() else None
    if lang == "rust":
        if not (folder / "rust" / "Cargo.toml").exists():
            return None
        return [str(ROOT / "target" / "release" / name)]
    if lang == "julia":
        script = folder / "julia" / f"{name}.jl"
        if not script.exists():
            return None
        return [env("JULIA", "julia"), f"--project={ROOT}", "--startup-file=no", str(script)]
    raise ValueError(lang)


def port_env() -> dict[str, str]:
    panir = Path(env("PANIR", ROOT.parent / "panir")).resolve()
    return {**os.environ, "PYTHONPATH": str(panir / "python" / "src")}


def original(folder: Path) -> list[str]:
    """The command running the original as a JSON filter would run; a Lua
    one after the Lua originals it runs after, in one pandoc, as a user
    runs them (`-L a.lua -L b.lua`)."""
    how = origin(folder)["run"].split()
    upstream = folder / "upstream"
    if how[0] == "python":
        if len(chain(folder)) > 1:
            raise ValueError(f"{folder.name}: `after` is for Lua originals")
        if how[1] == "-m":  # (a package's module, installed in $ORACLE_PYTHON)
            return [env("ORACLE_PYTHON", "python3"), "-m", how[2]]
        return [env("ORACLE_PYTHON", "python3"), str(upstream / how[1])]
    if how[0] == "lua":
        lua = []
        for f in chain(folder):
            run_f = origin(f)["run"].split()
            if run_f[0] != "lua":
                raise ValueError(f"{folder.name}: `after` is for Lua originals")
            lua += ["-L", str(f / "upstream" / run_f[1])]
        # pandoc runs the Lua filters for the format, and a last one writes
        # the document as JSON to stdout (pandoc's own output goes nowhere)
        return [
            env("PANDOC", "pandoc"), "--sandbox", "-f", "json", "-o", os.devnull,
            *lua, "-L", str(ROOT / "tojson.lua"), "-t",
        ]
    raise ValueError(f"{folder.name}: unknown run: {how}")


class Ran:
    def __init__(self, code: int, out: bytes, err: bytes, seconds: float, rss_kb: int):
        self.code, self.out, self.err, self.seconds, self.rss_kb = code, out, err, seconds, rss_kb


def run(cmd: list[str], data: bytes, cwd: Path | None = None, environ: dict | None = None, runs: int = 1) -> Ran:
    """Run a command on `data`, with its time and peak memory (its own,
    from wait4); stopped after `$TIMEOUT` seconds a run (`runs` of them:
    julia-cases.jl runs every case)."""
    start = time.perf_counter()
    p = subprocess.Popen(cmd, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                         stderr=subprocess.PIPE, cwd=cwd, env=environ)
    out: list[bytes] = []
    err: list[bytes] = []
    readers = [threading.Thread(target=lambda: out.append(p.stdout.read())),
               threading.Thread(target=lambda: err.append(p.stderr.read()))]
    for r in readers:
        r.start()
    # a filter that never ends (one walking its own output, say) is stopped
    timer = threading.Timer(float(env("TIMEOUT", "300")) * runs, p.kill)
    timer.start()
    try:
        p.stdin.write(data)
        p.stdin.close()
    except BrokenPipeError:
        pass
    _, status, usage = os.wait4(p.pid, 0)
    seconds = time.perf_counter() - start
    timer.cancel()
    p.returncode = os.waitstatus_to_exitcode(status)
    for r in readers:
        r.join()
    return Ran(p.returncode, out[0], err[0], seconds, usage.ru_maxrss)


def run_original(folder: Path, data: bytes, fmt: str, as_read: bool = True, environ: dict | None = None,
                 defaults: Path | None = None) -> Ran:
    """The original's output; with `as_read`, as pandoc reads it (a Lua
    filter's always is): JSON that pandoc reads as the same AST is the same
    output (pandocfilters writes `{"t": "DisplayMath", "c": []}`, pandoc
    `{"t": "DisplayMath"}`), and JSON pandoc can't read is a failure, as it
    is when pandoc runs the filter."""
    cmd = original(folder)
    environ = {**os.environ, **(environ or {})}
    if origin(folder)["run"].startswith("lua"):
        options = ["-d", str(defaults)] if defaults else []  # (the conversion's options)
        return run(cmd[:-1] + options + [cmd[-1], fmt], data, cwd=workdir(folder), environ=environ)
    if defaults:
        raise ValueError(f"{folder.name}: options are for Lua originals")
    ran = run(cmd + [fmt], data, cwd=workdir(folder) or folder / "upstream", environ=environ)
    if not as_read or ran.code != 0:
        return ran
    read = run([env("PANDOC", "pandoc"), "--sandbox", "-f", "json", "-t", "json"], ran.out)
    if read.code != 0:
        read.err = b"pandoc could not read the original's output: " + read.err
    return read


def run_port(cmds: list[list[str]], data: bytes, fmt: str, environ: dict | None = None, cwd: Path | None = None) -> Ran:
    """The ports of a chain, each on what the one before made, with the
    case's environment variables: the time is theirs together, the memory
    the most any took."""
    total = Ran(0, data, b"", 0.0, 0)
    for cmd in cmds:
        ran = run(cmd + [fmt], total.out, cwd=cwd, environ={**port_env(), **(environ or {})})
        total = Ran(ran.code, ran.out, ran.err, total.seconds + ran.seconds, max(total.rss_kb, ran.rss_kb))
        if ran.code != 0:
            break
    return total


def ast(text: bytes) -> object:
    doc = json.loads(text)
    doc.pop("pandoc-api-version", None)
    return doc


def first_difference(a: object, b: object, path: str = "") -> str | None:
    """Where two JSON values first differ, as a path."""
    if type(a) is not type(b) and not (isinstance(a, (int, float)) and isinstance(b, (int, float))):
        return f"{path or '.'}: {short(a)} != {short(b)}"
    if isinstance(a, dict):
        for k in sorted(set(a) | set(b)):
            if k not in a or k not in b:
                return f"{path}.{k}: missing on one side"
            d = first_difference(a[k], b[k], f"{path}.{k}")
            if d:
                return d
        return None
    if isinstance(a, list):
        for i, (x, y) in enumerate(zip(a, b)):
            d = first_difference(x, y, f"{path}[{i}]")
            if d:
                return d
        if len(a) != len(b):
            return f"{path}: length {len(a)} != {len(b)}"
        return None
    return None if a == b else f"{path or '.'}: {short(a)} != {short(b)}"


def short(x: object) -> str:
    s = json.dumps(x, ensure_ascii=False)
    return s if len(s) < 120 else s[:117] + "..."


def langs_of(arg: str | None) -> list[str]:
    langs = arg.split(",") if arg else list(LANGS)
    bad = [x for x in langs if x not in LANGS]
    if bad:
        sys.exit(f"unknown language: {', '.join(bad)}")
    return langs


def build(langs: list[str]) -> None:
    if "rust" in langs and (ROOT / "Cargo.toml").exists():
        subprocess.run(["cargo", "build", "--release", "--quiet"], cwd=ROOT, check=True)


def writer_options(options: dict) -> dict:
    """The writer's options libpandoc gives each filter it runs
    (`$PANDOC_WRITER_OPTIONS`: Lua's values, under pandoc's JSON keys) for a
    case's options (defaults-file keys), as far as a case sets them: what a
    JSON filter run by libpandoc is told (plain pandoc tells it nothing).
    `run.py host` runs the ports through libpandoc itself, which checks
    this."""
    reader = {"tab-stop", "default-image-extension", "indented-code-classes"}  # (not the writer's)
    unknown = set(options) - reader - {"number-sections", "top-level-division", "number-offset", "variables"}
    if unknown:
        raise ValueError(f"writer_options: {', '.join(sorted(unknown))} not known here")
    return {
        "number-sections": bool(options.get("number-sections", False)),
        "top-level-division": "top-level-" + options.get("top-level-division", "default"),
        "number-offset": list(options.get("number-offset", [])),
        "variables": {k: str(v) for k, v in options.get("variables", {}).items()},
    }


def check(cmds: list[list[str]] | Ran, case: dict, label: str, cwd: Path | None = None) -> str | None:
    """None if the port (its commands, or what it made) makes what the
    original made, else what differs."""
    if isinstance(cmds, Ran):
        ran = cmds
    else:
        environ = {**case.get("env", {}), **({"PANIR_FILTERS_OPTIONS": json.dumps(case["options"]),
                                              "PANDOC_WRITER_OPTIONS": json.dumps(writer_options(case["options"]))}
                                             if "options" in case else {})}
        ran = run_port(cmds, json.dumps(case["input"]).encode(), case["format"], environ, cwd)
    if case.get("error"):
        return None if ran.code != 0 else f"{label}: the original failed, the port didn't"
    if ran.code != 0:
        return f"{label}: exit {ran.code}\n{ran.err.decode(errors='replace')[-2000:]}"
    want = dict(case["output"])
    want.pop("pandoc-api-version", None)
    try:
        got = ast(ran.out)
    except json.JSONDecodeError as e:
        return f"{label}: not JSON ({e})"
    if got != want:
        return f"{label}: {first_difference(got, want)} (port != original)"
    return None


def expected(folder: Path) -> list[dict]:
    """cases/expected.jsonl: a case a line (split at newlines only: a
    string may hold U+2028 or U+0085, which `str.splitlines` splits at)."""
    text = (folder / "cases" / "expected.jsonl").read_text()
    return [json.loads(line) for line in text.split("\n") if line]


def known(folder: Path) -> dict[tuple[str, str], str]:
    """known.tsv: differences explained in NOTES.md, as `language, case or
    document, why`; the case or document perhaps with a format, `name
    (format)`, if it differs in that one only; the language perhaps with a
    mode, `python@standalone` (`run.py test` and `diff`), `rust@pandocrs-wasm`
    (a host of `run.py host`), if it differs there only, and perhaps with
    the run, `julia@pandocjl@host` (`run.py host`) or `@sandbox` (`run.py
    host --sandbox`). They don't fail a run; one that no longer differs
    does."""
    path = folder / "known.tsv"
    if not path.exists():
        return {}
    rows = [line.split("\t") for line in path.read_text().splitlines() if line and not line.startswith("#")]
    return {(lang, name): why for lang, name, why in rows}


class Tally:
    def __init__(self, verbose: bool):
        self.verbose, self.passed, self.failed, self.known = verbose, 0, 0, 0

    def add(self, folder: Path, lang: str, name: str, fmt: str, label: str, problem: str | None,
            mode: str = "standalone", run: str | None = None) -> None:
        """A run's result; `known.tsv`'s rows for its language, mode and run
        (`julia@pandocjl@host`: `run.py host` but not `--sandbox`, which is
        `@sandbox`), then for its language and mode (`rust@pandocrs-wasm`),
        then for its language in every mode (`rust`)."""
        listed = known(folder)
        whos = ([f"{lang}@{mode}@{run}"] if run else []) + [f"{lang}@{mode}", lang]
        keys = [(who, case) for who in whos for case in (f"{name} ({fmt})", name)]
        why = next((listed[k] for k in keys if k in listed), None)
        if problem and why:
            self.known += 1
            print("known", label, "--", why)
        elif problem:
            self.failed += 1
            print("FAIL", problem)
        elif why:
            self.failed += 1
            print("FAIL", label, "is the same now: remove it from known.tsv")
        else:
            self.passed += 1
            if self.verbose:
                print("ok  ", label)

    def done(self) -> int:
        print(f"{self.passed} passed, {self.failed} failed, {self.known} known differences")
        return 1 if self.failed else 0


def julia_in_one(jobs: list[tuple[list[str], dict, Path | None]]) -> list[Ran]:
    """What each Julia port made of its case, `(scripts, case, directory)`
    (a chain's scripts in turn; the directory it runs in, if not this one),
    all in one Julia process (julia-cases.jl): a process per case would
    compile Panir's walk each time."""
    lines = "".join(f"{chr(0x1f).join(scripts)}\t{case['format']}\t{chr(0x1f).join(f'{k}={v}' for k, v in case.get('env', {}).items())}\t"
                    f"{json.dumps(case['options']) if 'options' in case else ''}\t{cwd or ROOT}\t"
                    f"{json.dumps(case['input'], ensure_ascii=False)}\n"
                    for scripts, case, cwd in jobs)
    driver = [env("JULIA", "julia"), f"--project={ROOT}", "--startup-file=no", str(ROOT / "julia-cases.jl")]
    ran = run(driver, lines.encode(), environ=port_env(), runs=len(jobs))
    results = ran.out.decode().split("\n")[:-1]  # (newlines only: an output may hold U+2028)
    if ran.code != 0 or len(results) != len(jobs):
        sys.exit(f"julia-cases.jl failed (exit {ran.code}): {ran.err.decode(errors='replace')[-2000:]}")
    out = []
    for line in results:
        kind, _, rest = line.partition("\t")
        if kind == "output":
            out.append(Ran(0, rest.encode(), b"", 0, 0))
        else:
            out.append(Ran(1, b"", rest.replace("\\n", "\n").encode(), 0, 0))
    return out


def cmd_test(args: argparse.Namespace) -> int:
    langs = langs_of(args.lang)
    build(langs)
    tally = Tally(args.verbose)
    runs = []  # (folder, lang, case, the port's command, or what it made)
    for folder in filters(args.filters):
        cases = expected(folder)
        for lang in langs:
            for case in cases:
                if needs_pandoc(folder, case):
                    continue
                cmds = ports(folder, lang, "options" in case)
                if cmds is not None:
                    runs.append((folder, lang, case, cmds))
    julia = [i for i, (_, lang, _, _) in enumerate(runs) if lang == "julia"]
    if julia:
        made = julia_in_one([([cmd[-1] for cmd in runs[i][3]], runs[i][2], workdir(runs[i][0])) for i in julia])
        for i, ran in zip(julia, made):
            runs[i] = (*runs[i][:3], ran)
    for folder, lang, case, cmd in runs:
        label = f"{folder.name} {lang} {case['name']} ({case['format']})"
        tally.add(folder, lang, case["name"], case["format"], label, check(cmd, case, label, workdir(folder)))
    return tally.done()


def document(path: Path) -> bytes:
    """A document's JSON: a .json file as it is, anything else read by
    pandoc as markdown, with the options in a file beside it of the same
    name and `.args` (`-M lang=de`, `-f markdown-smart`), if any; or, if a
    `.via` beside it names a format (`docx`), written in that format by
    pandoc (with SOURCE_DATE_EPOCH=0), then read from it with the options
    (`--track-changes=all`)."""
    if path.suffix == ".json":
        return path.read_bytes()
    pandoc = env("PANDOC", "pandoc")
    args = path.with_suffix(".args")
    extra = shlex.split(args.read_text()) if args.exists() else []
    via = path.with_suffix(".via")
    with tempfile.TemporaryDirectory() as tmp:
        source, reader = str(path), ["-f", "markdown"]
        if via.exists():
            fmt = via.read_text().strip()
            source = str(Path(tmp) / f"{path.stem}.{fmt}")
            wrote = run([pandoc, "--sandbox", "-f", "markdown", "-t", fmt, "-o", source, str(path)], b"",
                        environ={**os.environ, "SOURCE_DATE_EPOCH": "0"})
            if wrote.code != 0:
                sys.exit(f"pandoc could not write {path} as {fmt}: {wrote.err.decode()}")
            reader = ["-f", fmt]
        ran = run([pandoc, "--sandbox", *reader, *extra, "-t", "json", source], b"")
    if ran.code != 0:
        sys.exit(f"pandoc could not read {path}: {ran.err.decode()}")
    return ran.out


def case_env(path: Path) -> dict[str, str]:
    """`NAME.env` beside a case: `KEY=VALUE` lines, the environment the
    original and the ports run the case with (pagebreak reads some)."""
    file = path.with_suffix(".env")
    if not file.exists():
        return {}
    pairs = (line.partition("=") for line in file.read_text().splitlines() if line.strip())
    return {key: value for key, _, value in pairs}


def formats(folder: Path) -> list[str]:
    return origin(folder).get("formats", "html").split()


def doc_name(path: Path) -> str:
    return path.parent.name if path.name == "doc.json" else path.name


def cmd_diff(args: argparse.Namespace) -> int:
    langs = langs_of(args.lang)
    build(langs)
    docs = [(doc_name(Path(d)), document(Path(d))) for d in args.docs]
    tally = Tally(args.verbose)
    for folder in filters(args.filters):
        if needs_pandoc(folder):
            print(f"note: {folder.name} calls pandoc: run.py host runs its ports")
            continue
        for name, data in docs:
            for fmt in formats(folder):
                want = run_original(folder, data, fmt)
                case = {"name": name, "format": fmt, "input": json.loads(data)}
                if want.code != 0:
                    case["error"] = True
                    print(f"note {folder.name} {name} ({fmt}): the original failed: "
                          f"{want.err.decode(errors='replace').strip()[-300:]}")
                else:
                    case["output"] = json.loads(want.out)
                for lang in langs:
                    cmds = ports(folder, lang)
                    if cmds is None:
                        continue
                    label = f"{folder.name} {lang} {name} ({fmt})"
                    tally.add(folder, lang, name, fmt, label, check(cmds, case, label, workdir(folder)))
    return tally.done()


# The hosts `run.py host` runs the ports in: a mode each, its language, and
# what it runs. A command (`pandocpy`, `pandocrs`, `pandocjl`: pandoc's
# command line run in process) gets `-F` for each port of the chain, then
# host/capture.lua, which writes the document as the ports left it; a
# driver (host/) runs every case in one process, through the host's API.
HOSTS = {
    "pandocpy": ("python", "a Python port as a filter script, run in process by pandocpy (-F port.py)"),
    "python-api": ("python", "a Python port's filters as functions, libpandoc.convert(filters=[...]) (host/libpandoc_api.py)"),
    "pandocrs-wasm": ("rust", "a Rust port built for wasm32-wasip1, run in process by pandocrs (-F port.wasm)"),
    "pandocrs-native": ("rust", "a Rust port as a JSON filter, a program pandocrs (libpandoc) runs (-F port)"),
    "pandocpy-wasm": ("rust", "the same wasm filter in pandocpy"),
    "pandocjl-wasm": ("rust", "the same wasm filter in pandocjl"),
    "node-wasm": ("rust", "the same wasm filter in libpandoc.wasm's Node host (wasm-filter.mjs; host/libpandoc-node.mjs)"),
    "pandocjl": ("julia", "a Julia port as a filter script, run in process by pandocjl (-F port.jl), a process a case"),
    "node": ("ts", "a TypeScript port's filter as a JS function in libpandoc.wasm's Node host (host/libpandoc-node.mjs)"),
    # the Lua originals, for the directory sandbox (inside it, as the document)
    "pandocpy-lua": ("lua", "a Lua original in pandocpy (-L), for the sandbox"),
    "pandocrs-lua": ("lua", "a Lua original in pandocrs (-L), for the sandbox"),
    "pandocjl-lua": ("lua", "a Lua original in pandocjl (-L), for the sandbox"),
}

# The modes running a port built for wasm, given what its filter states
# (`wasm_dirs`).
WASM = ("pandocrs-wasm", "pandocpy-wasm", "pandocjl-wasm", "node-wasm")

# The modes `host --sandbox` runs.
# (Not a JSON filter: the sandbox refuses it, status 83, documented.)
SANDBOXED = ["pandocpy-lua", "pandocrs-lua", "pandocjl-lua", "pandocpy", "python-api", "pandocrs-wasm",
             "pandocpy-wasm", "pandocjl-wasm", "pandocjl"]
# What a case of the sandbox's reads from outside its directory: each
# canary's marker (cases/outside/), by how it is named
CANARIES = re.compile(rb"(?:CANARY|SECRET)-(DOTDOT|LINK|ABSOLUTE)")


def host_command(mode: str) -> list[str] | None:
    """The command of a mode's host, from the environment (None: not
    given, the mode is skipped): `$PANDOCPY`, `$PANDOCRS`, `$PANDOCJL` (a
    command line: `julia -J IMAGE --project=DIR -m LibPandoc`), the drivers'
    `$HOST_PYTHON` (a Python with libpandoc) and `$LIBPANDOC_NODE` (libpandoc's
    wasm/node.mjs, with `$LIBPANDOC_WASM`)."""
    if mode in ("pandocpy", "pandocpy-wasm", "pandocpy-lua"):
        return [env("PANDOCPY")] if env("PANDOCPY") else None
    if mode in ("pandocrs-wasm", "pandocrs-native", "pandocrs-lua"):
        return [env("PANDOCRS")] if env("PANDOCRS") else None
    if mode in ("pandocjl", "pandocjl-wasm", "pandocjl-lua"):
        return shlex.split(env("PANDOCJL")) if env("PANDOCJL") else None
    if mode == "python-api":
        return [env("HOST_PYTHON"), str(ROOT / "host" / "libpandoc_api.py")] if env("HOST_PYTHON") else None
    if mode in ("node", "node-wasm"):
        ready = env("LIBPANDOC_NODE") and env("LIBPANDOC_WASM")
        return [env("NODE", "node"), "--no-warnings", str(ROOT / "host" / "libpandoc-node.mjs")] if ready else None
    raise ValueError(mode)


def host_filters(folder: Path, mode: str) -> list[str] | None:
    """The files a mode runs for a filter's chain (None if a port is
    missing)."""
    out = []
    for f in chain(folder):
        name = f.name
        if mode in ("pandocpy", "python-api"):
            path = f / "python" / f"{name}.py"
        elif mode == "pandocrs-native":
            path = ROOT / "target" / "release" / name
        elif HOSTS[mode][0] == "rust":
            path = ROOT / "target" / "wasm32-wasip1" / "release" / f"{name}.wasm"
        elif mode == "pandocjl":
            path = f / "julia" / f"{name}.jl"
        elif mode == "node":
            path = f / "ts" / f"{name}.ts"
        elif HOSTS[mode][0] == "lua":
            how = origin(f)["run"].split()
            if how[0] != "lua":
                return None
            path = f / "upstream" / how[1]
        else:
            raise ValueError(mode)
        if HOSTS[mode][0] != "lua" and not (f / HOSTS[mode][0]).exists():
            return None
        out.append(str(path))
    return out


def in_host(cmd: list[str], files: list[str], case: dict, cwd: Path | None = None, sandbox: bool = False) -> Ran:
    """A case through a host's pandoc command: its document (pandoc's JSON)
    to its format, with its options as a defaults file and its environment,
    the ports' files as filters (Lua ones with -L), then host/capture.lua.
    With `sandbox`, in the host's directory sandbox: the directory `cwd`
    read-only, and a temporary one holding the document, the options, the
    Lua filters (pandoc reads them, inside) and the output, read-write."""
    with tempfile.TemporaryDirectory() as tmp:
        doc = Path(tmp) / "input.json"
        doc.write_text(json.dumps(case["input"]))
        options = []
        if "options" in case:
            (Path(tmp) / "options.json").write_text(json.dumps(case["options"]))
            options = ["-d", str(Path(tmp) / "options.json")]
        lua = [*(f for f in files if f.endswith(".lua")), str(ROOT / "host" / "capture.lua")]
        if sandbox:
            lua = [shutil.copy(f, Path(tmp) / f"{i}-{Path(f).name}") for i, f in enumerate(lua)]
        filters = [x for f in files if not f.endswith(".lua") for x in ("-F", f)]
        filters += [x for f in lua[:-1] for x in ("-L", f)]
        out = Path(tmp) / "out" if sandbox else os.devnull
        args = [*cmd, *options, "-f", "json", "-t", case["format"], "-o", str(out), *filters, "-L", lua[-1], str(doc)]
        environ = {**os.environ, **case.get("env", {})}
        if cmd[0].endswith("julia") and "JULIA_LOAD_PATH" not in os.environ:
            # a Julia script's packages (pantable's YAML.jl...) are loaded from
            # pandocjl's environment: this project's after it (HOSTS.md, H16)
            environ["JULIA_LOAD_PATH"] = os.pathsep.join(["@", str(ROOT), "@stdlib"])
        if sandbox:
            environ |= {"LIBPANDOC_SANDBOX": str(cwd or tmp), "LIBPANDOC_SANDBOX_WRITE": tmp}
        return run(args, b"", cwd=cwd or (Path(tmp) if sandbox else None), environ=environ)


def in_driver(cmd: list[str], jobs: list[dict]) -> list[Ran]:
    """Jobs through a driver of host/, all in one process: a line each,
    `{"scripts" or "wasm": [...], "format", "options", "env", "input"}`."""
    lines = "".join(json.dumps(job, ensure_ascii=False) + "\n" for job in jobs)
    ran = run(cmd, lines.encode(), environ=os.environ.copy(), runs=len(jobs))
    results = ran.out.decode().split("\n")[:-1]  # (newlines only: an output may hold U+2028)
    if ran.code != 0 or len(results) != len(jobs):
        sys.exit(f"{cmd[-1]} failed (exit {ran.code}): {ran.err.decode(errors='replace')[-2000:]}")
    out = []
    for line in results:
        kind, _, rest = line.partition("\t")
        out.append(Ran(0, rest.encode(), b"", 0, 0) if kind == "output"
                   else Ran(1, b"", rest.replace("\\n", "\n").encode(), 0, 0))
    return out


def absolute_probe(folder: Path, case: dict) -> dict:
    """The case `outside` with its `../outside/NAME.EXT` named absolutely,
    as `cases/outside/NAME-absolute.EXT` (a canary of its own), for the
    sandbox: an absolute name is no case to commit."""
    outside = (folder / "cases" / "outside").resolve()
    text = re.sub(r"\.\./outside/([\w-]+)\.(\w+)", lambda m: f"{outside}/{m[1]}-absolute.{m[2]}", json.dumps(case["input"]))
    return {**case, "name": "outside-absolute", "input": json.loads(text)}


def cmd_host(args: argparse.Namespace) -> int:
    langs = langs_of(args.lang)
    if args.host:
        modes = args.host.split(",")
    elif args.sandbox:
        modes = [m for m in SANDBOXED if HOSTS[m][0] in (*langs, "lua")]
    else:
        modes = [m for m, (lang, _) in HOSTS.items() if lang in langs]
    bad = [m for m in modes if m not in HOSTS]
    if bad:
        sys.exit(f"unknown host: {', '.join(bad)} (one of {', '.join(HOSTS)})")
    build(["rust"] if any(HOSTS[m][0] == "rust" for m in modes) else [])
    if any(HOSTS[m][0] == "rust" and m != "pandocrs-native" for m in modes):
        subprocess.run(["cargo", "build", "--release", "--quiet", "--target", "wasm32-wasip1"], cwd=ROOT, check=True)
    tally = Tally(args.verbose)
    probes = []  # (filter, mode, case, what it read from outside), with --sandbox
    for mode in modes:
        lang = HOSTS[mode][0]
        cmd = host_command(mode)
        if cmd is None:
            print(f"note: {mode} skipped (its host is not given: run.py host's docstring)")
            continue
        runs = []  # (folder, case, the files run)
        for folder in filters(args.filters):
            if args.sandbox and workdir(folder) is None:
                continue
            files = host_filters(folder, mode)
            if files is None:
                continue
            if mode == "pandocrs-native" and needs_pandoc(folder):
                print(f"note: {folder.name} rust@{mode} skipped: a JSON filter has no pandoc to call (NOTES.md)")
                continue
            skipped = 0
            given = wasm_dirs(folder) if mode in WASM else {}
            for case in expected(folder):
                if mode == "pandocrs-native" and needs_pandoc(folder, case):
                    skipped += 1
                    continue
                if given:  # (the case's own environment after: it may give more)
                    case = {**case, "env": {**given, **case.get("env", {})}}
                runs.append((folder, case, files))
                if args.sandbox and case["name"] == "outside":
                    runs.append((folder, absolute_probe(folder, case), files))
            if skipped:
                print(f"note: {skipped} cases of {folder.name} skipped in {mode}: a JSON filter has no pandoc to call")
        if mode in ("python-api", "node", "node-wasm"):
            key = "wasm" if mode == "node-wasm" else "scripts"
            jobs = [{key: files, "format": case["format"], "options": case.get("options", {}),
                     "env": case.get("env", {}), "cwd": str(workdir(folder) or ROOT), "input": case["input"],
                     **({"sandbox": {"read": [str(workdir(folder))]}} if args.sandbox else {})}
                    for folder, case, files in runs]
            made = in_driver(cmd, jobs)
        else:
            made = [in_host(cmd, files, case, workdir(folder), args.sandbox) for folder, case, files in runs]
        for (folder, case, _), ran in zip(runs, made):
            label = f"{folder.name} {lang}@{mode} {case['name']} ({case['format']})"
            if args.sandbox and case["name"].startswith("outside"):
                read = sorted(set(m.decode() for m in CANARIES.findall(ran.out)))
                what = f"failed: {ran.err.decode(errors='replace').strip()[-160:]}" if ran.code else ", ".join(read) or "nothing"
                probes.append((folder.name, mode, case["name"], what))
                continue
            tally.add(folder, lang, case["name"], case["format"], label, check(ran, case, label), mode,
                      "sandbox" if args.sandbox else "host")
    if probes:
        print("\nWhat each read from outside its directory (cases outside and outside-absolute):")
        print("| filter | host | case | read |\n|---|---|---|---|")
        for row in probes:
            print("| " + " | ".join(row).replace("\n", " ") + " |")
    return tally.done()


def cmd_expect(args: argparse.Namespace) -> int:
    for folder in filters(args.filters):
        lines = []
        sources = (folder / "cases").glob("*.md"), (folder / "cases").glob("*.json")
        for md in sorted(p for found in sources for p in found):
            data = document(md)
            environ = case_env(md)
            defaults = md.with_suffix(".opts")
            options = {"options": json.loads(defaults.read_text())} if defaults.exists() else {}
            needs = md.with_suffix(".needs")
            needs = {"needs": needs.read_text().strip()} if needs.exists() else {}
            for fmt in formats(folder):
                ran = run_original(folder, data, fmt, environ=environ, defaults=defaults if options else None)
                case = {"name": md.stem, "format": fmt, **({"env": environ} if environ else {}), **options, **needs,
                        "input": json.loads(data)}
                if ran.code != 0:
                    case["error"] = True
                    print(f"{folder.name} {md.stem} ({fmt}): the original fails (expected so)")
                else:
                    case["output"] = json.loads(ran.out)
                lines.append(json.dumps(case, ensure_ascii=False, separators=(",", ":")))
        (folder / "cases" / "expected.jsonl").write_text("".join(x + "\n" for x in lines))
        print(f"{folder.name}: {len(lines)} cases")
    return 0


def cmd_time(args: argparse.Namespace) -> int:
    langs = langs_of(args.lang)
    build(langs)
    (folder,) = filters([args.filter])
    data = document(Path(args.doc))
    fmt = args.format or formats(folder)[0]
    rows = [("original", lambda: run_original(folder, data, fmt, as_read=False))]
    for lang in langs:
        cmds = ports(folder, lang)
        if cmds is not None:
            rows.append((lang, lambda cmds=cmds: run_port(cmds, data, fmt, cwd=workdir(folder))))
    print(f"{folder.name} on {doc_name(Path(args.doc))} ({len(data) / 1e6:.1f} MB of JSON, {fmt}), "
          f"best of {args.repeat}")
    print("| | seconds | peak MB |\n|---|---|---|")
    for label, go in rows:
        best = None
        for _ in range(args.repeat):
            ran = go()
            if ran.code != 0:
                print(f"| {label} | failed: {ran.err.decode(errors='replace')[-200:]} | |")
                break
            if best is None or ran.seconds < best.seconds:
                best = ran
        if best:
            print(f"| {label} | {best.seconds:.2f} | {best.rss_kb / 1024:.0f} |")
    return 0


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = ap.add_subparsers(dest="command", required=True)
    for name in ("test", "diff", "expect", "host"):
        p = sub.add_parser(name)
        p.add_argument("filters", nargs="*")
        if name != "expect":
            p.add_argument("--lang")
            p.add_argument("-v", "--verbose", action="store_true")
        if name == "host":
            p.add_argument("--host", help="modes, comma-separated: " + ", ".join(HOSTS))
            p.add_argument("--sandbox", action="store_true",
                           help="in the hosts' directory sandbox, the filters that read files")
        if name == "diff":
            p.add_argument("--docs", nargs="+", required=True)
    p = sub.add_parser("time")
    p.add_argument("filter")
    p.add_argument("--doc", required=True)
    p.add_argument("--format")
    p.add_argument("--lang")
    p.add_argument("--repeat", type=int, default=3)
    args = ap.parse_args()
    command = {"test": cmd_test, "diff": cmd_diff, "expect": cmd_expect, "host": cmd_host, "time": cmd_time}[args.command]
    # a wasm filter is given what its filter states (wasm_dirs), not the
    # directories of the caller's LIBPANDOC_WASM_DIRS
    os.environ.pop("LIBPANDOC_WASM_DIRS", None)
    if not env("PANDOC"):
        return command(args)
    # a port or an original running the `pandoc` command runs this one: a
    # directory of that one name first on PATH (not $PANDOC's own directory,
    # whose other programs, a python3 among them, would come first too)
    with tempfile.TemporaryDirectory() as shim:
        (Path(shim) / "pandoc").symlink_to(Path(env("PANDOC")).resolve())
        os.environ["PATH"] = os.pathsep.join([shim, env("PATH", "")])
        return command(args)


if __name__ == "__main__":
    sys.exit(main())
