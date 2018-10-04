use super::*;
use crate::ast::*;
use nom::*;

named!(
    block<Vec<AST>>,
    do_parse!(tag!("{") >> tree: parse_tree >> tag!("}") >> (tree))
);

named!(
    els<Option<Vec<AST>>>,
    opt!(complete!(ws!(preceded!(tag!("else"), block))))
);

named!(pub if_stmt<Conditional>, do_parse!(
        tag!("if") >>
        command: ws!(command) >>
        block: block >>
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
                Conditional::new(Command::new("true", vec![]), vec![], None)
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
                    vec![
                        AST::Command(Command::new("echo", vec!["foo"])),
                        AST::Command(Command::new("echo", vec!["bar"])),
                    ],
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
                Conditional::new(Command::new("false", vec![]), vec![], Some(vec![]))
            )
        );
    }
}
