"""Run a Python port, then panir's check_unique on what it made (run.py
with CHECK_UNIQUE=1): `python3 unique/check.py PORT.py FORMAT`."""

import runpy
import sys

import panir
from panir.conversion import Conversion
from panir.filter import handoff

script = sys.argv.pop(1)
handed: list[panir.Filter] = []  # (main([f, g]) hands each over, in turn)
handoff.set(handed.append)
runpy.run_path(script, run_name="__main__")
conversion = Conversion.from_environment(sys.argv[1:])
doc = panir.run(panir.loads(sys.stdin.buffer.read()), handed, conversion=conversion)
panir.check_unique(doc)
sys.stdout.write(panir.dumps(doc))
