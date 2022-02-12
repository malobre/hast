use pretty::{Arena, DocAllocator, DocBuilder};

use crate::{
    ast::{comment::Comment, doctype::Doctype, element::Element, Node},
    Configuration,
};

/// Prettify the given input according to the configuration.
///
/// # Errors
/// Will return an error if parsing / printing fails.
pub fn format(input: &str, config: &Configuration) -> anyhow::Result<String> {
    let (_, nodes) =
        Node::parse_many(input).map_err(nom::Err::<nom::error::Error<&str>>::to_owned)?;

    let alloc = Arena::<()>::new();

    let mut buffer = String::new();

    nodes
        .iter()
        .map(|node| pretty_node(node, &alloc, config).append(alloc.line_()))
        .reduce(DocBuilder::append)
        .unwrap_or_else(|| alloc.nil())
        .render_fmt(usize::try_from(config.line_width)?, &mut buffer)?;

    Ok(buffer)
}

fn pretty_node<'b, D, A>(
    node: &'b Node,
    alloc: &'b D,
    config: &Configuration,
) -> DocBuilder<'b, D, A>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    match node {
        Node::Comment(comment) => pretty_comment(comment, alloc, config),
        Node::Doctype(doctype) => pretty_doctype(doctype, alloc),
        Node::Element(element) => pretty_element(element, alloc, config),
        Node::Text(text) => pretty_text(text, alloc),
    }
}

fn pretty_comment<'b, D, A>(
    Comment(comment): &'b Comment,
    alloc: &'b D,
    config: &Configuration,
) -> DocBuilder<'b, D, A>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    if comment.is_empty() {
        alloc.text("<!---->")
    } else {
        let mut buffer = alloc.nil();

        for line in comment.lines() {
            buffer += alloc.line();
            buffer += alloc.text(line);
        }

        // Only indent single line comments
        if comment.lines().nth(1).is_none() {
            buffer = buffer.nest(isize::from(config.indent_width));
        }

        if comment.lines().nth(1).is_some() {
            buffer += alloc.hardline();
        } else {
            buffer += alloc.line();
        }

        alloc.text("<!--").append(buffer).append("-->").group()
    }
}

fn pretty_doctype<'b, D, A>(doctype: &'b Doctype, alloc: &'b D) -> DocBuilder<'b, D, A>
where
    D: DocAllocator<'b, A>,
{
    alloc.text(if doctype.legacy {
        r#"<!DOCTYPE html SYSTEM "about:legacy-compat">"#
    } else {
        "<!DOCTYPE html>"
    })
}

fn pretty_element<'b, D, A>(
    start: &'b Element,
    alloc: &'b D,
    config: &Configuration,
) -> DocBuilder<'b, D, A>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    let (name, attributes, inner_nodes_if_not_void) = match start {
        Element::Void {
            name, attributes, ..
        } => (*name, attributes, None),
        Element::Normal {
            name,
            attributes,
            content,
            ..
        } => (*name, attributes, Some(content)),
    };

    let mut buffer = alloc.text("<").append(name);

    if let Some(attributes) = attributes
        .iter()
        .map(|(name, value)| {
            alloc.line().append(*name).append(
                value.map(|value| alloc.text("=").append(alloc.text(value).double_quotes())),
            )
        })
        .reduce(DocBuilder::append)
    {
        buffer += attributes
            .nest(isize::from(config.indent_width))
            .append(alloc.line_())
            .group();
    }

    if let Some(nodes) = inner_nodes_if_not_void {
        let force_multiline =
            !nodes.is_empty() && nodes.iter().all(|node| !matches!(node, Node::Text(_)));

        buffer += alloc.text(">");

        if let Some(nodes) = nodes
            .iter()
            .map(|node| {
                if force_multiline {
                    alloc.hardline()
                } else {
                    alloc.line_()
                }
                .append(pretty_node(node, alloc, config))
            })
            .reduce(DocBuilder::append)
        {
            buffer += nodes
                .nest(isize::from(config.indent_width))
                .append(alloc.line_())
                .group();
        }

        buffer += alloc.text("</").append(name).append(">");

        buffer.group()
    } else {
        buffer.append(alloc.text("/>"))
    }
}

fn pretty_text<'b, D, A>(text: &'b str, alloc: &'b D) -> DocBuilder<'b, D, A>
where
    D: DocAllocator<'b, A>,
    D::Doc: Clone,
    A: Clone,
{
    if text.is_empty() {
        alloc.nil()
    } else {
        alloc.reflow(text)
    }
}

#[cfg(test)]
mod tests {
    use std::{ffi::OsStr, path::PathBuf};

    use crate::{configuration::Configuration, format};

    #[test]
    fn check_tests_dir() -> anyhow::Result<()> {
        const ANSI_RESET: &str = "\x1b[0m";
        const ANSI_RED: &str = "\x1b[31m";
        const ANSI_GREEN: &str = "\x1b[32m";
        const ANSI_BOLD_GREEN: &str = "\x1b[1;32m";

        let configuration = Configuration {
            line_width: 80,
            indent_width: 2,
        };

        let mut failed = false;

        for entry in std::fs::read_dir(
            [&std::env::var("CARGO_MANIFEST_DIR")?, "tests"]
                .into_iter()
                .collect::<PathBuf>(),
        )? {
            let path = entry?.path();

            match path.extension().and_then(OsStr::to_str) {
                Some("html") => {}
                _ => continue,
            }

            println!(
                "{ANSI_BOLD_GREEN}{:>12}{ANSI_RESET} format test file ({})",
                "Checking",
                path.display()
            );

            let raw = std::fs::read_to_string(&path)?;
            let pretty = format(&raw, &configuration)?;

            if raw != pretty {
                use similar::{ChangeTag, TextDiff};

                failed = true;

                for change in TextDiff::from_lines(&raw, &pretty).iter_all_changes() {
                    match change.tag() {
                        ChangeTag::Delete => print!("{ANSI_RED}-{change}{ANSI_RESET}"),
                        ChangeTag::Insert => print!("{ANSI_GREEN}+{change}{ANSI_RESET}"),
                        ChangeTag::Equal => print!(" {change}"),
                    }
                }
            }
        }

        if failed {
            Err(anyhow::anyhow!("At least one check failed"))
        } else {
            Ok(())
        }
    }
}
