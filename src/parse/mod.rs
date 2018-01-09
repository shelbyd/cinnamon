mod command;
mod comment;

use ast::*;
use nom::*;
use self::command::*;
use self::comment::*;

pub trait Parse: Sized {
    fn parse(self) -> Result<Vec<AST>, ErrorKind>;
}

impl Parse for Vec<u8> {
    fn parse(self) -> Result<Vec<AST>, ErrorKind> {
        parse(&self[..])
            .to_result()
            .map_err(|e| e.into_error_kind())
    }
}

fn into_string<'a>(bytes: &'a [u8]) -> String {
    String::from_utf8(bytes.to_owned()).unwrap()
}

named!(parse_tree<&[u8], Vec<AST>>, ws!(many0!(alt_complete!(
    comment => { |s| AST::Comment(s) } |
    command => { |c| AST::Command(c) }
))));

named!(parse<&[u8], Vec<AST>>, do_parse!(
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
            vec![AST::Comment(" This is a comment!".to_owned())]
        );
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
                AST::Comment(" Comment".to_owned()),
                AST::Comment(" Another".to_owned()),
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
