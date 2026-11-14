-- Run after an original Lua filter (run.py): writes the document as the
-- filters before left it to stdout, as pandoc's JSON, and ends pandoc
-- there, so that the output format's writer (which the filters were run
-- for) doesn't run, nor count in the times.
function Pandoc(doc)
  io.stdout:write(pandoc.write(doc, "json"))
  io.stdout:flush()
  os.exit(0)
end
