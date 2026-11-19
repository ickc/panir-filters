// Run ports inside libpandoc's Node.js host (libpandoc.wasm under node:wasi,
// libpandoc's wasm/node.mjs), for `run.py host`: one instance of the module
// for every case, each case a `convertWithFilters` of its document
// (pandoc's JSON) to its format, with its options (defaults-file keys) and
// environment, and as filters
//
// - TypeScript ports (mode `node`): JS filter functions, panir's
//   `libpandocFilter` of each port's filters, with the case's options as the
//   conversion's; each port's script imported anew for each case (as a new
//   process would run it), its `runFilter` from "panir/node" handing its
//   filter over (handoff.mjs), or, for a port that calls pandoc, its
//   `hosted` export, a function of the instance (which libpandocFilter
//   gives it) returning its filters;
// - Rust ports built for wasm32-wasip1 (mode `node-wasm`): libpandoc's
//   wasm-filter.mjs, given this pandoc for the filter's calls;
//
// then a function that keeps the document as they left it.
//
//   LIBPANDOC_NODE=.../libpandoc/wasm/node.mjs LIBPANDOC_WASM=.../libpandoc.wasm \
//     node host/libpandoc-node.mjs < jobs
//
// Reads a job a line, `{"scripts": [...]` or `"wasm": [...], "format",
// "options", "env", "cwd", "input"}`, and perhaps `"panclude": true`
// (libpandoc's include step before them, its root the working directory);
// writes a line a job, `output<TAB>` the
// document's JSON, or `error<TAB>` the message (its newlines as `\n`).
import { register } from "node:module";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { createInterface } from "node:readline";
import { pathToFileURL } from "node:url";
import { libpandocFilter } from "panir";

register("data:text/javascript," + encodeURIComponent(`
  export async function resolve(specifier, context, next) {
    if (specifier === "panir/node") return { url: ${JSON.stringify(new URL("handoff.mjs", import.meta.url).href)}, shortCircuit: true };
    return next(specifier, context);
  }`));

const nodeHost = process.env.LIBPANDOC_NODE;
const { load } = await import(pathToFileURL(nodeHost).href);
const { wasmFilter } = await import(pathToFileURL(join(dirname(nodeHost), "wasm-filter.mjs")).href);
// the instance sees this repository's files where they are (the cases'
// bibliographies, for a filter's citeproc), its working directory each
// case's
const ROOT = dirname(dirname(new URL(import.meta.url).pathname));
const pandoc = await load(process.env.LIBPANDOC_WASM, { preopens: { [ROOT]: ROOT } });
globalThis.panirFiltersHandoff = [];
let n = 0;

/** The filters a TypeScript port hands over, its script run anew: its
 *  `hosted` (a function of the instance) if it has one. */
async function handedOver(script) {
  globalThis.panirFiltersHandoff = [];
  const port = await import(`${pathToFileURL(script).href}?case=${n++}`);
  if (typeof port.hosted === "function") return port.hosted;
  const handed = globalThis.panirFiltersHandoff;
  if (!handed.length) throw new Error(`${script} hands no filter over`);
  return handed.flatMap((f) => (Array.isArray(f) ? f : [f]));
}

async function run(job) {
  // the case's environment and directory first: a wasm filter is given the
  // directories of $LIBPANDOC_WASM_DIRS as it is when the filter is made,
  // relative to the working directory when it runs
  const saved = { ...process.env };
  Object.assign(process.env, job.env ?? {});
  const here = process.cwd();
  process.chdir(job.cwd); // (the case's files are named relative to it)
  pandoc.chdir(job.cwd);
  let kept;
  try {
    const fns = [];
    for (const script of job.scripts ?? []) fns.push(libpandocFilter(await handedOver(script), job.options));
    for (const path of job.wasm ?? []) fns.push(await wasmFilter(readFileSync(path), { pandoc, name: path }));
    fns.push((doc) => (kept = doc));
    const filters = job.panclude ? { filters: ["panclude", ...fns.map((_, index) => ({ type: "callback", index }))] } : {};
    pandoc.convertWithFilters({ ...job.options, ...filters, from: "json", to: job.format }, JSON.stringify(job.input), fns, { raw: true, bytes: true });
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
