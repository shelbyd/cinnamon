use super::*;
use crate::ast::*;
use nom::*;

named!(
    pub block<Vec<AST>>,
    do_parse!(tag!("{") >> tree: parse_tree >> tag!("}") >> (tree))
);

named!(
    els<Option<AST>>,
    opt!(complete!(ws!(preceded!(tag!("else"), ast))))
);

named!(pub if_stmt<Conditional>, do_parse!(
        tag!("if") >>
        command: ws!(command) >>
        block: ast >>
        els: els >>
        (Conditional::new(command, block, els))
      ));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_and_empty_block() {
        assert_eq!(
            if_stmt(&b"if true {}"[..]),
            IResult::Done(
                &b""[..],
                Conditional::new(
                    Command::new("true", vec![]),
                    AST::Block(Block(vec![])),
                    None
                )
            )
        );
    }

    #[test]
    fn block_with_statements() {
        assert_eq!(
            if_stmt(
                &b"if true {
                      echo foo;
                      echo bar;
                    }"[..]
            ),
            IResult::Done(
                &b""[..],
                Conditional::new(
                    Command::new("true", vec![]),
                    AST::Block(Block(vec![
                        AST::Command(Command::new("echo", vec!["foo"])),
                        AST::Command(Command::new("echo", vec!["bar"])),
                    ])),
                    None,
                )
            )
        );
    }

    #[test]
    fn empty_else_block() {
        assert_eq!(
            if_stmt(&b"if false {} else {}"[..]),
            IResult::Done(
                &b""[..],
                Conditional::new(
                    Command::new("false", vec![]),
                    AST::Block(Block(vec![])),
                    Some(AST::Block(Block(vec![])))
                )
            )
        );
    }

    #[test]
    fn empty_else_if() {
        assert_eq!(
            if_stmt(&b"if false {} else if false {}"[..]),
            IResult::Done(
                &b""[..],
                Conditional::new(
                    Command::new("false", vec![]),
                    AST::Block(Block(vec![])),
                    Some(AST::If(Conditional::new(
                        Command::new("false", vec![]),
                        AST::Block(Block(vec![])),
                        None
                    )))
                )
            )
        );
    }
}
