-- Run last by `run.py host` in a host's pandoc command (pandocpy,
-- pandocrs, pandocjl): writes the document as the filters before left it
-- to stdout, as pandoc's JSON. Unlike tojson.lua it doesn't end pandoc
-- (os.exit in a host's process ends the host), so the writer still runs,
-- its output going wherever -o says.
function Pandoc(doc)
  io.stdout:write(pandoc.write(doc, "json"))
  io.stdout:flush()
end
