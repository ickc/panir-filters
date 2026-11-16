// Run ports inside libpandoc's Node.js host (libpandoc.wasm under node:wasi,
// libpandoc's wasm/node.mjs), for `run.py host`: one instance of the module
// for every case, each case a `convertWithFilters` of its document
// (pandoc's JSON) to its format, with its options (defaults-file keys) and
// environment, and as filters
//
// - TypeScript ports (mode `node`): JS filter functions; each port's script
//   imported anew for each case (as a new process would run it), its
//   `runFilter` from "panir/node" handing its filter over (handoff.mjs); a
//   function on pandoc's JSON text and the callback's context, which runs
//   the panir filter with a Conversion made of that context (glue every user
//   writes: the host has none);
// - Rust ports built for wasm32-wasip1 (mode `node-wasm`): libpandoc's
//   wasm-filter.mjs, given this pandoc for the filter's calls;
//
// then a function that keeps the document as they left it.
//
//   LIBPANDOC_NODE=.../libpandoc/wasm/node.mjs LIBPANDOC_WASM=.../libpandoc.wasm \
//     node host/libpandoc-node.mjs < jobs
//
// Reads a job a line, `{"scripts": [...]` or `"wasm": [...], "format",
// "options", "env", "cwd", "input"}`; writes a line a job, `output<TAB>` the
// document's JSON, or `error<TAB>` the message (its newlines as `\n`).
import { register } from "node:module";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { createInterface } from "node:readline";
import { pathToFileURL } from "node:url";
import { applyFilter, Conversion, parse, serialize } from "panir";

register("data:text/javascript," + encodeURIComponent(`
  export async function resolve(specifier, context, next) {
    if (specifier === "panir/node") return { url: ${JSON.stringify(new URL("handoff.mjs", import.meta.url).href)}, shortCircuit: true };
    return next(specifier, context);
  }`));

const nodeHost = process.env.LIBPANDOC_NODE;
const { load } = await import(pathToFileURL(nodeHost).href);
const { wasmFilter } = await import(pathToFileURL(join(dirname(nodeHost), "wasm-filter.mjs")).href);
const pandoc = await load(process.env.LIBPANDOC_WASM);
globalThis.panirFiltersHandoff = [];
let n = 0;

/** The filters a TypeScript port hands over, its script run anew. */
async function handedOver(script) {
  globalThis.panirFiltersHandoff = [];
  await import(`${pathToFileURL(script).href}?case=${n++}`);
  const handed = globalThis.panirFiltersHandoff;
  if (!handed.length) throw new Error(`${script} hands no filter over`);
  return handed.flatMap((f) => (Array.isArray(f) ? f : [f]));
}

/** A JS filter function running panir filters, in the conversion the
 *  callback's context describes. */
function panirFilter(filters) {
  return (doc, context) => {
    const ctx = JSON.parse(context);
    const conversion = new Conversion({
      format: ctx.format,
      inputFormat: ctx["input-format"],
      outputFormat: ctx["output-format"],
      readerOptions: ctx["reader-options"],
      writerOptions: ctx["writer-options"],
    });
    return serialize(applyFilter(parse(doc), filters, conversion));
  };
}

async function run(job) {
  const fns = [];
  for (const script of job.scripts ?? []) fns.push(panirFilter(await handedOver(script)));
  for (const path of job.wasm ?? []) fns.push(await wasmFilter(readFileSync(path), { pandoc, name: path }));
  let kept;
  fns.push((doc) => (kept = doc));
  const saved = { ...process.env };
  Object.assign(process.env, job.env ?? {});
  const here = process.cwd();
  process.chdir(job.cwd); // (the case's files are named relative to it)
  try {
    pandoc.convertWithFilters({ ...job.options, from: "json", to: job.format }, JSON.stringify(job.input), fns, { raw: true, bytes: true });
  } finally {
    process.chdir(here);
    for (const k of Object.keys(job.env ?? {})) if (!(k in saved)) delete process.env[k];
    Object.assign(process.env, saved);
  }
  return kept;
}

for await (const line of createInterface({ input: process.stdin })) {
  let result;
  try {
    result = "output\t" + (await run(JSON.parse(line)));
  } catch (e) {
    result = "error\t" + String(e?.message ?? e).replaceAll("\n", "\\n");
  }
  process.stdout.write(result + "\n");
}
