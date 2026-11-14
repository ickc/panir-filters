#!/bin/sh
# Fetch pandoc's own test documents (markdown) into .cache/pandoc-tests, for
# the differential runs: python3 run.py diff --docs .cache/pandoc-tests/*
# They are data from pandoc's repository (GPL): used as inputs, never
# committed (.cache is ignored).
set -eu
cd "$(dirname "$0")"
mkdir -p .cache/pandoc-tests
for f in testsuite.txt markdown-reader-more.txt markdown-citations.txt tables.markdown pipe-tables.txt; do
  curl -fsSL -o ".cache/pandoc-tests/$f" "https://raw.githubusercontent.com/jgm/pandoc/3.14/test/$f"
done
ls .cache/pandoc-tests
