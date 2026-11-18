# include-code-files

pandoc/lua-filters' include-code-files: a code block with an `include`
attribute takes the text of the file it names, or a range of its lines
(`startLine`, `endLine`, or `start-line`, `end-line`; `startFrom` set for
`numberLines`), its lines dedented (`dedent=n`); the attributes it reads
are removed. Round 2's other filter: it reads files, and calls no pandoc,
so its ports run everywhere, `run.py test` included.

The cases are the original's sample (which includes the filter's own
source, copied into `cases/files`) and nine of this repository's: ranges of
every shape (`ranges`: both spellings at once, past the end, backwards,
zero, negative, hexadecimal, a fraction, a `startFrom` given), dedent
(`dedent`: more than the line, zero, negative, `2.0`, hexadecimal, bytes
against characters), kinds of file (`files-kinds`: no final newline, empty,
CRLF, UTF-8), a missing file (`missing`), files outside the directory
through `..` and a symbolic link (`outside`), and five failures (a start or
end that is no number, a dedent that is none or a fraction, a directory).
They run in `cases/files`, the canaries are in `cases/outside`.

## The original's behaviour, kept

- **Lua's strings are bytes**: `dedent=n` removes the spaces among a
  line's first n *bytes* (`line:sub(1, n):gsub(" ", "")`), so `é  x` with
  `dedent=3` loses one space, not two (case `dedent`); each port works on
  the file's bytes and decodes at the end (Python's `bytes`, Node's
  `Buffer`, Rust's `Vec<u8>`, Julia's `Vector{UInt8}`), with Lua's
  `string.sub` for negative positions (`dedent=-2` removes every space but
  the newline's: `fourspaces`).
- **Lua's numbers**: `startLine` and `endLine` go through `tonumber`
  (`0x2` is 2, `3.5` is 3.5, so line 3 is the last); one that is no
  number fails, but only when a line is compared with it (an empty file
  doesn't fail); `dedent` goes to `string.sub` as text, which takes a
  number with an integer value (`2.0`, `0x3`) and fails on `abc` and
  `1.5`, again only on a line.
- **Attributes** are Lua's `attributes` by key: the hyphenated spelling
  replaces the camel-case one in place (or is added at the end), then is
  removed; `startFrom` is the start's text as written (`0x2`), replacing a
  given one in place; for a missing file nothing is renamed, so
  `start-line` stays (case `missing`); an empty `include` is a file that
  can't be opened.
- **Lines** are Lua's `fh:lines("L")`: split after each `\n`, CRLF kept as
  it is, the last line without one kept.
- **A directory named as a file**: Lua opens it and fails to read it
  (`error-directory`); each port tests for one and fails, as reading one
  does something else in each language.

## For upstream: what looks unintended

1. `dedent` counts bytes, not characters: a line with non-ASCII text
   before its spaces loses fewer than asked.
2. A negative `dedent` removes every space of the line but the last
   bytes'.
3. For a missing file the hyphenated range attributes are kept, though
   the camel-case ones are removed.

## By language

- **Python**: `bytes` throughout, `content.decode(errors="replace")` at
  the end (Lua hands pandoc the bytes); `IsADirectoryError` caught before
  `OSError`.
- **TypeScript**: `Buffer`s, `subarray` and `filter` for Lua's `sub` and
  `gsub` on bytes; `EISDIR` tested on the error.
- **Rust**: `split_inclusive` is Lua's `lines("L")`; `ctx.ok` turns each of
  Lua's errors (a comparison with `nil`, `sub`'s argument) into the
  walk's failure.
- **Julia**: the file as `Vector{UInt8}`; `String(content)` at the end.

Copies: none (`CHECK_UNIQUE=1`, every language).

## In the hosts

The same in every host (`run.py host`), but for what the host decides
(`known.tsv`, `HOSTS.md`). A wasm filter is given no directory unless
`LIBPANDOC_WASM_DIRS` names some (libpandoc 1.14's hosts; the working
directory here, `wasm-dirs: .`): given none, 9 of the 11 cases differ.
`outside` gives `.` and `../outside` (`outside.env`): the file through
`..` is read, the symbolic link leaving the working directory is refused
wherever it leads, and its include skipped, in the four wasm hosts. Under the
directory sandbox the Lua original, inside the module, fails on
`include=""` (case `missing`), where natively it skips it: in WASI an
empty name is the working directory, which `io.open` opens and `lines`
then can't read (`HOSTS.md`).
