use nom::{branch::alt, combinator::fail, IResult, Parser};

use self::{comment::Comment, doctype::Doctype, element::Element};

pub mod comment;
pub mod doctype;
pub mod element;
mod util;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Node<'a> {
    Comment(Comment<'a>),
    Doctype(Doctype),
    Element(Element<'a>),
    // NOTE: Cannot contain an end tag.
    Text(&'a str),
}

impl From<Doctype> for Node<'_> {
    fn from(doctype: Doctype) -> Self {
        Self::Doctype(doctype)
    }
}

impl<'a> From<Element<'a>> for Node<'a> {
    fn from(element: Element<'a>) -> Self {
        Self::Element(element)
    }
}

impl<'a> From<Comment<'a>> for Node<'a> {
    fn from(comment: Comment<'a>) -> Self {
        Self::Comment(comment)
    }
}

impl<'a> Node<'a> {
    /// Consume input as text until:
    /// - a non-text node (returned in the second part of the tuple),
    /// - an end tag,
    /// - or eof.
    fn parse_text(input: &'a str) -> IResult<&'a str, (Self, Option<Self>)> {
        let mut index = 0;

        let input = input.trim_start();

        if input.is_empty() {
            return fail(input);
        }

        loop {
            if let Some(delta) = input.get(index..).and_then(|input| input.find('<')) {
                index += delta;

                if Element::parse_end_tag(&input[index..]).is_ok() {
                    break Ok((
                        &input[index..],
                        (Self::Text(input[..index].trim_end()), None),
                    ));
                }

                if let Ok((remaining, next)) = Self::parse_non_text(&input[index..]) {
                    break Ok((
                        remaining,
                        (Self::Text(input[..index].trim_end()), Some(next)),
                    ));
                }

                index += 1;
            } else {
                break Ok(("", (Self::Text(input.trim_end()), None)));
            }
        }
    }

    fn parse_non_text(input: &'a str) -> IResult<&'a str, Self> {
        alt((
            Comment::parse.map(Self::from),
            Doctype::parse.map(Self::from),
            Element::parse.map(Self::from),
        ))
        .parse(input)
    }

    /// Consume input as long as it parses into a node.
    pub fn parse_many(input: &'a str) -> IResult<&'a str, Vec<Self>> {
        let mut remaining = input.trim_start();
        let mut buffer = Vec::new();

        loop {
            if Element::parse_end_tag(remaining).is_ok() {
                break Ok((remaining, buffer));
            }

            if remaining.is_empty() {
                break Ok(("", buffer));
            }

            if let Ok((rest, node)) = Self::parse_non_text(remaining.trim_start()) {
                buffer.push(node);
                remaining = rest.trim_start();
            } else {
                let (rest, (node, next)) = Self::parse_text(remaining)?;

                buffer.push(node);

                if let Some(node) = next {
                    buffer.push(node);
                }

                remaining = rest.trim_start();
            }
        }
    }
}
