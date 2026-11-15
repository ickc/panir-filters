#!/usr/bin/env python3
"""Documents for the differential runs, made of every filter's cases, as
real documents rarely hold a filter's syntax: the blocks of each case's
input in a shuffled order, some put in a div, a quote or a list item,
three times over; and metadata of its own: each field one of the values
the cases give it, or missing (one time in four), and sometimes in another
shape (`reshaped`): text as the string a command line gives (`-M`), a
value as a list of it or a list of one as its item, a boolean where text
was, an entry of a map missing; the keys sorted (as pandoc writes them). Not the cases the original fails on, nor those a
`known.tsv` names: each would make every document differ for a reason
already known. Deterministic: the same cases make the same documents.

    python3 assemble.py [N] [--lang DOC...] [--amsthm DOC...]
                                writes .cache/assembled/mix-0.json ... mix-{N-1}.json (N = 8)
    python3 run.py diff --docs .cache/assembled/*.json

With `--lang DOC...` (pandoc JSON documents), also each document with an
`auto-lang` map of its own, twice: every Unicode script mapped
(`lang-all-NAME.json`), and every one but Latin (`lang-some-NAME.json`, so
that Latin text is another script's), each script to its name in lower
case: the real documents' text, in whatever scripts it has, for auto-lang,
which a document without the map leaves alone. With `--amsthm DOC...`,
each document with an `amsthm` map declaring every class of a div in it
an environment of the plain style, numbered within sections
(`amsthm-NAME.json`): a textbook's theorems, definitions and exercises,
read from LaTeX as divs, for amsthm.
"""

import json
import random
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def inputs() -> list[dict]:
    """Each case's input once, unless the original failed on it or a known
    difference names it."""
    out = []
    for path in sorted(ROOT.glob("*/cases/expected.jsonl")):
        cases = [json.loads(line) for line in path.read_text().splitlines()]
        left_out = {c["name"] for c in cases if c.get("error")}
        known = path.parent.parent / "known.tsv"
        if known.exists():
            rows = [line.split("\t") for line in known.read_text().splitlines() if line and not line.startswith("#")]
            left_out |= {name.split(" (")[0] for _, name, _ in rows}
        by_name = {c["name"]: c["input"] for c in cases if c["name"] not in left_out}
        out.extend(by_name[name] for name in sorted(by_name))
    return out


def placed(blocks: list, rng: random.Random) -> list:
    """The blocks as they are, or in a div, a quote or a list item."""
    where = rng.choice(["here", "here", "div", "quote", "item"])
    if where == "div":
        return [{"t": "Div", "c": [["", [], []], blocks]}]
    if where == "quote":
        return [{"t": "BlockQuote", "c": blocks}]
    if where == "item":
        return [{"t": "BulletList", "c": [blocks]}]
    return blocks


def text(value: object) -> str:
    """The text of metadata in pandoc's JSON, near enough (a string's, a
    `Str`'s, a space for a space)."""
    if isinstance(value, str):
        return value
    if isinstance(value, list):
        return "".join(map(text, value))
    if isinstance(value, dict):
        if value.get("t") in ("Space", "SoftBreak", "LineBreak"):
            return " "
        return text(value.get("c", ""))
    return ""


def reshaped(value: dict, rng: random.Random, top: bool) -> dict:
    """The value, or, one time in six, in another shape: inlines as a
    string (as `-M` gives), text as a boolean, a value as a list of it (a
    field only: a list in a list of authors is not a shape any filter here
    reads), a list of one as its item; a map's entries missing one time in
    ten. Not blocks for inlines: scholarly-metadata names an author given as
    blocks after Lua's tostring of them, a known difference."""
    t, c = value["t"], value.get("c")
    if t == "MetaMap":
        value = {"t": t, "c": {k: reshaped(v, rng, False) for k, v in c.items() if rng.random() >= 0.1}}
    elif t == "MetaList":
        value = {"t": t, "c": [reshaped(v, rng, False) for v in c]}
    roll = rng.random()
    if roll < 0.05 and t == "MetaInlines":
        return {"t": "MetaString", "c": text(c)}
    if roll < 0.09 and t in ("MetaInlines", "MetaString"):
        return {"t": "MetaBool", "c": rng.random() < 0.5}
    if roll < 0.13 and top and t != "MetaList":
        return {"t": "MetaList", "c": [value]}
    if roll < 0.17 and t == "MetaList" and len(value["c"]) == 1:
        return value["c"][0]
    return value


def mix(docs: list[dict], seed: int) -> dict:
    rng = random.Random(seed)
    given: dict[str, list] = {}  # each field's values, in the cases' order
    for doc in docs:
        for key, value in doc["meta"].items():
            given.setdefault(key, []).append(value)
    meta = {key: reshaped(rng.choice(values), rng, True) for key, values in given.items() if rng.random() >= 0.25}
    blocks = []
    for _ in range(3):
        for doc in rng.sample(docs, len(docs)):
            if doc["blocks"]:
                blocks.extend(placed(doc["blocks"], rng))
    return {"pandoc-api-version": docs[0]["pandoc-api-version"], "meta": dict(sorted(meta.items())), "blocks": blocks}


def scripts() -> list[str]:
    """The names of every script in auto-lang's table (the original's)."""
    table = (ROOT / "auto-lang" / "upstream" / "script-ranges.lua").read_text(encoding="utf-8")
    return re.findall(r"(\w+) = true", re.search(r"^  names = \{(.*)\},$", table, re.M)[1])


def with_languages(doc: dict, names: list[str]) -> dict:
    """The document with `auto-lang` mapping each script of `names` to its
    name in lower case."""
    languages = {name: {"t": "MetaString", "c": name.lower()} for name in names}
    return {**doc, "meta": dict(sorted({**doc["meta"], "auto-lang": {"t": "MetaMap", "c": languages}}.items()))}


def div_classes(value: object) -> set[str]:
    """The classes of every div in a document's JSON."""
    if isinstance(value, dict):
        own = set(value["c"][0][1]) if value.get("t") == "Div" else set()
        return own.union(*map(div_classes, value.values()))
    if isinstance(value, list):
        return set().union(*map(div_classes, value))
    return set()


def with_environments(doc: dict) -> dict:
    """The document with `amsthm` declaring its divs' classes, numbered
    within sections."""
    names = [{"t": "MetaInlines", "c": [{"t": "Str", "c": c}]} for c in sorted(div_classes(doc["blocks"]))]
    config = {"plain": {"t": "MetaList", "c": names}, "parent_counter": {"t": "MetaInlines", "c": [{"t": "Str", "c": "section"}]}}
    return {**doc, "meta": dict(sorted({**doc["meta"], "amsthm": {"t": "MetaMap", "c": config}}.items()))}


def options(args: list[str], name: str) -> list[str]:
    """The paths after `--name`, up to the next option."""
    if f"--{name}" not in args:
        return []
    rest = args[args.index(f"--{name}") + 1 :]
    return rest[: next((i for i, a in enumerate(rest) if a.startswith("--")), len(rest))]


def main() -> None:
    args = sys.argv[1:]
    real, theorems = options(args, "lang"), options(args, "amsthm")
    args = args[: next((i for i, a in enumerate(args) if a.startswith("--")), len(args))]
    n = int(args[0]) if args else 8
    out = ROOT / ".cache" / "assembled"
    out.mkdir(parents=True, exist_ok=True)
    docs = inputs()
    for seed in range(n):
        (out / f"mix-{seed}.json").write_text(json.dumps(mix(docs, seed), ensure_ascii=False))
    print(f"{n} documents from {len(docs)} cases in {out.relative_to(ROOT)}")
    names = scripts()
    for path in map(Path, real):
        doc = json.loads(path.read_text(encoding="utf-8"))
        name = path.parent.name if path.name == "doc.json" else path.stem
        for kind, mapped in (("all", names), ("some", [s for s in names if s != "Latin"])):
            (out / f"lang-{kind}-{name}.json").write_text(json.dumps(with_languages(doc, mapped), ensure_ascii=False))
    if real:
        print(f"{2 * len(real)} documents with an auto-lang map, of {len(real)}, in {out.relative_to(ROOT)}")
    for path in map(Path, theorems):
        doc = json.loads(path.read_text(encoding="utf-8"))
        name = path.parent.name if path.name == "doc.json" else path.stem
        (out / f"amsthm-{name}.json").write_text(json.dumps(with_environments(doc), ensure_ascii=False))
    if theorems:
        print(f"{len(theorems)} documents with an amsthm map in {out.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
