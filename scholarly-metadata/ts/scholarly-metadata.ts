#!/usr/bin/env node
// Normalise the authors and institutes of a document's metadata: a pandoc
// filter.
//
// `author` and `institute` become lists of maps, each with a `name` and an
// `id`; an author's `institute` becomes the numbers of its institutes in
// `institute` (each given an `index`), which collects the top-level ones
// and those given in an author. An author or institute may be a name, a map
// with a `name`, or a map of one entry, its key the name and its value the
// rest (or the name); text with commas is a list of names; an author refers
// to an institute by identifier, by name or by number.
//
// A port of pandoc/lua-filters' scholarly-metadata (Copyright (c) 2017-2021
// Albert Krewinkel, Robert Winkler; ISC; the repository's MIT licence, see
// ../LICENSE).

import { copy, items, MetaInlines, MetaList, MetaString, Str, stringify, truthy, type MetaValue } from "panir";
import { runFilter } from "panir/node";

// A named object, as the original's Lua table: a Map while it is worked on
// (its keys are the document's: any string, `__proto__` too).
type Named = Map<string, MetaValue>;

/** Lua's %s, which its `tonumber` skips too. */
const trim = (s: string) => s.replace(/^[ \t\n\v\f\r]+|[ \t\n\v\f\r]+$/g, "");

/** A number as Lua's `tonumber` reads a string (decimal only). */
function number(text: string): number | undefined {
  return /^[ \t\n\v\f\r]*[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?[ \t\n\v\f\r]*$/.test(text) ? Number(text) : undefined;
}

/** A list as the original takes one: a list's items, text with commas split
 * there (each a string), anything else alone (nothing if missing). */
function asList(value: MetaValue | undefined): MetaValue[] {
  if (value?.t === "MetaInlines") {
    const names = stringify(value).split(",").map(trim);
    if (names.length >= 2) return names.map((name) => MetaString(name));
  }
  return items(value);
}

const nameAndId = (name: MetaValue, id: string): Named => new Map([["name", name], ["id", MetaString(id)]]);

/** An author or institute as a map with a name and an identifier. */
function named(obj: MetaValue): Named {
  switch (obj.t) {
    case "MetaInlines":
      return nameAndId(obj, stringify(obj));
    case "MetaString":
    case "MetaBool": {
      const text = stringify(obj); // (Lua's tostring)
      return nameAndId(MetaInlines([Str(text)]), text);
    }
    case "MetaMap": {
      const entries = Object.entries(obj.content);
      if (obj.content.name !== undefined) {
        const out = new Map(entries);
        const id = out.get("id");
        out.set("id", MetaString(stringify(truthy(id) ? id : out.get("name"))));
        return out;
      }
      if (entries.length === 1) {
        const [[key, value]] = entries;
        let out: Named;
        if (value.t === "MetaString" || value.t === "MetaInlines") {
          out = new Map<string, MetaValue>([["name", value]]);
        } else if (value.t === "MetaMap") {
          out = new Map(Object.entries(value.content));
          if (!truthy(out.get("name"))) out.set("name", MetaInlines([Str(key)]));
        } else {
          throw new Error(`scholarly-metadata: ${key}: neither a name nor a map`);
        }
        const id = out.get("id");
        out.set("id", MetaString(truthy(id) ? stringify(id) : key));
        return out;
      }
    }
  }
  // a map of none or several entries; a list or blocks, which the original
  // names after Lua's tostring of them (NOTES.md)
  throw new Error(`scholarly-metadata: not a named object: ${stringify(obj)}`);
}

const ident = (x: Named) => stringify(x.get("id")); // (a string's text)

/** An author's institutes: by number (a string's), by identifier, or new. */
function resolve(refs: MetaValue[], institutes: Named[]): Named[] {
  return refs.map((ref) => {
    const n = ref.t === "MetaString" ? number(ref.text) : undefined;
    if (n !== undefined && Number.isInteger(n) && n >= 1 && n <= institutes.length) return institutes[n - 1];
    const key = stringify(ref);
    return institutes.find((x) => ident(x) === key) ?? named(ref);
  });
}

/** Add an institute, or what it has the one of its identifier lacks. */
function merge(institutes: Named[], inst: Named): void {
  const same = institutes.find((x) => ident(x) === ident(inst));
  if (same === undefined) {
    institutes.push(inst);
    return;
  }
  // a copy: inst may stay too (a second top-level one, referred to by number)
  for (const [key, value] of inst) if (!truthy(same.get(key))) same.set(key, copy(value));
}

await runFilter({
  Meta: (meta) => {
    const institutes = asList(meta.institute).map(named);
    const authors = asList(meta.author).map(named);
    const theirs = authors.map((author) => resolve(asList(author.get("institute")), institutes));
    for (const inst of theirs.flat()) merge(institutes, inst);
    const numbers = new Map<string, string>(); // each identifier's first institute
    institutes.forEach((inst, i) => {
      inst.set("index", MetaInlines([Str(String(i + 1))]));
      if (!numbers.has(ident(inst))) numbers.set(ident(inst), String(i + 1));
    });
    authors.forEach((author, i) => author.set("institute", MetaList(theirs[i].map((x) => numbers.get(ident(x))!))));
    meta.author = MetaList(authors.map((x) => Object.fromEntries(x)));
    meta.institute = MetaList(institutes.map((x) => Object.fromEntries(x)));
  },
});
