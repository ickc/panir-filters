#!/usr/bin/env python3
"""Documents for the differential runs, made of every filter's cases, as
real documents rarely hold a filter's syntax: the blocks of each case's
input in a shuffled order, some put in a div, a quote or a list item,
three times over; the metadata of all, the first case with a key giving
it, the keys sorted (as pandoc writes them). Not the cases the original
fails on, nor those a `known.tsv` names: each would make every document
differ for a reason already known. Deterministic: the same cases make the
same documents.

    python3 assemble.py [N]     writes .cache/assembled/mix-0.json ... mix-{N-1}.json (N = 5)
    python3 run.py diff --docs .cache/assembled/*.json
"""

import json
import random
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


def mix(docs: list[dict], seed: int) -> dict:
    rng = random.Random(seed)
    meta: dict = {}
    for doc in docs:
        for key, value in doc["meta"].items():
            meta.setdefault(key, value)
    blocks = []
    for _ in range(3):
        for doc in rng.sample(docs, len(docs)):
            if doc["blocks"]:
                blocks.extend(placed(doc["blocks"], rng))
    return {"pandoc-api-version": docs[0]["pandoc-api-version"], "meta": dict(sorted(meta.items())), "blocks": blocks}


def main() -> None:
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 5
    out = ROOT / ".cache" / "assembled"
    out.mkdir(parents=True, exist_ok=True)
    docs = inputs()
    for seed in range(n):
        (out / f"mix-{seed}.json").write_text(json.dumps(mix(docs, seed), ensure_ascii=False))
    print(f"{n} documents from {len(docs)} cases in {out.relative_to(ROOT)}")


if __name__ == "__main__":
    main()
