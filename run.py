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
as track-changes' cases go through docx with `--track-changes=all`).

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
import shlex
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


def chain(folder: Path) -> list[Path]:
    """The filters run for this one: those its ORIGIN's `after` names
    (each perhaps after others), then itself."""
    after = origin(folder).get("after")
    return [*chain(ROOT / after), folder] if after else [folder]


def ports(folder: Path, lang: str) -> list[list[str]] | None:
    """The commands running a filter's port in a language, after the ports
    of the filters it runs after; None if one has no port there."""
    cmds = [port(f, lang) for f in chain(folder)]
    return None if None in cmds else cmds  # type: ignore[return-value]


def port(folder: Path, lang: str) -> list[str] | None:
    """The command running a filter's port in a language, if it has one."""
    name = folder.name
    # CHECK_UNIQUE=1: panir's check that no node is in two places, on what
    # each port made (Rust's ownership needs none)
    check = env("CHECK_UNIQUE") == "1"
    if lang == "python":
        script = folder / "python" / f"{name}.py"
        wrap = [str(ROOT / "unique" / "check.py")] if check else []
        return [env("PYTHON", "python3"), *wrap, str(script)] if script.exists() else None
    if lang == "ts":
        script = folder / "ts" / f"{name}.ts"
        wrap = ["--import", str(ROOT / "unique" / "check.mjs")] if check else []
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


def run(cmd: list[str], data: bytes, cwd: Path | None = None, environ: dict | None = None) -> Ran:
    """Run a command on `data`, with its time and peak memory (its own,
    from wait4)."""
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
    timer = threading.Timer(float(env("TIMEOUT", "300")), p.kill)
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


def run_original(folder: Path, data: bytes, fmt: str, as_read: bool = True, environ: dict | None = None) -> Ran:
    """The original's output; with `as_read`, as pandoc reads it (a Lua
    filter's always is): JSON that pandoc reads as the same AST is the same
    output (pandocfilters writes `{"t": "DisplayMath", "c": []}`, pandoc
    `{"t": "DisplayMath"}`), and JSON pandoc can't read is a failure, as it
    is when pandoc runs the filter."""
    cmd = original(folder)
    environ = {**os.environ, **(environ or {})}
    if origin(folder)["run"].startswith("lua"):
        return run(cmd + [fmt], data, environ=environ)
    ran = run(cmd + [fmt], data, cwd=folder / "upstream", environ=environ)
    if not as_read or ran.code != 0:
        return ran
    read = run([env("PANDOC", "pandoc"), "--sandbox", "-f", "json", "-t", "json"], ran.out)
    if read.code != 0:
        read.err = b"pandoc could not read the original's output: " + read.err
    return read


def run_port(cmds: list[list[str]], data: bytes, fmt: str, environ: dict | None = None) -> Ran:
    """The ports of a chain, each on what the one before made, with the
    case's environment variables: the time is theirs together, the memory
    the most any took."""
    total = Ran(0, data, b"", 0.0, 0)
    for cmd in cmds:
        ran = run(cmd + [fmt], total.out, environ={**port_env(), **(environ or {})})
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


def check(cmds: list[list[str]] | Ran, case: dict, label: str) -> str | None:
    """None if the port (its commands, or what it made) makes what the
    original made, else what differs."""
    ran = cmds if isinstance(cmds, Ran) else run_port(cmds, json.dumps(case["input"]).encode(), case["format"], case.get("env"))
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


def known(folder: Path) -> dict[tuple[str, str], str]:
    """known.tsv: differences explained in NOTES.md, as `language, case or
    document, why`; the case or document perhaps with a format, `name
    (format)`, if it differs in that one only. They don't fail a run; one
    that no longer differs does."""
    path = folder / "known.tsv"
    if not path.exists():
        return {}
    rows = [line.split("\t") for line in path.read_text().splitlines() if line and not line.startswith("#")]
    return {(lang, name): why for lang, name, why in rows}


class Tally:
    def __init__(self, verbose: bool):
        self.verbose, self.passed, self.failed, self.known = verbose, 0, 0, 0

    def add(self, folder: Path, lang: str, name: str, fmt: str, label: str, problem: str | None) -> None:
        listed = known(folder)
        why = listed.get((lang, f"{name} ({fmt})")) or listed.get((lang, name))
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


def julia_in_one(jobs: list[tuple[list[str], dict]]) -> list[Ran]:
    """What each Julia port made of its case, `(scripts, case)` (a chain's
    scripts in turn), all in one Julia process (julia-cases.jl): a process
    per case would compile Panir's walk each time."""
    lines = "".join(f"{chr(0x1f).join(scripts)}\t{case['format']}\t{chr(0x1f).join(f'{k}={v}' for k, v in case.get('env', {}).items())}\t"
                    f"{json.dumps(case['input'], ensure_ascii=False)}\n"
                    for scripts, case in jobs)
    driver = [env("JULIA", "julia"), f"--project={ROOT}", "--startup-file=no", str(ROOT / "julia-cases.jl")]
    ran = run(driver, lines.encode(), environ=port_env())
    results = ran.out.decode().splitlines()
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
        cases = [json.loads(line) for line in (folder / "cases" / "expected.jsonl").read_text().splitlines()]
        for lang in langs:
            cmds = ports(folder, lang)
            if cmds is not None:
                runs.extend((folder, lang, case, cmds) for case in cases)
    julia = [i for i, (_, lang, _, _) in enumerate(runs) if lang == "julia"]
    if julia:
        made = julia_in_one([([cmd[-1] for cmd in runs[i][3]], runs[i][2]) for i in julia])
        for i, ran in zip(julia, made):
            runs[i] = (*runs[i][:3], ran)
    for folder, lang, case, cmd in runs:
        label = f"{folder.name} {lang} {case['name']} ({case['format']})"
        tally.add(folder, lang, case["name"], case["format"], label, check(cmd, case, label))
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
                    tally.add(folder, lang, name, fmt, label, check(cmds, case, label))
    return tally.done()


def cmd_expect(args: argparse.Namespace) -> int:
    for folder in filters(args.filters):
        lines = []
        sources = (folder / "cases").glob("*.md"), (folder / "cases").glob("*.json")
        for md in sorted(p for found in sources for p in found):
            data = document(md)
            environ = case_env(md)
            for fmt in formats(folder):
                ran = run_original(folder, data, fmt, environ=environ)
                case = {"name": md.stem, "format": fmt, **({"env": environ} if environ else {}), "input": json.loads(data)}
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
            rows.append((lang, lambda cmds=cmds: run_port(cmds, data, fmt)))
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
    for name in ("test", "diff", "expect"):
        p = sub.add_parser(name)
        p.add_argument("filters", nargs="*")
        if name != "expect":
            p.add_argument("--lang")
            p.add_argument("-v", "--verbose", action="store_true")
        if name == "diff":
            p.add_argument("--docs", nargs="+", required=True)
    p = sub.add_parser("time")
    p.add_argument("filter")
    p.add_argument("--doc", required=True)
    p.add_argument("--format")
    p.add_argument("--lang")
    p.add_argument("--repeat", type=int, default=3)
    args = ap.parse_args()
    return {"test": cmd_test, "diff": cmd_diff, "expect": cmd_expect, "time": cmd_time}[args.command](args)


if __name__ == "__main__":
    sys.exit(main())
