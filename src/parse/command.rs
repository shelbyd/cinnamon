use ast::*;
use nom::*;
use super::*;
use super::escaped::escaped as escaped_string;

named!(
    path<String>,
    map!(
        recognize!(many1!(alt_complete!(alphanumeric | tag!("/")))),
        into_string
    )
);

named!(
    arg<String>,
    alt_complete!(
        delimited!(tag!("\""), escaped_string, tag!("\"")) | map!(is_not!("; \""), into_string)
    )
);

named!(pub command<Command>, do_parse!(
    path: path >>
    args: many0!(ws!(arg)) >>
    char!(';') >>
    (Command::new(path, args))
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha() {
        assert_eq!(
            command(&b"echo;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec![]))
        );
    }

    #[test]
    fn alpha_numeric() {
        assert_eq!(
            command(&b"echo2;"[..]),
            IResult::Done(&b""[..], Command::new("echo2", vec![]))
        );
    }

    #[test]
    fn includes_path() {
        assert_eq!(
            command(&b"/bin/echo;"[..]),
            IResult::Done(&b""[..], Command::new("/bin/echo", vec![]))
        );
    }

    #[test]
    fn empty_string() {
        assert!(command(&b";"[..]).is_err());
    }

    #[test]
    fn bare_word_argument() {
        assert_eq!(
            command(&b"echo foo;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo"]))
        );
    }

    #[test]
    fn three_bare_word_arguments() {
        assert_eq!(
            command(&b"echo foo bar baz;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo", "bar", "baz"]))
        );
    }

    #[test]
    fn extra_spaces() {
        assert_eq!(
            command(&b"echo foo    bar;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo", "bar"]))
        );
    }

    #[cfg(test)]
    mod arg {
        use super::*;

        #[test]
        fn numeric_argument() {
            assert_eq!(arg(&b"5"[..]), IResult::Done(&b""[..], "5".to_owned()));
        }

        #[test]
        fn path_argument() {
            assert_eq!(
                arg(&b"/bin/bash"[..]),
                IResult::Done(&b""[..], "/bin/bash".to_owned())
            );
        }

        #[test]
        fn special_characters() {
            assert_eq!(
                arg(&b"foo-_.,baz"[..]),
                IResult::Done(&b""[..], "foo-_.,baz".to_owned())
            );
        }

        #[test]
        fn semicolon() {
            assert_eq!(
                arg(&b"foo;"[..]),
                IResult::Done(&b";"[..], "foo".to_owned())
            );
        }

        #[test]
        fn empty_string() {
            assert_eq!(arg(&b""[..]), IResult::Done(&b""[..], "".to_owned()));
        }

        #[test]
        fn empty_quotes() {
            assert_eq!(arg(&b"\"\""[..]), IResult::Done(&b""[..], "".to_owned()));
        }

        #[test]
        fn semicolon_wrapped_in_quotes() {
            assert_eq!(arg(&b"\";\""[..]), IResult::Done(&b""[..], ";".to_owned()));
        }

        #[test]
        fn spaces_in_quotes() {
            assert_eq!(
                arg(&b"\"foo bar baz\""[..]),
                IResult::Done(&b""[..], "foo bar baz".to_owned())
            );
        }

        #[test]
        fn single_quote() {
            assert!(arg(&b"\""[..]).is_err());
        }

        #[test]
        fn escaped_single_quote() {
            assert_eq!(
                arg(&b"\"\\\"\""[..]),
                IResult::Done(&b""[..], "\"".to_owned())
            );
        }
    }
}
