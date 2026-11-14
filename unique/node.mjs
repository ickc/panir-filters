// panir/node's runFilter, then checkUnique on what the filter made.
import { applyFilter, checkUnique, Conversion, parse, serialize } from "panir";

export async function runFilter(filter) {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const doc = parse(Buffer.concat(chunks).toString("utf8"));
  const conversion = Conversion.fromEnvironment(process.argv.slice(2), process.env);
  const out = applyFilter(doc, Array.isArray(filter) ? filter : [filter], conversion);
  checkUnique(out);
  await new Promise((resolve, reject) => process.stdout.write(serialize(out), (e) => (e ? reject(e) : resolve())));
}
