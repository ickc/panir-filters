// Run a TypeScript port in process, as a host running filter scripts does,
// for run.py: `node --import ./host/register.mjs PORT.ts FORMAT`. The port's
// "panir/node" is this folder's node.mjs instead.
import { register } from "node:module";

register("data:text/javascript," + encodeURIComponent(`
  export async function resolve(specifier, context, next) {
    if (specifier === "panir/node") return { url: ${JSON.stringify(new URL("node.mjs", import.meta.url).href)}, shortCircuit: true };
    return next(specifier, context);
  }`));
