use nom::*;

#[derive(PartialEq, Eq, Debug)]
pub enum ParseTree {
    Comment(String),
    Command(Command),
}

pub trait Parse: Sized {
    fn parse(self) -> Result<Vec<ParseTree>, ErrorKind>;
}

impl Parse for Vec<u8> {
    fn parse(self) -> Result<Vec<ParseTree>, ErrorKind> {
        parse(&self[..]).to_result()
    }
}

fn into_string<'a>(bytes: &'a [u8]) -> String {
    String::from_utf8(bytes.to_owned()).unwrap()
}

named!(comment<&[u8], String>, map!(
    delimited!(tag!("#"), not_line_ending, alt_complete!(eol | eof!())),
    into_string
));

named!(path<&[u8], String>, map!(recognize!(
    many1!(alt_complete!(alphanumeric | tag!("/")))
), into_string));

#[derive(PartialEq, Eq, Debug)]
pub struct Command(String);

named!(command<&[u8], Command>, do_parse!(
    path: path >>
    char!(';') >>
    (Command(path))
));

named!(parse_tree<&[u8], Vec<ParseTree>>, ws!(many0!(alt_complete!(
    comment => { |s| ParseTree::Comment(s) } |
    command => { |c| ParseTree::Command(c) }
))));

named!(parse<&[u8], Vec<ParseTree>>, do_parse!(
    parse_tree: ws!(parse_tree) >>
    eof!() >>
    (parse_tree)
));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_is_empty_list() {
        let file: Vec<u8> = "".into();

        assert_eq!(file.parse().unwrap(), vec![]);
    }

    #[test]
    fn comment_is_comment() {
        let file: Vec<u8> = "# This is a comment!".into();

        assert_eq!(
            file.parse().unwrap(),
            vec![ParseTree::Comment(" This is a comment!".to_owned())]
        );
    }

    #[cfg(test)]
    mod comment {
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

    #[cfg(test)]
    mod command {
        use super::*;

        #[test]
        fn alpha() {
            assert_eq!(
                command(&b"echo;"[..]),
                IResult::Done(&b""[..], Command("echo".to_owned()))
            );
        }

        #[test]
        fn alpha_numeric() {
            assert_eq!(
                command(&b"echo2;"[..]),
                IResult::Done(&b""[..], Command("echo2".to_owned()))
            );
        }

        #[test]
        fn includes_path() {
            assert_eq!(
                command(&b"/bin/echo;"[..]),
                IResult::Done(&b""[..], Command("/bin/echo".to_owned()))
            );
        }

        #[test]
        fn empty_string() {
            assert!(command(&b";"[..]).is_err());
        }
    }

    #[test]
    fn empty_line() {
        let file: Vec<u8> = "\n".into();

        assert_eq!(file.parse().unwrap(), vec![]);
    }

    #[test]
    fn empty_line_between_comments() {
        let file: Vec<u8> = "# Comment\n\n# Another".into();

        assert_eq!(
            file.parse().unwrap(),
            vec![
                ParseTree::Comment(" Comment".to_owned()),
                ParseTree::Comment(" Another".to_owned()),
            ]
        );
    }

    #[test]
    fn trailing_characters() {
        let file: Vec<u8> = vec![1];
        let parse = file.parse();

        assert!(parse.is_err(), "Expected {:?} to be an error", parse);
    }
}
