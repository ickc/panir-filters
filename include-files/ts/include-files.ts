#!/usr/bin/env node
// Code blocks of class `include` replaced by the files they name: a pandoc
// filter.
//
// Each line of the block names a file (`//` starts a comment); each is read
// by pandoc, as markdown or as the block's `format`, and its blocks (not its
// metadata) take the block's place. Headings shift by
// `shift-heading-level-by`, or, with `include-auto` in the metadata, by the
// level of the last heading before the block. Included files may include
// others, named relative to their own directory; relative links, images and
// `include` attributes (include-code-files') in an included file are
// rewritten relative to the including one.
//
// Reading the files is Node's; parsing them is pandoc's (`pandoc.read` in
// the original): in libpandoc's JavaScript host, the instance the filter runs
// in (`hosted`, for panir's `libpandocFilter`), which reads a file as the
// conversion reads its input (`pandoc.read(text, ctx.conversion)`: its reader
// options, its tabs kept, in pandoc's sandbox when the conversion is
// sandboxed); run as a JSON filter, the `pandoc` command, as pandocfilters
// and panflute filters do: a process for each file, which reads it as the
// command reads its input, without the document's reader options.
//
// A port of pandoc-ext/include-files (© 2019–2021 Albert Krewinkel; MIT,
// see ../LICENSE).

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { pathToFileURL } from "node:url";
import {
  applyFilter, fromJSON, getAttribute, luaNumber, parse, setAttribute, truthy,
  type Block, type Conversion, type Filter, type Header, type Pandoc,
} from "panir";
import { runFilter } from "panir/node";

/** What parses a file's text in a format, in the conversion the filter runs in. */
type Reader = (text: string, format: string, conversion: Conversion) => Pandoc;

/** libpandoc's JavaScript host, as much as this filter uses of it. */
interface Host {
  read(text: string, from: unknown, options?: Record<string, unknown>): unknown;
}

/** The `pandoc` command: a process a file, which reads it as the command
 *  reads its input. */
const byCommand: Reader = (text, format) =>
  parse(execFileSync("pandoc", ["-f", format, "-t", "json"], { input: text }).toString("utf8"));

/** The instance of libpandoc's JavaScript host the filter runs in: as the
 *  conversion reads its input, in another format. */
const byHost = (pandoc: Host): Reader => (text, format, conversion) =>
  fromJSON(pandoc.read(text, conversion, { from: format })) as Pandoc;

// pandoc.path's functions, as pandoc has them (Haskell's System.FilePath,
// POSIX): `normalize` drops `.` and doubled slashes but keeps `..`, unlike
// node:path's.
const isRelative = (path: string) => !path.startsWith("/");
const filename = (path: string) => path.slice(path.lastIndexOf("/") + 1);

function directory(path: string): string {
  const slash = path.lastIndexOf("/");
  if (slash < 0) return path ? "." : "";
  const stripped = path.slice(0, slash + 1).replace(/\/+$/, "");
  return stripped || "/";
}

function join(a: string, b: string): string {
  if (b.startsWith("/") || !a) return b;
  if (!b) return a;
  return a.endsWith("/") ? a + b : `${a}/${b}`;
}

function normalize(path: string): string {
  const rest = path.replace(/^\/+/, "");
  const parts = rest.split("/").filter((p) => p && p !== ".");
  const out = (path.startsWith("/") ? "/" : "") + parts.join("/");
  if (!out) return ".";
  const isDir = rest.endsWith("/") || (rest.endsWith(".") && rest.slice(0, -1).endsWith("/"));
  return isDir && !out.endsWith("/") ? out + "/" : out;
}

let auto = false; // (include-auto)
let lastLevel = 0; // the level of the last heading met

const updateLastLevel = (header: Header) => {
  lastLevel = header.level;
};

/** Headings shifted; relative paths taken to the including file's directory. */
function updateContents(blocks: Block[], shiftBy: number | undefined, includePath: string): Block[] {
  return applyFilter(blocks, {
    Header: (header) => {
      if (shiftBy === undefined) return;
      const level = header.level + shiftBy;
      // (Lua's error, as pandoc's Header takes an integer)
      if (!Number.isInteger(level)) throw new Error(`shift-heading-level-by: ${shiftBy}: number has no integer representation`);
      header.level = level;
    },
    Link: (link) => {
      const url = link.target.url;
      if (isRelative(url) && !filename(url).startsWith("#") && !/https?:\/\//.test(url)) {
        link.target.url = normalize(join(includePath, url));
      }
    },
    Image: (image) => {
      if (isRelative(image.target.url)) image.target.url = normalize(join(includePath, image.target.url));
    },
    CodeBlock: (cb) => {
      const path = getAttribute(cb.attr, "include");
      if (path !== undefined && isRelative(path)) setAttribute(cb.attr, "include", normalize(join(includePath, path)));
    },
  });
}

/** The filter including files, for blocks of a file in `base` (the original
 *  changes the working directory to it: here the names are joined to it, as
 *  the working directory is the process's), parsed by `read`. */
function transclusion(base: string, read: Reader): Filter {
  return {
    Header: updateLastLevel,
    CodeBlock: (cb, ctx) => {
      if (!cb.attr.classes.includes("include")) return;
      const format = getAttribute(cb.attr, "format") || "markdown";
      const shiftInput = getAttribute(cb.attr, "shift-heading-level-by");
      const shiftBy = shiftInput !== undefined ? luaNumber(shiftInput) : auto ? lastLevel : 0;
      const before = lastLevel;
      const blocks: Block[] = [];
      for (const line of cb.text.split("\n")) {
        if (!line || line.startsWith("//")) continue;
        let text: Buffer;
        try {
          text = readFileSync(join(base, line));
        } catch (e) {
          // (Lua opens a directory, and pandoc fails to read nothing)
          if ((e as NodeJS.ErrnoException).code === "EISDIR") throw e;
          process.stderr.write(`Cannot open file ${line} | Skipping includes\n`);
          continue;
        }
        const doc = read(text.toString("utf8"), format, ctx.conversion);
        lastLevel = 0;
        // (the walk of the included blocks knows this one's conversion)
        const contents = applyFilter(doc.blocks, transclusion(join(base, directory(line)), read));
        lastLevel = before;
        blocks.push(...updateContents(contents, shiftBy, directory(line)));
      }
      return blocks;
    },
  };
}

/** The filters, parsing with `read`. */
const filters = (read: Reader): Filter[] => [
  {
    // (for each document: a host may run this filter on many, where Lua
    // loads the original anew for each)
    Meta: (meta) => {
      auto = truthy(meta["include-auto"]);
      lastLevel = 0;
    },
  },
  transclusion("", read),
];

/** For libpandoc's JavaScript host, given the instance the filter runs in
 *  (`libpandocFilter(hosted)`): reading with it. */
export const hosted = (pandoc: Host): Filter[] => filters(byHost(pandoc));

// run as a JSON filter (the main module): the pandoc command reads
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await runFilter(filters(byCommand));
