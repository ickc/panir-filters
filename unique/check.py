"""Run a Python port, then panir's check_unique on what it made (run.py
with CHECK_UNIQUE=1): `python3 unique/check.py PORT.py FORMAT`."""

import runpy
import sys

import panir
from panir.conversion import Conversion
from panir.filter import handoff


def take(f: panir.Filter) -> None:
    doc = f(panir.loads(sys.stdin.buffer.read()), conversion=Conversion.from_environment(sys.argv[1:]))
    panir.check_unique(doc)
    sys.stdout.write(panir.dumps(doc))


script = sys.argv.pop(1)
handoff.set(take)
runpy.run_path(script, run_name="__main__")
