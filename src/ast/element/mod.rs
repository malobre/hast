use nom::{
    bytes::complete::{take_while, take_while1},
    character::complete::char,
    combinator::{fail, opt},
    multi::many0,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

use crate::ast::Node;

use self::util::{parse_attribute, parse_tag_name};

use super::util::is_ascii_whitespace;

mod util;

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Element<'a> {
    Normal {
        name: &'a str,
        attributes: Vec<(&'a str, Option<&'a str>)>,
        content: Vec<Node<'a>>,
    },
    Void {
        name: &'a str,
        attributes: Vec<(&'a str, Option<&'a str>)>,
    },
}

impl<'a> Element<'a> {
    pub fn parse_end_tag(input: &'a str) -> IResult<&'a str, &'a str> {
        delimited(
            tuple((char('<'), char('/'))),
            parse_tag_name,
            tuple((take_while(is_ascii_whitespace), char('>'))),
        )
        .parse(input)
    }

    pub fn parse(input: &'a str) -> IResult<&'a str, Self> {
        let (input, (name, attributes, self_closing)) = delimited(
            char('<'),
            tuple((
                parse_tag_name,
                many0(preceded(take_while1(is_ascii_whitespace), parse_attribute)),
                preceded(
                    take_while(is_ascii_whitespace),
                    opt(char('/')).map(|solidus| solidus.is_some()),
                ),
            )),
            char('>'),
        )
        .parse(input)?;

        if matches!(
            name.to_ascii_lowercase().as_str(),
            "area"
                | "base"
                | "br"
                | "col"
                | "embed"
                | "hr"
                | "img"
                | "input"
                | "link"
                | "meta"
                | "param"
                | "source"
                | "track"
                | "wbr"
        ) || self_closing
        {
            return Ok((input, Self::Void { name, attributes }));
        }

        let (input, content) = Node::parse_many(input)?;

        let (input, end_name) = Self::parse_end_tag(input)?;

        if name != end_name {
            return fail(input);
        }

        Ok((
            input,
            Self::Normal {
                name,
                attributes,
                content,
            },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Element;

    #[test]
    fn test_parse_void_element() {
        assert_eq!(
            Element::parse("<input>"),
            Ok((
                "",
                Element::Void {
                    name: "input",
                    attributes: vec![],
                }
            ))
        );

        assert_eq!(
            Element::parse(r#"<input type="text" required>"#),
            Ok((
                "",
                Element::Void {
                    name: "input",
                    attributes: vec![("type", Some("text")), ("required", None)],
                }
            ))
        );
    }

    #[test]
    fn test_parse_self_closing_element() {
        assert_eq!(
            Element::parse("<MyComponent/>"),
            Ok((
                "",
                Element::Void {
                    name: "MyComponent",
                    attributes: vec![],
                }
            ))
        );

        assert_eq!(
            Element::parse(r#"<MyComponent :attr="yes"/>"#),
            Ok((
                "",
                Element::Void {
                    name: "MyComponent",
                    attributes: vec![(":attr", Some("yes")),],
                }
            ))
        );
    }
}
