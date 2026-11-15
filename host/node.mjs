// panir/node's runFilter, with the conversion's options (defaults-file keys)
// from $PANIR_FILTERS_OPTIONS, a case's writer options, which a JSON filter
// is not given; with CHECK_UNIQUE=1, checkUnique on what the filter made.
import { applyFilter, checkUnique, Conversion, parse, serialize } from "panir";

export async function runFilter(filter) {
  const chunks = [];
  for await (const chunk of process.stdin) chunks.push(chunk);
  const doc = parse(Buffer.concat(chunks).toString("utf8"));
  const conversion = Conversion.fromEnvironment(process.argv.slice(2), process.env);
  const options = process.env.PANIR_FILTERS_OPTIONS;
  const run = options ? new Conversion({ ...conversion, options: JSON.parse(options) }) : conversion;
  const out = applyFilter(doc, Array.isArray(filter) ? filter : [filter], run);
  if (process.env.CHECK_UNIQUE === "1") checkUnique(out);
  await new Promise((resolve, reject) => process.stdout.write(serialize(out), (e) => (e ? reject(e) : resolve())));
}
