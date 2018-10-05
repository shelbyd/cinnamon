use super::*;
use crate::ast::*;
use nom::*;

named!(pub while_stmt<While>, do_parse!(
        tag!("while") >>
        command: ws!(command) >>
        block: ast >>
        (While::new(command, block))
      ));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn true_and_empty_block() {
        assert_eq!(
            while_stmt(&b"while true {}"[..]),
            IResult::Done(
                &b""[..],
                While::new(Command::new("true", vec![]), AST::Block(Block(vec![])))
            )
        );
    }
}
