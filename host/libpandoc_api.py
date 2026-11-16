"""Run Python ports as filter functions through libpandoc-python's API, for
`run.py host` (mode `python-api`): each case is `libpandoc.convert` of its
document (pandoc's JSON) to its format, with its options (defaults-file
keys) and environment, and as filters the `panir.Filter`s its port's
script hands over (the script run anew for each case, as a new process
would), then a function that keeps the document as they left it.

Reads a job a line, `{"scripts": [...], "format", "options", "env",
"cwd", "input"}`, and perhaps `"sandbox"` (`{"read": [...]}`) (a chain's scripts in turn); writes a line a job, `output<TAB>`
the document's JSON, or `error<TAB>` the message (its newlines as `\\n`).
Needs a Python with libpandoc (and panir)."""

import json
import os
import runpy
import sys
from contextlib import contextmanager

import libpandoc
import panir
from panir.filter import handoff


def handed_over(script: str) -> list[panir.Filter]:
    """What the script's `main(...)` hands over, run as `python3 script`
    runs it (its folder first on the import path)."""
    handed: list[panir.Filter] = []
    handoff.set(handed.append)
    sys.path.insert(0, os.path.dirname(os.path.abspath(script)))
    try:
        runpy.run_path(script, run_name="__main__")
    finally:
        sys.path.pop(0)
    if not handed:
        raise RuntimeError(f"{script} hands no filter over")
    return handed


@contextmanager
def environment(variables: dict[str, str]):
    saved = {k: os.environ.get(k) for k in variables}
    os.environ.update(variables)
    try:
        yield
    finally:
        for k, v in saved.items():
            if v is None:
                os.environ.pop(k, None)
            else:
                os.environ[k] = v


def run(job: dict) -> str:
    kept: list[panir.Pandoc] = []

    def keep(doc: panir.Pandoc) -> None:
        kept.append(doc)

    here = os.getcwd()
    os.chdir(job["cwd"])  # (the case's files are named relative to it)
    try:
        with environment(job.get("env") or {}):
            filters = [f for script in job["scripts"] for f in handed_over(script)]
            options = {**(job.get("options") or {}), **({"sandbox": job["sandbox"]} if "sandbox" in job else {})}
            libpandoc.convert(json.dumps(job["input"]), options, from_="json", to=job["format"],
                              filters=[*filters, keep])
    finally:
        os.chdir(here)
    return panir.dumps(kept[0])


for line in sys.stdin:
    try:
        result = "output\t" + run(json.loads(line))
    except BaseException as e:  # (a port's sys.exit too)
        result = "error\t" + f"{type(e).__name__}: {e}".replace("\n", "\\n")
    print(result, flush=True)
