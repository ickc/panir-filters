// "panir/node" for a TypeScript port run by libpandoc-node.mjs: its
// `runFilter` hands the filter over instead of reading stdin.
export async function runFilter(filter) {
  globalThis.panirFiltersHandoff.push(filter);
}
