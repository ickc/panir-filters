#!/usr/bin/env python3
"""The ports' rule for a note of plain text, against pandoc's markdown
reader: random notes the rule takes (words of letters, digits, characters
that are not ASCII, hyphens inside, `,;:?!` after, single spaces between),
each read by pandoc (`$PANDOC`, 3.14) as a paragraph, must be their words
as `Str`s and single `Space`s, nothing else, as the ports make them.

    python3 amsthm/plain-notes.py [N]     (N = 3000)
"""

import json
import os
import random
import re
import subprocess
import sys
from pathlib import Path

PORT = Path(__file__).resolve().parent / "python" / "amsthm.py"
# the Python port's rule, as written there (the others are the same rule)
PLAIN_NOTE = re.compile(re.search(r'PLAIN_NOTE = re\.compile\(rf"(.*)"\)', PORT.read_text())[1].replace(
    "{PLAIN.pattern}", re.search(r'PLAIN = re\.compile\(r"(.*)"\)', PORT.read_text())[1]))

# ASCII letters and digits, Roman numerals' letters, letters of other
# scripts, a combining mark, and punctuation and symbols that are not ASCII
LETTERS = "abcxyzABCIVXMDL0123456789éßαβγ漢字한글אבעبتΩ" + "".join(map(chr, (
    0x1F600, 0x2603, 0xBF, 0xAB, 0xBB, 0x2014, 0x2019, 0x201C, 0x301, 0x200B, 0xB7, 0x2026, 0xFF01)))


def main() -> int:
    n = int(sys.argv[1]) if len(sys.argv) > 1 else 3000
    rng = random.Random(1)

    def part() -> str:
        return "".join(rng.choice(LETTERS) for _ in range(rng.randint(1, 4)))

    def word() -> str:
        w = part()
        while rng.random() < 0.2:
            w += "-" + part()
        return w + (rng.choice(",;:?!") if rng.random() < 0.3 else "")

    notes: list[str] = []
    while len(notes) < n:
        note = " ".join(word() for _ in range(rng.randint(1, 5)))
        if PLAIN_NOTE.fullmatch(note):
            notes.append(note)
    pandoc = os.environ.get("PANDOC", "pandoc")
    read = subprocess.run([pandoc, "--sandbox", "-f", "markdown", "-t", "json"], input="\n\n".join(notes).encode(),
                          capture_output=True, check=True)
    blocks = json.loads(read.stdout)["blocks"]
    differ = 0
    for note, block in zip(notes, blocks, strict=True):
        words = note.split(" ")
        want = [x for i, w in enumerate(words) for x in ([{"t": "Space"}] if i else []) + [{"t": "Str", "c": w}]]
        if block != {"t": "Para", "c": want}:
            differ += 1
            print(f"differs: {note!r}: {json.dumps(block, ensure_ascii=False)[:200]}")
    print(f"{n} notes read by pandoc: {differ} differ")
    return 1 if differ else 0


if __name__ == "__main__":
    sys.exit(main())
