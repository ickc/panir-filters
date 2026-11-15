"""Run a Python port in process, as a host running filter scripts does
(libpandoc), instead of as a JSON filter: `python3 host/python.py PORT.py
FORMAT`, for run.py. The conversion is the JSON filter's, with the
conversion's options (defaults-file keys) from `$PANIR_FILTERS_OPTIONS`, a
case's writer options, which a JSON filter is not given; with
`CHECK_UNIQUE=1`, panir's check_unique on what the port made."""

import json
import os
import runpy
import sys

import panir
from panir.conversion import Conversion
from panir.filter import handoff

script = sys.argv.pop(1)
sys.path.insert(0, os.path.dirname(os.path.abspath(script)))  # as `python3 script.py` has it
handed: list[panir.Filter] = []  # (main([f, g]) hands each over, in turn)
handoff.set(handed.append)
runpy.run_path(script, run_name="__main__")
conversion = Conversion.from_environment(sys.argv[1:])
if options := os.environ.get("PANIR_FILTERS_OPTIONS"):
    conversion.options = json.loads(options)
doc = panir.run(panir.loads(sys.stdin.buffer.read()), handed, conversion=conversion)
if os.environ.get("CHECK_UNIQUE") == "1":
    panir.check_unique(doc)
sys.stdout.write(panir.dumps(doc))
