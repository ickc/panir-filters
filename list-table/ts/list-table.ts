#!/usr/bin/env node
// RST-like list tables in Markdown: a pandoc filter.
//
// A div of class `list-table` (or `list-table-body`, without a head)
// holding a bullet list of bullet lists becomes a table: each item a row,
// each item of an item a cell. A paragraph before the list is the caption.
// An empty span opening a cell gives the cell its attributes (`colspan`,
// `rowspan`, `align`), one alone before a row's list gives the row its own;
// a table in an item starts a new body with its rows. The div's attributes
// `header-rows`, `header-cols`, `aligns` (`l,c,r,d`) and `widths` shape the
// table.
//
// A port of pandoc-ext/list-table (Copyright (c) 2021 Martin Fischer,
// 2023-2024 Pandoc Extensions contributors; MIT, see ../LICENSE).

import { Attr, Caption, Cell, ColSpec, ColWidth, Plain, Row, Table, TableBody, TableFoot, TableHead } from "panir";
import type { Alignment, Block, Div } from "panir";
import { runFilter } from "panir/node";

const ALIGNMENTS: Record<string, Alignment> = { d: "AlignDefault", l: "AlignLeft", r: "AlignRight", c: "AlignCenter" };

/** The original also shows the offending block as markdown: that needs
 * pandoc's writer (libpandoc), so here only its kind is named. */
class ListTableError extends Error {}

/** A number as Lua's `tonumber` reads one (decimal only), else undefined. */
function number(text: string | undefined): number | undefined {
  return text !== undefined && /^\s*[-+]?(\d+\.?\d*|\.\d+)([eE][-+]?\d+)?\s*$/.test(text) ? Number(text) : undefined;
}

/** The block itself, or the first in a div the sourcepos extension wrapped it in. */
function skipDataPos(block: Block): Block {
  if (block.t === "Div" && block.attr.attributes.some(([k]) => k === "data-pos")) return block.content[0];
  return block;
}

/** Remove an attribute, returning its value. */
function pop(attr: Attr, key: string): string | undefined {
  const i = attr.attributes.findIndex(([k]) => k === key);
  return i < 0 ? undefined : attr.attributes.splice(i, 1)[0][1];
}

function newCell(item: Block[]): Cell {
  const blocks = item.map(skipDataPos);
  let attr = Attr();
  // pandoc has no attributes on list items: an empty span opening the
  // first block holds the cell's
  const first = blocks[0];
  if (first && "content" in first) {
    const opening = first.content[0];
    if (opening && !Array.isArray(opening) && opening.t === "Span" && opening.content.length === 0) {
      first.content.shift();
      attr = opening.attr;
    }
  }
  const colSpan = Number(pop(attr, "colspan") || 1);
  const rowSpan = Number(pop(attr, "rowspan") || 1);
  const alignment = ALIGNMENTS[pop(attr, "align") ?? ""] ?? "AlignDefault";
  return Cell(blocks, { alignment, rowSpan, colSpan, attr });
}

/** The attributes of a row: a block holding one empty span. */
function rowAttr(block: Block): Attr {
  if (!("content" in block)) throw new ListTableError("expected list item to have row attrs");
  const content = block.content;
  if (content.length !== 1) throw new ListTableError("expected row attrs to contain only one inline");
  const span = content[0];
  if (Array.isArray(span) || span.t !== "Span") throw new ListTableError("expected row attrs to contain a span");
  if (span.content.length > 0) throw new ListTableError("expected row attrs span to be empty");
  return span.attr;
}

function colSpecs(attr: Attr, columns: number): ColSpec[] {
  const specs = Array.from({ length: columns }, () => ColSpec());
  const aligns = pop(attr, "aligns");
  if (aligns !== undefined) {
    const given = aligns.split(",").filter((a) => a !== "");
    const unknown = given.find((a) => !(a in ALIGNMENTS));
    if (unknown !== undefined) throw new ListTableError(`unknown column alignment ${unknown}`);
    if (given.length > columns) throw new ListTableError("more alignments than columns");
    given.forEach((a, i) => { specs[i].alignment = ALIGNMENTS[a]; });
  }
  const widths = pop(attr, "widths");
  if (widths !== undefined) {
    const given = widths.split(",").filter((w) => w !== "").map(number);
    if (given.some((w) => w === undefined) || given.length < columns) {
      throw new ListTableError(`invalid widths: ${widths}`);
    }
    const total = given.reduce((a: number, b) => a + b!, 0);
    specs.forEach((spec, i) => { spec.width = ColWidth(given[i]! / total); });
  }
  return specs;
}

function listTable(div: Div): Table | undefined {
  const kind = div.attr.classes[0];
  if (kind !== "list-table" && kind !== "list-table-body") return;
  const content = div.content.map(skipDataPos);
  let caption: Block[] = [];
  const first = content[0];
  if (first?.t === "Para") {
    content.shift();
    caption = [Plain(first.content)];
  }
  // an empty div, or a caption alone, stays as it is
  if (content.length === 0) return;
  const list = content[0];
  if (list.t !== "BulletList") throw new ListTableError(`expected bullet list, found ${list.t}`);

  const bodies = [TableBody()];
  for (let item of list.content) {
    let attr = Attr();
    if (item.length > 1) {
      attr = rowAttr(skipDataPos(item[0]));
      item = item.slice(1);
    }
    if (item.length !== 1) throw new ListTableError("expected item to contain only one block");
    const block = skipDataPos(item[0]);
    if (block.t === "Table") {
      // its rows are a body of their own
      if (block.caption.content.length > 0 || (block.caption.short?.length ?? 0) > 0) {
        throw new ListTableError("table bodies can't have captions (they'd be ignored)");
      }
      if (block.bodies.length !== 1) throw new ListTableError("table bodies can't contain other table bodies");
      if (bodies.at(-1)!.body.length > 0) bodies.push(TableBody());
      bodies.at(-1)!.attr = block.attr;
      bodies.at(-1)!.body.push(...block.bodies[0].body);
    } else if (block.t === "BulletList") {
      bodies.at(-1)!.body.push(Row(block.content.map(newCell), { attr }));
    } else {
      throw new ListTableError(`expected bullet list, found ${block.t}`);
    }
  }

  div.attr.classes.shift();
  const rows = bodies[0].body;
  const headerRows = number(pop(div.attr, "header-rows")) ?? (kind === "list-table" ? 1 : 0);
  let headerCols = number(pop(div.attr, "header-cols")) ?? 0;
  const columns = rows.length > 0 ? rows[0].cells.reduce((n, cell) => n + cell.colSpan, 0) : 0;
  const specs = colSpecs(div.attr, columns);
  const head = rows.splice(0, Math.max(0, Math.trunc(headerRows)));
  for (const body of bodies) {
    body.rowHeadColumns = Math.trunc(headerCols);
    headerCols = 0; // the first body's only
  }
  return Table(
    Caption(caption, { short: [] }),
    specs,
    TableHead(head),
    bodies.filter((body) => body.body.length > 0),
    TableFoot(),
    { attr: div.attr },
  );
}

await runFilter({ Div: listTable });
