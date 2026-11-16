# include-files

pandoc-ext/include-files: a code block of class `include` is replaced by
the files its lines name, each read by pandoc (`pandoc.read`, as markdown
or as the block's `format`, with the document's reader options), its
headings shifted, its relative paths rewritten; included files include
others, relative to their own directory. Round 2's filter (it reads files)
and the first of round 3 (it parses them with pandoc): the ports read the
files themselves, and get the reader from the host they run in.

The cases are the original's sample, run as its Makefile runs it (`sample`,
and `sample-auto` with `-M include-auto`), and thirteen of this
repository's: nested includes in subdirectories, links and images of every
shape in an included file (`nested`), a missing file and comments
(`missing`), files outside the directory, through `..` and through a
symbolic link (`outside`), other formats (`formats`: org, rst, commonmark,
markdown without `smart`, html, latex), shifts (`shift`, `shift-auto`,
`shift-auto-false`), metadata dropped (`meta`), the document's reader
options (`reader-options`, with `NAME.opts`), includes inside quotes, divs
and lists (`structure`), and three failures (a directory, an unknown
format, a shift of 1.5). They run in `cases/files` (`cwd:` in `ORIGIN`),
the canaries are in `cases/outside`.

## How each port gets pandoc's reader

| | the reader | under plain pandoc (a JSON filter) |
|---|---|---|
| Python | `libpandoc.read(text, options=libpandoc.read_options(ctx.conversion, format))`: libpandoc in this process (pandocpy, `libpandoc.convert`) | libpandoc as a library in the filter's process, if installed; else it fails at start, saying so |
| TypeScript | the `pandoc` command, a process a file (`execFileSync`), as pandocfilters and panflute filters do: no JavaScript host gives a filter the pandoc it runs in (`HOSTS.md`) | the same |
| Rust | built for wasm32-wasip1, libpandoc-rs's `libpandoc::read_many` with `read_options(conversion)` and the format: calls to the host (pandocrs, pandocpy, pandocjl, libpandoc.wasm) | built natively it has none: it fails at the first file it would read, saying to build it for wasm (`run.py host` doesn't run it as `pandocrs-native`) |
| Julia | `LibPandoc.read(text, ctx.conversion; from = format)`: LibPandoc.jl in this process (pandocjl, `LibPandoc.convert`) | LibPandoc as a library, if in the environment; else Julia's "package not found" |

`ORIGIN`'s `needs: pandoc` keeps `run.py test` (and CI, which has no
pandoc) from running the ports; `run.py host` runs them, in each host
(`HOSTS.md`).

## The original's behaviour, kept

- **The working directory.** The original opens each name relative to
  pandoc's working directory, and for an included file's own includes
  changes it to that file's directory (`pandoc.system.with_working_directory`).
  The ports join the names to the including file's directory instead:
  the same files, without changing a working directory that in an
  in-process host is the whole process's (and that WASI doesn't have).
- **`pandoc.read`'s reader options**: the format's own extensions, not the
  document's (the reader options' `extensions` are ignored: `formats`
  reads `quotes.md` with `smart` though the document is markdown without
  it), and the rest of the document's reader options (`reader-options`:
  `--default-image-extension`, `--indented-code-classes`, `--tab-stop`).
- **pandoc.path's functions**: `normalize` drops `.` and doubled slashes
  but keeps `..` (Haskell's `normalise`), which no language's own path
  normalising does (each resolves `..`, or keeps `//`); each port has the
  five it uses (`is_relative`, `filename`, `directory`, `join`,
  `normalize`), 25 lines (`GAPS.md` 41).
- **Lua's `tonumber`** for `shift-heading-level-by`: `abc` is no shift at
  all (not 0: `nil`, so headings stay), `0x1` is 1, `1.5` fails when a
  heading is shifted (pandoc's Header takes an integer). Each port has
  `lua_number`, as amsthm's (`GAPS.md` 42).
- **A directory named as a file**: Lua opens it, and pandoc then fails to
  read nothing (`error-directory`); Python's `open` and Node's
  `readFileSync` fail with "is a directory", Julia reads a directory as
  `""` and Rust's `fs::read` fails differently by system (and under WASI an
  empty name is the working directory): each port tests for a directory
  and fails.
- **Comment lines** start with `//`; empty lines are skipped; a name is
  the whole line, spaces included (`file-a.md ` is another file).

## For upstream: what looks unintended

1. **Absolute URLs in images are mangled**: an image's path is rewritten
   whenever `pandoc.path.is_relative` says so, which it says of
   `https://example.org/i.png`; joined and normalised, it becomes
   `nest/two/https:/example.org/i.png` (case `nested`), and in a file of
   the same directory `https:/example.org/i.png` (the double slash
   normalised away). Links test for `https?://`; images don't.
2. **Other schemes in links**: `mailto:a@example.org` becomes
   `nest/two/mailto:a@example.org` (only `http(s)://` is spared).
3. **An anchor after a directory**: `dir/#x` is left alone (its file name
   starts with `#`), while `page.md#sec` is rewritten: the test is on the
   file name, not on the whole target.
4. **An empty link** (`[empty]()`) becomes a link to the directory
   (`nest/two`).
5. **A negative shift** can make a level-0 heading (`shift`), which
   pandoc's writers print as a paragraph or worse.

## By language

| what | Lua | Python | TypeScript | Rust | Julia |
|---|---|---|---|---|---|
| read a file | `io.open`, `fh:read '*a'` | `open(path, "rb")` | `readFileSync` | `std::fs::read` | `read(path, String)` |
| parse it | `pandoc.read` | `libpandoc.read` | `pandoc` command | `libpandoc::read_many` (wasm) | `LibPandoc.read` |
| the nested walk | `pandoc.walk_block(pandoc.Div(...), {...})` | `contents.walk(f, conversion=ctx.conversion)` | `applyFilter(contents, f)` | `ctx.apply(&mut contents, &mut nested)` | `walk!(t, contents; format = ctx.conversion)` |
| state (auto, last level) | upvalues | a module `State` | module `let`s | a `&mut State` lent to each walk | `const` `Ref`s |

- **Python** and **Julia**: a fragment's walk from a filter function knows
  the conversion only if given it (`conversion=`, `format =`); Lua's
  `PANDOC_READER_OPTIONS` is a global, so a nested include there reads
  with the document's reader options without thinking (`GAPS.md` 43).
- **Rust**: libpandoc-rs is a dependency only when built for wasm
  (`[target.'cfg(target_family = "wasm")'.dependencies]`), so the
  workspace builds natively, and in CI, with no libpandoc; its panir is
  patched to this repository's (`[patch]` in `Cargo.toml`), else the
  documents it returns are another crate's `Pandoc`.

Copies: none; nothing is in two places (`CHECK_UNIQUE=1` is for `run.py
test`, which doesn't run these ports; the in-process hosts don't check).

## In the hosts

Every port gives the original's result in every host, but for what the
host decides (`known.tsv`, `HOSTS.md`): libpandoc's `read` expands tabs
where `pandoc.read` keeps them (`reader-options`: Python, Rust, Julia; the
TypeScript port's `pandoc` command also has no reader options); a wasm
filter sees only the working directory, read-only, so neither `..` nor a
symbolic link out of it (`outside`, in pandocrs, pandocpy and pandocjl),
and in libpandoc.wasm's Node host no directory at all (every case). In the
directory sandbox (`run.py host --sandbox`), every case inside the
directory gives the original's result in every host that has one; what
each kind of filter reads from outside it is `HOSTS.md`'s table.
