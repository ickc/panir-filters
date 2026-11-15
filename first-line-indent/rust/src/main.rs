//! First-line indents, removed after quotes, lists and the like: a pandoc
//! filter.
//!
//! A paragraph right after a block quote, a list, a code block or a rule
//! (`first-line-indent.remove-after`, `dont-remove-after`) gets `\noindent`
//! in LaTeX, or a marker div before it in HTML, unless it starts with
//! `\indent` or `\noindent` itself. The metadata's `indent` is set true, and
//! `header-includes` gets the CSS (HTML) or the indent's `size` (LaTeX).
//!
//! A port of pandoc/lua-filters' first-line-indent (Copyright (c) 2021
//! Julien Dutant; MIT, see ../LICENSE).

use panir::{
    Block, Ctx, Filter, Inline, Meta, MetaValue, Pandoc, blocks_of, given, items, lookup, stringify,
};

const CSS: &str = "  <style>
    p {
      text-indent: SIZE;
      margin: 0;
    }
    header + p {
      text-indent: ;
    }
    :is(h1, h2, h3, h4, h5, h6) + p {
      text-indent: 0;
    }
    :is(div.no-first-line-indent-after) + p {
      text-indent: 0;
    }
    :is(div.first-line-indent-after) + p {
      text-indent: SIZE;
    }
  </style>
";

struct Options {
    set_metadata_variable: bool,
    set_header_includes: bool,
    auto_remove: bool,
    remove_after: Vec<String>,
    size: String,
}

impl Default for Options {
    fn default() -> Self {
        let after = [
            "BlockQuote",
            "BulletList",
            "CodeBlock",
            "DefinitionList",
            "HorizontalRule",
            "OrderedList",
        ];
        Options {
            set_metadata_variable: true,
            set_header_includes: true,
            auto_remove: true,
            remove_after: after.map(String::from).to_vec(),
            size: "1em".into(),
        }
    }
}

#[derive(Clone, Copy)]
enum Kind {
    Indent,
    NoIndent,
}

/// `\indent ` or `\noindent `, as the markdown reader makes them.
fn command(kind: Kind) -> Inline {
    let text = match kind {
        Kind::Indent => "\\indent ",
        Kind::NoIndent => "\\noindent ",
    };
    Inline::raw("tex", text)
}

/// Block types: text is one, a list's items each one; else none.
fn names(value: Option<&MetaValue>) -> Option<Vec<String>> {
    match value? {
        v @ MetaValue::MetaInlines(_) => Some(vec![stringify(v)]),
        MetaValue::MetaList(xs) => Some(xs.iter().map(stringify).collect()),
        _ => None,
    }
}

/// `text` as Lua's `gsub` reads a replacement, for a pattern without
/// captures: `%0` and `%1` are the match, `%%` a percent sign, any other `%`
/// an error. (So a size of `5%` fails, as in the original.)
fn lua_replacement(text: &str, matched: &str) -> Result<String, &'static str> {
    let mut out = String::new();
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c != '%' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('0' | '1') => out.push_str(matched),
            Some('%') => out.push('%'),
            _ => return Err("first-line-indent: invalid use of '%' in replacement string"),
        }
    }
    Ok(out)
}

/// The first filter: the options, and the metadata.
#[derive(Default)]
struct Configure(Options);

impl Configure {
    fn configure(&mut self, meta: &mut Meta, format: &str) -> Result<(), String> {
        let options = &mut self.0;
        let user = |key: &str| lookup(meta, &["first-line-indent", key]);
        let is_false = |v: Option<&MetaValue>| matches!(v, Some(MetaValue::MetaBool(false)));
        if matches!(
            meta.get("first-line-indent"),
            Some(MetaValue::MetaBool(true))
        ) {
            return Err("first-line-indent: a configuration of true".into()); // (Lua indexes a boolean)
        }
        if is_false(user("set-metadata-variable")) {
            options.set_metadata_variable = false;
        }
        if is_false(user("set-header-includes")) {
            options.set_header_includes = false;
        }
        if is_false(user("auto-remove")) {
            options.auto_remove = false;
        }
        options
            .remove_after
            .extend(names(user("remove-after")).unwrap_or_default());
        if let Some(blacklist) = names(user("dont-remove-after")) {
            options.remove_after.retain(|x| !blacklist.contains(x));
        }
        if let Some(size) = user("size") {
            options.size = stringify(size);
        }

        // The original changes the metadata, then returns it only if it set
        // `indent`: otherwise (Lua's `return nil`) nothing it changed counts,
        // but what fails as it changes it (header-includes) fails all the same.
        let changes = options.set_metadata_variable && !is_false(meta.get("indent"));
        // (Lua's pattern `html*`: `htm`, then any `l`s)
        let header = if !options.set_header_includes {
            None
        } else if format.contains("htm") {
            let size = lua_replacement(&options.size, "SIZE")?;
            Some(Block::raw("html", CSS.replace("SIZE", &size)))
        } else if format.contains("latex") && options.size != "1em" {
            let size = &options.size;
            Some(Block::raw(
                "tex",
                format!("\\setlength{{\\parindent}}{{{size}}}"),
            ))
        } else {
            None
        };
        // before what is there: a list's items, or one value
        let mut includes = None;
        if let Some(header) = header {
            let existing = items(given(meta.get("header-includes"))).to_vec();
            let mut blocks = vec![header];
            blocks.extend(blocks_of(&existing.into())?);
            includes = Some(blocks);
        }
        if changes {
            meta.insert("indent".into(), true.into());
            if let Some(blocks) = includes {
                meta.insert("header-includes".into(), blocks.into());
            }
        }
        Ok(())
    }
}

impl Filter for Configure {
    fn meta(&mut self, meta: &mut Meta, ctx: &mut Ctx) -> Option<()> {
        ctx.ok(self.configure(meta, ctx.format().unwrap_or_default()))
    }
}

/// The second filter: the body's paragraphs.
struct Body(Options);

/// The paragraph with its first line indented or not.
fn markup(kind: Kind, mut content: Vec<Inline>, format: &str) -> Vec<Block> {
    if format.contains("latex") {
        if matches!(content.first(), Some(x) if *x == command(Kind::Indent) || *x == command(Kind::NoIndent))
        {
            content[0] = command(kind);
        } else {
            content.insert(0, command(kind));
        }
        return vec![Block::Para(content)];
    }
    if format.contains("htm") {
        let div = match kind {
            Kind::Indent => "<div class=\"first-line-indent-after\"></div>",
            Kind::NoIndent => "<div class=\"no-first-line-indent-after\"></div>",
        };
        return vec![Block::raw("html", div), Block::Para(content)];
    }
    vec![Block::Para(content)]
}

impl Filter for Body {
    fn pandoc(&mut self, doc: &mut Pandoc, ctx: &mut Ctx) -> Option<()> {
        let format = ctx.format().unwrap_or_default();
        let options = &self.0;
        let mut blocks = vec![];
        let mut after = false; // whether the block before removes an indent
        for block in std::mem::take(&mut doc.blocks) {
            let Block::Para(content) = block else {
                after = options.remove_after.iter().any(|x| x == block.tag());
                blocks.push(block);
                continue;
            };
            let explicit = [Kind::Indent, Kind::NoIndent]
                .into_iter()
                .find(|&kind| content.first() == Some(&command(kind)));
            match explicit {
                Some(kind) => blocks.extend(markup(kind, content, format)),
                None if options.auto_remove && after => {
                    blocks.extend(markup(Kind::NoIndent, content, format))
                }
                None => blocks.push(Block::Para(content)),
            }
            after = false;
        }
        doc.blocks = blocks;
        Some(())
    }
}

fn main() {
    panir::filter(|doc, format| {
        let mut configure = Configure::default();
        panir::apply(doc, &mut configure, format)?; // the options and the metadata
        panir::apply(doc, &mut Body(configure.0), format) // then the body
    });
}
