#!/usr/bin/env node
// Several bibliographies, one a topic: a pandoc filter.
//
// The metadata's `bibliography` maps topics to files; a div `#refs-TOPIC`
// (or `refs_TOPIC`, `refsTOPIC`) gets the bibliography of that topic's file,
// of every work the document cites. The document's citations are resolved
// with all the files together.
//
// citeproc is pandoc's (`pandoc.utils.citeproc` in the original). No
// JavaScript host gives a filter the pandoc it runs in, so this one runs the
// `pandoc` command with `--citeproc` on the document's JSON, as the
// original does where pandoc's Lua has no citeproc (`run_json_filter`).
//
// A port of pandoc-ext/multibib (Copyright © 2018-2024 Albert Krewinkel;
// MIT, see ../LICENSE).

import { execFileSync } from "node:child_process";
import { copy, Div, MetaList, MetaString, Pandoc, Para, parse, serialize, stringify, type Cite, type Meta } from "panir";
import { runFilter } from "panir/node";

// What citeproc reads of the metadata, copied for a topic's bibliography.
const FIELDS = ["bibliography", "references", "csl", "citation-style", "link-citations", "citation-abbreviations",
  "lang", "suppress-bibliography", "reference-section-title", "notes-after-punctuation", "nocite"];

/** pandoc's citeproc on a document, its warnings silenced. */
const citeproc = (doc: Pandoc): Pandoc =>
  parse(execFileSync("pandoc", ["--from=json", "--to=json", "--citeproc", "--quiet"], { input: serialize(doc) }).toString("utf8"));

let cites: Cite[] = []; // every citation of the document
let docMeta: Meta = {}; // the document's metadata, as the first pass saw it
let refs: Div | undefined; // the bibliography's div, as citeproc made it

await runFilter([
  // a new document (a host may run these filters on many: Lua loads the
  // original anew for each)
  { Pandoc: () => void (cites = []) },
  {
    Cite: (cite) => void cites.push(copy(cite)),
    Meta: (meta) => void (docMeta = copy(meta)),
  },
  {
    // the citations resolved with every bibliography at once
    Pandoc: (doc) => {
      const given = doc.meta.bibliography;
      // (Lua's pandoc.utils.type: a map is a "table"; its values in any order)
      const files = given?.t === "MetaMap" ? Object.values(given.content).map((v) => MetaString(stringify(v))) : [];
      doc.meta.bibliography = MetaList(files);
      doc.blocks.push(Div([], { identifier: "refs" }));
      const out = citeproc(doc);
      refs = out.blocks.pop() as Div;
      // (the original restores `orig_bib`, which it never set: the field goes)
      delete out.meta.bibliography;
      return out;
    },
  },
  {
    Div: (div, ctx) => {
      const m = /^refs[-_]?([-_A-Za-z0-9]*)$/.exec(div.attr.identifier);
      const bibliography = docMeta.bibliography;
      // (a map's entry; Lua indexes a list or text by a name to nothing)
      const bibfile = m && bibliography?.t === "MetaMap" ? bibliography.content[m[1]] : undefined;
      if (bibfile === undefined) return;
      const meta: Meta = Object.fromEntries(FIELDS.filter((k) => docMeta[k] !== undefined).map((k) => [k, copy(docMeta[k])]));
      meta.bibliography = copy(bibfile);
      const res = citeproc(Pandoc([Para(copy(cites)), Div([], { identifier: "refs" })], { meta }));
      const filled = res.blocks[1] as Div;
      div.content = filled.content;
      div.attr.classes = [...new Set(refs!.attr.classes)];
      div.attr.attributes = copy(refs!.attr.attributes);
      if (ctx.format === "odt" || ctx.format === "docx") {
        // pandoc gives Word's bibliography this style
        const i = div.attr.attributes.findIndex(([k]) => k === "custom-style");
        if (i < 0) div.attr.attributes.push(["custom-style", "Bibliography"]);
        else if (!div.attr.attributes[i][1]) div.attr.attributes[i][1] = "Bibliography";
      }
      return div;
    },
  },
]);
