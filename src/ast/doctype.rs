use nom::{
    branch::alt,
    bytes::complete::{tag, tag_no_case, take_while1},
    character::complete::char,
    combinator::opt,
    sequence::{delimited, preceded, tuple},
    IResult, Parser,
};

use super::util::is_ascii_whitespace;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash)]
pub struct Doctype {
    pub legacy: bool,
}

impl Doctype {
    pub fn parse(input: &str) -> IResult<&str, Self> {
        delimited(
            tuple((
                char('<'),
                char('!'),
                tag_no_case("DOCTYPE"),
                take_while1(is_ascii_whitespace),
                tag_no_case("html"),
            )),
            opt(preceded(
                take_while1(is_ascii_whitespace),
                parse_legacy_string,
            ))
            .map(|legacy| legacy.is_some()),
            char('>'),
        )
        .map(|legacy| Doctype { legacy })
        .parse(input)
    }
}

fn parse_legacy_string(input: &str) -> IResult<&str, ()> {
    tuple((
        tag_no_case("SYSTEM"),
        take_while1(is_ascii_whitespace),
        alt((
            delimited(char('"'), tag("about:legacy-compat"), char('"')),
            delimited(char('\''), tag("about:legacy-compat"), char('\'')),
        )),
    ))
    .map(|_| ())
    .parse(input)
}

#[cfg(test)]
mod tests {
    use super::{parse_legacy_string, Doctype};

    #[test]
    fn test_parse_legacy_string() {
        assert_eq!(
            parse_legacy_string(r#"SYSTEM "about:legacy-compat""#),
            Ok(("", ()))
        );

        assert_eq!(
            parse_legacy_string("SYSTEM 'about:legacy-compat'"),
            Ok(("", ()))
        );
    }

    #[test]
    fn test_parse_doctype() {
        assert_eq!(
            Doctype::parse("<!DOCTYPE html>"),
            Ok(("", Doctype { legacy: false }))
        );

        assert_eq!(
            Doctype::parse("<!DOCTYPE html SYSTEM 'about:legacy-compat'>"),
            Ok(("", Doctype { legacy: true }))
        );
    }
}
