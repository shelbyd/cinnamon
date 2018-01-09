use super::*;

named!(pub escaped<String>, alt_complete!(
    value!(String::new(), peek!(tag!("\""))) |
    map!(escaped_transform!(
        is_not!("\\\""),
        '\\',
        alt!(
            value!(&b"\""[..], char!('"')) |
            value!(&b"\\"[..], take!(0))
        )),
    |s| into_string(&s))
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_string() {
        assert_eq!(escaped(&b""[..]), IResult::Done(&b""[..], "".to_owned()));
    }

    #[test]
    fn no_problem_characters() {
        assert_eq!(
            escaped(&b"foo"[..]),
            IResult::Done(&b""[..], "foo".to_owned())
        );
    }

    #[test]
    fn unescaped_terminal() {
        assert_eq!(
            escaped(&b"f\"oo"[..]),
            IResult::Done(&b"\"oo"[..], "f".to_owned())
        );
    }

    #[test]
    fn escaped_terminal() {
        assert_eq!(
            escaped(&b"f\\\"oo"[..]),
            IResult::Done(&b""[..], "f\"oo".to_owned())
        );
    }

    #[test]
    fn multiple_escaped_terminals() {
        assert_eq!(
            escaped(&b"f\\\"o\\\"o"[..]),
            IResult::Done(&b""[..], "f\"o\"o".to_owned())
        );
    }

    #[test]
    fn escaped_after_unescaped() {
        assert_eq!(
            escaped(&b"f\"o\\\"o"[..]),
            IResult::Done(&b"\"o\\\"o"[..], "f".to_owned())
        );
    }

    #[test]
    fn immediate_terminal() {
        assert_eq!(
            escaped(&b"\""[..]),
            IResult::Done(&b"\""[..], "".to_owned())
        );
    }

    #[test]
    fn newline() {
        assert_eq!(
            escaped(&b"\\n"[..]),
            IResult::Done(&b""[..], "\\n".to_owned())
        );
    }

    #[test]
    fn other_escapes() {
        assert_eq!(
            escaped(&b"\\t\\r\\'"[..]),
            IResult::Done(&b""[..], "\\t\\r\\'".to_owned())
        );
    }

    #[test]
    fn starts_with_escaped_terminal() {
        assert_eq!(
            escaped(&b"\\\""[..]),
            IResult::Done(&b""[..], "\"".to_owned())
        );
    }
}
