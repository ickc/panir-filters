//! RST-like list tables in Markdown: a pandoc filter.
//!
//! A div of class `list-table` (or `list-table-body`, without a head)
//! holding a bullet list of bullet lists becomes a table: each item a row,
//! each item of an item a cell. A paragraph before the list is the caption.
//! An empty span opening a cell gives the cell its attributes (`colspan`,
//! `rowspan`, `align`), one alone before a row's list gives the row its own;
//! a table in an item starts a new body with its rows. The div's attributes
//! `header-rows`, `header-cols`, `aligns` (`l,c,r,d`) and `widths` shape the
//! table.
//!
//! A port of pandoc-ext/list-table (Copyright (c) 2021 Martin Fischer,
//! 2023-2024 Pandoc Extensions contributors; MIT, see ../LICENSE).

use panir::{
    Alignment, Attr, Block, Caption, Cell, ColSpec, ColWidth, Ctx, Filter, Inline, Row, Table,
    TableBody, TableHead, Text,
};

/// The original also shows the offending block as markdown: that needs
/// pandoc's writer (libpandoc), so here only its kind is named.
type Error = String;

fn alignment(code: &str) -> Option<Alignment> {
    match code {
        "d" => Some(Alignment::AlignDefault),
        "l" => Some(Alignment::AlignLeft),
        "r" => Some(Alignment::AlignRight),
        "c" => Some(Alignment::AlignCenter),
        _ => None,
    }
}

/// A number as Lua's `tonumber` reads one (decimal only).
fn number(text: Option<&str>) -> Option<f64> {
    let text = text?.trim();
    // Rust reads "inf" and "nan" too, Lua doesn't
    if text
        .chars()
        .any(|c| c.is_alphabetic() && c != 'e' && c != 'E')
    {
        return None;
    }
    text.parse().ok()
}

fn has_data_pos(block: &Block) -> bool {
    matches!(block, Block::Div(div) if div.attr.get("data-pos").is_some())
}

/// The block itself, or the first in a div the sourcepos extension wrapped
/// it in.
fn skip_data_pos(block: Block) -> Block {
    match block {
        Block::Div(div) if div.attr.get("data-pos").is_some() => div
            .content
            .into_iter()
            .next()
            .expect("an empty data-pos div"),
        block => block,
    }
}

fn new_cell(item: Vec<Block>) -> Result<Cell, Error> {
    let mut content: Vec<Block> = item.into_iter().map(skip_data_pos).collect();
    let mut attr = Attr::default();
    // pandoc has no attributes on list items: an empty span opening the
    // first block holds the cell's
    if let Some(xs) = content.first_mut().and_then(Block::inlines_mut)
        && let Some(Inline::Span(span)) = xs.first()
        && span.content.is_empty()
    {
        let Inline::Span(span) = xs.remove(0) else {
            unreachable!()
        };
        attr = span.attr;
    }
    let span = |value: Option<Text>| -> Result<i64, Error> {
        value.map_or(Ok(1), |v| {
            v.parse().map_err(|_| format!("not a number of cells: {v}"))
        })
    };
    let col_span = span(attr.remove("colspan"))?;
    let row_span = span(attr.remove("rowspan"))?;
    let alignment = attr
        .remove("align")
        .and_then(|a| alignment(&a))
        .unwrap_or(Alignment::AlignDefault);
    Ok(Cell {
        attr,
        alignment,
        row_span,
        col_span,
        content,
    })
}

/// The attributes of a row: a block holding one empty span.
fn row_attr(mut block: Block) -> Result<Attr, Error> {
    let Some(content) = block.inlines_mut() else {
        return Err("expected list item to have row attrs".into());
    };
    if content.len() != 1 {
        return Err("expected row attrs to contain only one inline".into());
    }
    match content.pop() {
        Some(Inline::Span(span)) if span.content.is_empty() => Ok(span.attr),
        Some(Inline::Span(_)) => Err("expected row attrs span to be empty".into()),
        _ => Err("expected row attrs to contain a span".into()),
    }
}

fn col_specs(attr: &mut Attr, columns: usize) -> Result<Vec<ColSpec>, Error> {
    let mut specs = vec![ColSpec::default(); columns];
    if let Some(aligns) = attr.remove("aligns") {
        let given: Vec<&str> = aligns.split(',').filter(|a| !a.is_empty()).collect();
        if given.len() > columns {
            return Err("more alignments than columns".into());
        }
        for (spec, a) in specs.iter_mut().zip(given) {
            spec.alignment = alignment(a).ok_or_else(|| format!("unknown column alignment {a}"))?;
        }
    }
    if let Some(widths) = attr.remove("widths") {
        let given: Option<Vec<f64>> = widths
            .split(',')
            .filter(|w| !w.is_empty())
            .map(|w| number(Some(w)))
            .collect();
        let given = given
            .filter(|g| g.len() >= columns)
            .ok_or_else(|| format!("invalid widths: {widths}"))?;
        let total: f64 = given.iter().sum();
        for (spec, w) in specs.iter_mut().zip(given) {
            spec.width = ColWidth::ColWidth(w / total);
        }
    }
    Ok(specs)
}

/// The table a list-table div makes, or `None` to leave the div as it is.
fn list_table(block: &mut Block) -> Result<Option<Table>, Error> {
    let Block::Div(div) = block else {
        return Ok(None);
    };
    let kind = match div.attr.classes.first().map(|c| &**c) {
        Some(kind @ ("list-table" | "list-table-body")) => kind.to_string(),
        _ => return Ok(None),
    };
    // an empty div, or a caption alone, stays as it is
    let starts_with_para = |b: &Block| match b {
        Block::Div(d) if has_data_pos(b) => matches!(d.content.first(), Some(Block::Para(_))),
        _ => matches!(b, Block::Para(_)),
    };
    match div.content.as_slice() {
        [] => return Ok(None),
        [only] if starts_with_para(only) => return Ok(None),
        _ => {}
    }

    let mut content = std::mem::take(&mut div.content)
        .into_iter()
        .map(skip_data_pos)
        .peekable();
    let caption = match content.next_if(|b| matches!(b, Block::Para(_))) {
        Some(Block::Para(xs)) => vec![Block::Plain(xs)],
        _ => vec![],
    };
    let items = match content.next() {
        Some(Block::BulletList(items)) => items,
        Some(other) => return Err(format!("expected bullet list, found {}", other.tag())),
        None => unreachable!(),
    };

    let mut bodies = vec![TableBody::default()];
    for mut item in items {
        let mut attr = Attr::default();
        if item.len() > 1 {
            attr = row_attr(skip_data_pos(item.remove(0)))?;
        }
        let [block] =
            <[Block; 1]>::try_from(item).map_err(|_| "expected item to contain only one block")?;
        match skip_data_pos(block) {
            // its rows are a body of their own
            Block::Table(table) => {
                let Table {
                    attr,
                    caption,
                    bodies: mut table_bodies,
                    ..
                } = *table;
                if !caption.content.is_empty() || caption.short.is_some_and(|s| !s.is_empty()) {
                    return Err("table bodies can't have captions (they'd be ignored)".into());
                }
                let [body] = <[TableBody; 1]>::try_from(std::mem::take(&mut table_bodies))
                    .map_err(|_| "table bodies can't contain other table bodies")?;
                if !bodies.last().unwrap().body.is_empty() {
                    bodies.push(TableBody::default());
                }
                let last = bodies.last_mut().unwrap();
                last.attr = attr;
                last.body.extend(body.body);
            }
            Block::BulletList(cells) => {
                let cells = cells.into_iter().map(new_cell).collect::<Result<_, _>>()?;
                bodies.last_mut().unwrap().body.push(Row { attr, cells });
            }
            other => return Err(format!("expected bullet list, found {}", other.tag())),
        }
    }

    let mut attr = std::mem::take(&mut div.attr);
    attr.classes.remove(0);
    let header_rows = number(attr.remove("header-rows").as_deref())
        .unwrap_or(if kind == "list-table" { 1.0 } else { 0.0 });
    let mut header_cols = number(attr.remove("header-cols").as_deref()).unwrap_or(0.0);
    let rows = &mut bodies[0].body;
    let columns = rows
        .first()
        .map_or(0, |row| row.cells.iter().map(|c| c.col_span).sum::<i64>());
    let col_specs = col_specs(&mut attr, columns.max(0) as usize)?;
    let head: Vec<Row> = rows
        .drain(..(header_rows.max(0.0) as usize).min(rows.len()))
        .collect();
    for body in &mut bodies {
        body.row_head_columns = header_cols as i64;
        header_cols = 0.0; // the first body's only
    }
    bodies.retain(|body| !body.body.is_empty());
    Ok(Some(Table {
        attr,
        caption: Caption {
            short: Some(vec![]),
            content: caption,
        },
        col_specs,
        head: TableHead {
            rows: head,
            ..Default::default()
        },
        bodies,
        foot: Default::default(),
    }))
}

struct ListTable;

impl Filter for ListTable {
    fn block(&mut self, x: &mut Block, ctx: &mut Ctx) -> Option<Vec<Block>> {
        match list_table(x) {
            Ok(table) => table.map(|t| vec![t.into()]),
            Err(e) => ctx.fail(e),
        }
    }
}

fn main() {
    panir::filter(|doc, format| panir::apply(doc, &mut ListTable, format));
}
