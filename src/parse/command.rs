use super::escaped::escaped as escaped_string;
use super::*;
use crate::ast::*;
use nom::*;

named!(
    path<String>,
    map!(
        recognize!(many1!(alt_complete!(alphanumeric | tag!("/")))),
        into_string
    )
);

named!(
    bare_word<String>,
    map!(
        // TODO(shelbyd): Reduce duplication between this and escaped.rs
        escaped_transform!(
            is_not!(" \t\r\n;\"\\{)"),
            '\\',
            alt!(value!(&b"\"", char!('"')) | value!(&b"\\", take!(0)))
        ),
        |s| into_string(&s)
    )
);

named!(
    arg<String>,
    alt_complete!(delimited!(tag!("\""), escaped_string, tag!("\"")) | bare_word)
);

named!(
    raw_command<Command>,
    do_parse!(
        path: path
            >> args: ws!(separated_list_complete!(multispace, arg))
            >> (Command::new(path, args))
    )
);

named!(pub command<Command>,
       alt!(
           raw_command |
           delimited!(char!('('), command, char!(')'))
       ));

named!(pub command_line<Command>, do_parse!(
    command: command >>
    char!(';') >>
    (command)
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn alpha() {
        assert_eq!(
            command_line(&b"echo;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec![]))
        );
    }

    #[test]
    fn alpha_numeric() {
        assert_eq!(
            command_line(&b"echo2;"[..]),
            IResult::Done(&b""[..], Command::new("echo2", vec![]))
        );
    }

    #[test]
    fn includes_path() {
        assert_eq!(
            command_line(&b"/bin/echo;"[..]),
            IResult::Done(&b""[..], Command::new("/bin/echo", vec![]))
        );
    }

    #[test]
    fn empty_string() {
        assert!(command_line(&b";"[..]).is_err());
    }

    #[test]
    fn bare_word_argument() {
        assert_eq!(
            command_line(&b"echo foo;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo"]))
        );
    }

    #[test]
    fn three_bare_word_arguments() {
        assert_eq!(
            command_line(&b"echo foo bar baz;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo", "bar", "baz"]))
        );
    }

    #[test]
    fn extra_spaces() {
        assert_eq!(
            command_line(&b"echo foo    bar;"[..]),
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

        #[test]
        fn bare_word_and_escaped_quote() {
            assert_eq!(
                arg(&b"foo\\\"bar"[..]),
                IResult::Done(&b""[..], "foo\"bar".to_owned())
            );
        }

        #[test]
        fn newline_in_bare_word() {
            assert_eq!(
                arg(&b"foo\nbar"[..]),
                IResult::Done(&b"\nbar"[..], "foo".to_owned())
            );
        }

        #[test]
        fn escaped_newline_in_bare_word() {
            assert_eq!(
                arg(&b"foo\\nbar"[..]),
                IResult::Done(&b""[..], "foo\\nbar".to_owned())
            );
        }

        #[test]
        fn braces() {
            assert!(arg(&b"{}"[..]).is_err());
        }
    }

    #[test]
    fn quotes_next_to_bare_words() {
        assert!(command_line(&b"echo foo\"bar\";"[..]).is_err());
    }

    #[test]
    fn newlines_between_bare_words() {
        assert_eq!(
            command_line(&b"echo foo\nbar;"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo", "bar"]))
        );
    }

    #[test]
    fn surrounded_in_parens() {
        assert_eq!(
            command(&b"(echo foo bar)"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo", "bar"]))
        );
    }

    #[test]
    fn three_parens() {
        assert_eq!(
            command(&b"(((echo foo bar)))"[..]),
            IResult::Done(&b""[..], Command::new("echo", vec!["foo", "bar"]))
        );
    }
}
