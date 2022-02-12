use nom::{
    bytes::complete::{tag, take_until},
    sequence::delimited,
    IResult, Parser,
};

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Comment<'a>(pub &'a str);

impl<'a> Comment<'a> {
    pub fn parse(input: &str) -> IResult<&str, Comment> {
        delimited(tag("<!--"), take_until("-->"), tag("-->"))
            .map(|input: &str| {
                // TODO: Revisit this.

                let multiline = input.trim().contains('\n');

                if multiline {
                    let start = input.find('\n').unwrap() + 1;
                    let end = input.rfind('\n').unwrap();
                    &input[start..end]
                } else {
                    input.trim()
                }
            })
            .map(Comment)
            .parse(input)
    }
}

#[cfg(test)]
mod tests {
    use super::Comment;

    #[test]
    fn test_parse_inline_comment() {
        assert_eq!(
            Comment::parse("<!-- My comment -->"),
            Ok(("", Comment("My comment")))
        );
    }

    #[test]
    fn test_parse_single_line_comment() {
        assert_eq!(
            Comment::parse(
                "<!--
                    My comment
                -->"
            ),
            Ok(("", Comment("My comment")))
        );
    }

    #[test]
    fn test_parse_multiline_comment() {
        assert_eq!(
            Comment::parse(
                "<!--
                    My
                    multiline
                    comment
                -->"
            ),
            Ok((
                "",
                Comment(concat!(
                    "                    My\n",
                    "                    multiline\n",
                    "                    comment"
                ))
            ))
        );
    }
}
