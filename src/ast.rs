use std::process::{Command as ProcessCommand, ExitStatus};

use failure::*;

trait Executor {
    type ExitStatus: Success;

    fn execute(&mut self, command: &str, args: &[&str]) -> Result<Self::ExitStatus, Error>;
}

trait Success {
    fn success(&self) -> bool;
}

struct StdExecutor;

impl Executor for StdExecutor {
    type ExitStatus = ExitStatus;

    fn execute(&mut self, command: &str, args: &[&str]) -> Result<Self::ExitStatus, Error> {
        let exit = ProcessCommand::new(command).args(args).spawn()?.wait()?;
        Ok(exit)
    }
}

impl Success for ExitStatus {
    fn success(&self) -> bool {
        self.success()
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum AST {
    Comment(String),
    Command(Command),
    If(Conditional),
    Block(Block),
    While(While),
}

impl AST {
    pub fn execute(&self) -> Result<Option<ExitStatus>, Error> {
        self.execute_with(&mut StdExecutor)
    }

    fn execute_with<E: Executor>(&self, executor: &mut E) -> Result<Option<E::ExitStatus>, Error> {
        match self {
            AST::Comment(_) => Ok(None),
            AST::Command(c) => c.execute(executor).map(Some),
            AST::If(c) => c.execute(executor),
            AST::Block(b) => b.execute(executor),
            AST::While(w) => w.execute(executor),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub struct Command {
    command: String,
    args: Vec<String>,
}

impl Command {
    pub fn new<S>(s: S, args: Vec<S>) -> Command
    where
        S: ToString,
    {
        Command {
            command: s.to_string(),
            args: args.into_iter().map(|s| s.to_string()).collect(),
        }
    }

    fn no_args<S: ToString>(s: S) -> Command {
        Self::new(s, vec![])
    }
}

impl Command {
    fn execute<E: Executor>(&self, executor: &mut E) -> Result<E::ExitStatus, Error> {
        let exit = executor.execute(
            &self.command,
            &self.args.iter().map(AsRef::as_ref).collect::<Vec<_>>(),
        )?;
        Ok(exit)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Conditional {
    predicate: Command,
    if_block: Box<AST>,
    else_block: Option<Box<AST>>,
}

impl Conditional {
    pub fn new(predicate: Command, if_block: AST, else_block: Option<AST>) -> Conditional {
        Conditional {
            predicate,
            if_block: Box::new(if_block),
            else_block: else_block.map(Box::new),
        }
    }

    fn execute<E: Executor>(&self, executor: &mut E) -> Result<Option<E::ExitStatus>, Error> {
        if self.predicate.execute(executor)?.success() {
            self.if_block.execute_with(executor)
        } else {
            match &self.else_block {
                None => Ok(None),
                Some(b) => b.execute_with(executor),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block(pub Vec<AST>);

impl Block {
    fn execute<E: Executor>(&self, executor: &mut E) -> Result<Option<E::ExitStatus>, Error> {
        let mut last = None;

        let iter = self.0.iter().map(|ast| ast.execute_with(executor));
        for exit in iter {
            last = exit?.or(last);
            if let Some(last) = &last {
                if !last.success() {
                    break;
                }
            }
        }
        Ok(last)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct While {
    predicate: Command,
    block: Box<AST>,
}

impl While {
    pub fn new(predicate: Command, block: AST) -> While {
        While {
            predicate,
            block: Box::new(block),
        }
    }

    fn execute<E: Executor>(&self, executor: &mut E) -> Result<Option<E::ExitStatus>, Error> {
        let mut last = None;

        // TODO(shelbyd): Remove duplication between this and Block.
        while self.predicate.execute(executor)?.success() {
            let exit = self.block.execute_with(executor);
            last = exit?.or(last);
            if let Some(last) = &last {
                if !last.success() {
                    break;
                }
            }
        }
        Ok(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;

    struct TestExecutor {
        history: Vec<(String, Vec<String>)>,
        future: VecDeque<Future>,
    }

    impl TestExecutor {
        fn new() -> TestExecutor {
            TestExecutor {
                history: Vec::new(),
                future: VecDeque::new(),
            }
        }

        fn last(&self) -> Option<(&str, Vec<&str>)> {
            self.history
                .last()
                .map(|(s, a)| (s.as_ref(), a.iter().map(|s| s.as_ref()).collect()))
        }

        fn will_fail(&mut self) {
            self.future.push_back(Future::Fail);
        }

        fn will_succeed(&mut self) {
            self.future.push_back(Future::Success);
        }

        fn will_error(&mut self, error: Error) {
            self.future.push_back(Future::Error(error));
        }

        fn count(&self, cmd: &str) -> usize {
            self.history.iter().filter(|(c, _)| c == cmd).count()
        }
    }

    enum Future {
        Fail,
        Success,
        Error(Error),
    }

    impl Executor for TestExecutor {
        type ExitStatus = bool;

        fn execute(&mut self, command: &str, args: &[&str]) -> Result<Self::ExitStatus, Error> {
            self.history.push((
                command.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            match self.future.pop_front() {
                None => Ok(true),
                Some(Future::Fail) => Ok(false),
                Some(Future::Success) => Ok(true),
                Some(Future::Error(e)) => Err(e),
            }
        }
    }

    impl Success for bool {
        fn success(&self) -> bool {
            *self
        }
    }

    fn cmd(s: &str) -> Command {
        Command::no_args(s)
    }

    #[cfg(test)]
    mod command {
        use super::*;

        #[test]
        fn executes_using_the_provided_executor() {
            let mut executor = TestExecutor::new();
            let command = Command::no_args("foo");

            command.execute(&mut executor).unwrap();

            assert_eq!(executor.last(), Some(("foo", vec![])));
        }
    }

    #[cfg(test)]
    mod conditional {
        use super::*;

        #[test]
        fn executes_if_block_if_predicate_is_true() {
            let mut executor = TestExecutor::new();
            let conditional = Conditional::new(cmd("foo"), AST::Command(cmd("bar")), None);

            conditional.execute(&mut executor).unwrap();

            assert_eq!(executor.last(), Some(("bar", vec![])));
        }

        #[test]
        fn does_not_execute_if_block_if_predicate_is_false() {
            let mut executor = TestExecutor::new();
            let conditional = Conditional::new(cmd("foo"), AST::Command(cmd("bar")), None);

            executor.will_fail();
            conditional.execute(&mut executor).unwrap();

            assert_eq!(executor.last(), Some(("foo", vec![])));
        }

        #[test]
        fn executes_else_block_if_predicate_fails() {
            let mut executor = TestExecutor::new();
            let conditional = Conditional::new(
                cmd("foo"),
                AST::Command(cmd("bar")),
                Some(AST::Command(cmd("baz"))),
            );

            executor.will_fail();
            conditional.execute(&mut executor).unwrap();

            assert_eq!(executor.last(), Some(("baz", vec![])));
        }
    }

    #[cfg(test)]
    mod block {
        use super::*;

        #[test]
        fn returns_ok_none_with_empty_block() {
            let mut executor = TestExecutor::new();
            let block = Block(vec![]);

            assert_eq!(block.execute(&mut executor).unwrap(), None);
        }

        #[test]
        fn returns_ok_true_if_one_success() {
            let mut executor = TestExecutor::new();
            let block = Block(vec![AST::Command(cmd("foo"))]);

            assert_eq!(block.execute(&mut executor).unwrap(), Some(true));
        }

        #[test]
        fn returns_ok_false_if_first_fails() {
            let mut executor = TestExecutor::new();
            let block = Block(vec![AST::Command(cmd("foo")), AST::Command(cmd("bar"))]);

            executor.will_fail();

            assert_eq!(block.execute(&mut executor).unwrap(), Some(false));
        }

        #[test]
        fn returns_ok_false_if_second_fails() {
            let mut executor = TestExecutor::new();
            let block = Block(vec![AST::Command(cmd("foo")), AST::Command(cmd("bar"))]);

            executor.will_succeed();
            executor.will_fail();

            assert_eq!(block.execute(&mut executor).unwrap(), Some(false));
        }

        #[test]
        fn returns_ok_none_if_only_comment() {
            let mut executor = TestExecutor::new();
            let block = Block(vec![AST::Comment(String::from("comment"))]);

            assert_eq!(block.execute(&mut executor).unwrap(), None);
        }

        #[test]
        fn returns_err_if_command_errors() {
            let mut executor = TestExecutor::new();
            let block = Block(vec![AST::Command(cmd("foo"))]);

            executor.will_error(failure::err_msg("error"));

            assert!(block.execute(&mut executor).is_err());
        }
    }

    #[cfg(test)]
    mod while_ {
        use super::*;

        #[test]
        fn returns_ok_none_if_predicate_fails() {
            let mut executor = TestExecutor::new();
            let while_ = While::new(cmd("foo"), AST::Block(Block(vec![])));

            executor.will_fail();

            assert_eq!(while_.execute(&mut executor).unwrap(), None);
        }

        #[test]
        fn executes_block_once() {
            let mut executor = TestExecutor::new();
            let while_ = While::new(cmd("foo"), AST::Command(cmd("bar")));

            executor.will_succeed();
            executor.will_succeed();
            executor.will_fail();

            while_.execute(&mut executor).unwrap();

            assert_eq!(executor.count("bar"), 1);
        }

        #[test]
        fn executes_block_thrice() {
            let mut executor = TestExecutor::new();
            let while_ = While::new(cmd("foo"), AST::Command(cmd("bar")));

            executor.will_succeed();
            executor.will_succeed();

            executor.will_succeed();
            executor.will_succeed();

            executor.will_succeed();
            executor.will_succeed();

            executor.will_fail();

            while_.execute(&mut executor).unwrap();

            assert_eq!(executor.count("bar"), 3);
        }

        #[test]
        fn block_failure_breaks_loop() {
            let mut executor = TestExecutor::new();
            let while_ = While::new(cmd("foo"), AST::Command(cmd("bar")));

            executor.will_succeed();
            executor.will_fail();

            assert_eq!(while_.execute(&mut executor).unwrap(), Some(false));
        }

        #[test]
        fn one_loop_returns_last() {
            let mut executor = TestExecutor::new();
            let while_ = While::new(cmd("foo"), AST::Command(cmd("bar")));

            executor.will_succeed();
            executor.will_succeed();
            executor.will_fail();

            assert_eq!(while_.execute(&mut executor).unwrap(), Some(true));
        }

        #[test]
        fn error_in_predicate_breaks() {
            let mut executor = TestExecutor::new();
            let while_ = While::new(cmd("foo"), AST::Command(cmd("bar")));

            executor.will_error(failure::err_msg("err"));

            assert!(while_.execute(&mut executor).is_err());
        }
    }
}
