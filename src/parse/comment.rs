use nom::*;
use super::*;

named!(pub comment<&[u8], String>, map!(
    delimited!(tag!("#"), not_line_ending, alt_complete!(eol | eof!())),
    into_string
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_hash() {
        assert_eq!(comment(&b"#"[..]), IResult::Done(&b""[..], "".to_owned()));
    }

    #[test]
    fn text() {
        assert_eq!(
            comment(&b"# text"[..]),
            IResult::Done(&b""[..], " text".to_owned())
        );
    }

    #[test]
    fn another_line() {
        assert_eq!(
            comment(&b"# text\n# other text"[..]),
            IResult::Done(&b"# other text"[..], " text".to_owned())
        );
    }

    #[test]
    fn windows_line_ending() {
        assert_eq!(
            comment(&b"# text\r\n"[..]),
            IResult::Done(&b""[..], " text".to_owned())
        );
    }

    #[test]
    fn escaped_newline() {
        assert_eq!(
            comment(&b"# text\\n"[..]),
            IResult::Done(&b""[..], " text\\n".to_owned())
        );
    }
}
