// Run a TypeScript port, then panir's checkUnique on what it made (run.py
// with CHECK_UNIQUE=1): `node --import ./unique/check.mjs PORT.ts FORMAT`.
// The port's "panir/node" is this folder's node.mjs instead.
import { register } from "node:module";

register("data:text/javascript," + encodeURIComponent(`
  export async function resolve(specifier, context, next) {
    if (specifier === "panir/node") return { url: ${JSON.stringify(new URL("node.mjs", import.meta.url).href)}, shortCircuit: true };
    return next(specifier, context);
  }`));
