mod command;
mod comment;
mod escaped;
mod if_stmt;
mod while_stmt;

use self::command::*;
use self::comment::*;
use self::if_stmt::*;
use self::while_stmt::*;
use crate::ast::*;
use nom::*;

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

named!(
    pub ast<AST>,
    alt_complete!(
        block => { |b| AST::Block(Block(b)) } |
        if_stmt => { |cond| AST::If(cond) } |
        while_stmt => { |stmt| AST::While(stmt) } |
        comment => { |s| AST::Comment(s) } |
        command_line => { |c| AST::Command(c) }
));

named!(
    pub parse_tree<Vec<AST>>,
    ws!(many0!(ast))
);

named!(
    parse<Vec<AST>>,
    do_parse!(parse_tree: ws!(parse_tree) >> eof!() >> (parse_tree))
);

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
