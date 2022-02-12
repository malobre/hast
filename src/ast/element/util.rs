use nom::{
    branch::alt,
    bytes::complete::{take_till, take_until, take_while, take_while1},
    character::complete::char,
    combinator::opt,
    sequence::{delimited, pair, preceded, tuple},
    IResult,
};

use crate::ast::util::is_ascii_whitespace;

/// See <https://html.spec.whatwg.org/multipage/syntax.html#attributes-2>.
pub fn parse_attribute_name(input: &str) -> IResult<&str, &str> {
    take_while1(|char: char| {
        !matches!(char,
        '\u{007F}'..='\u{009F}'
        | '\u{0020}'
        | '\u{0022}'
        | '\u{0027}'
        | '\u{002F}'
        | '\u{003E}'
        | '\u{003D}'
        | '\u{FDD0}'..='\u{FDEF}'
        | '\u{FFFE}'
        | '\u{FFFF}'
        | '\u{1FFFE}'
        | '\u{1FFFF}'
        | '\u{2FFFE}'
        | '\u{2FFFF}'
        | '\u{3FFFE}'
        | '\u{3FFFF}'
        | '\u{4FFFE}'
        | '\u{4FFFF}'
        | '\u{5FFFE}'
        | '\u{5FFFF}'
        | '\u{6FFFE}'
        | '\u{6FFFF}'
        | '\u{7FFFE}'
        | '\u{7FFFF}'
        | '\u{8FFFE}'
        | '\u{8FFFF}'
        | '\u{9FFFE}'
        | '\u{9FFFF}'
        | '\u{AFFFE}'
        | '\u{AFFFF}'
        | '\u{BFFFE}'
        | '\u{BFFFF}'
        | '\u{CFFFE}'
        | '\u{CFFFF}'
        | '\u{DFFFE}'
        | '\u{DFFFF}'
        | '\u{EFFFE}'
        | '\u{EFFFF}'
        | '\u{FFFFE}'
        | '\u{FFFFF}'
        | '\u{10FFFE}'
        | '\u{10FFFF}')
    })(input)
}

/// See <https://html.spec.whatwg.org/multipage/syntax.html#attributes-2>.
pub fn parse_attribute(input: &str) -> IResult<&str, (&str, Option<&str>)> {
    pair(
        parse_attribute_name,
        opt(preceded(
            tuple((
                take_while(is_ascii_whitespace),
                char('='),
                take_while(is_ascii_whitespace),
            )),
            alt((
                delimited(char('"'), take_until("\""), char('"')),
                delimited(char('\''), take_until("'"), char('\'')),
                take_while1(|char: char| {
                    !char.is_ascii_whitespace()
                        && !matches!(
                            char,
                            '\u{0022}' | '\u{0027}' | '\u{003C}'..='\u{003E}' | '\u{0060}'
                        )
                }),
            )),
        )),
    )(input)
}

/// See <https://html.spec.whatwg.org/multipage/syntax.html#syntax-tag-name>.
pub fn parse_tag_name(input: &str) -> IResult<&str, &str> {
    take_till(|char: char| char.is_ascii_whitespace() || char == '/' || char == '>')(input)
}

#[cfg(test)]
mod tests {
    use super::{parse_attribute, parse_attribute_name, parse_tag_name};

    #[test]
    fn test_parse_attribute_name() {
        assert_eq!(
            parse_attribute_name(r#"lang="ts" setup>"#),
            Ok((r#"="ts" setup>"#, "lang"))
        );

        assert_eq!(parse_attribute_name("setup>"), Ok((">", "setup")));

        assert!(parse_attribute_name("> text").is_err(),);
    }

    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            parse_attribute(r#"lang="ts" setup>"#),
            Ok((" setup>", ("lang", Some("ts"))))
        );

        assert_eq!(parse_attribute("setup>"), Ok((">", ("setup", None))));
    }

    #[test]
    fn test_parse_tag_name() {
        assert_eq!(parse_tag_name("script>"), Ok((">", "script")));
        assert_eq!(parse_tag_name("script >"), Ok((" >", "script")));
        assert_eq!(parse_tag_name("script\t>"), Ok(("\t>", "script")));
        assert_eq!(parse_tag_name("script \t>"), Ok((" \t>", "script")));
        assert_eq!(
            parse_tag_name(r#"script lang="ts">"#),
            Ok((r#" lang="ts">"#, "script"))
        );
    }
}
