use super::*;
use crate::ast::*;
use nom::*;

named!(
    block<Vec<AST>>,
    do_parse!(tag!("{") >> tree: parse_tree >> tag!("}") >> (tree))
);

named!(pub if_stmt<(Command, Vec<AST>)>, do_parse!(
        tag!("if") >>
        command: ws!(command) >>
        block: block >>
        (command, block)
      ));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_and_empty_block() {
        assert_eq!(
            if_stmt(&b"if true {}"[..]),
            IResult::Done(&b""[..], (Command::new("true", vec![]), vec![]))
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
                (
                    Command::new("true", vec![]),
                    vec![
                        AST::Command(Command::new("echo", vec!["foo"])),
                        AST::Command(Command::new("echo", vec!["bar"])),
                    ]
                )
            )
        );
    }
}
