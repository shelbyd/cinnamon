use std::process::{Command as ProcessCommand, ExitStatus};

use failure::*;

struct StdExecutor;

impl Executor for StdExecutor {
    type ExitStatus = ExitStatus;

    fn execute(&mut self, command: &str, args: &[&str]) -> Result<Self::ExitStatus, Error> {
        let exit = ProcessCommand::new(command)
            .args(args)
            .spawn()?
            .wait()?;
        Ok(exit)
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
        let mut executor = StdExecutor;
        match self {
            AST::Comment(_) => Ok(None),
            AST::Command(c) => c.execute(&mut executor).map(Some),
            AST::If(c) => c.execute(),
            AST::Block(b) => b.execute(),
            AST::While(w) => w.execute(),
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

    fn execute(&self) -> Result<Option<ExitStatus>, Error> {
        if self.predicate.execute(&mut StdExecutor)?.success() {
            self.if_block.execute()
        } else {
            match &self.else_block {
                None => Ok(None),
                Some(b) => b.execute(),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block(pub Vec<AST>);

impl Block {
    fn execute(&self) -> Result<Option<ExitStatus>, Error> {
        self.0
            .iter()
            .map(|ast| ast.execute())
            .take_while(|exit| match exit {
                Err(_) => false,
                Ok(None) => true,
                Ok(Some(exit)) => exit.success(),
            }).last()
            .unwrap_or(Ok(None))
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

    fn execute(&self) -> Result<Option<ExitStatus>, Error> {
        let mut last_exit = None;
        while self.predicate.execute(&mut StdExecutor)?.success() {
            last_exit = self.block.execute()?.or(last_exit);
            if let Some(exit) = last_exit {
                if !exit.success() {
                    break;
                }
            }
        }
        Ok(last_exit)
    }
}

trait Executor {
    type ExitStatus;

    fn execute(&mut self, command: &str, args: &[&str]) -> Result<Self::ExitStatus, Error>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestExecutor {
        history: Vec<(String, Vec<String>)>,
    }

    impl TestExecutor {
        fn new() -> TestExecutor {
            TestExecutor {
                history: Vec::new(),
            }
        }

        fn last(&self) -> Option<(&str, Vec<&str>)> {
            self.history
                .last()
                .map(|(s, a)| (s.as_ref(), a.iter().map(|s| s.as_ref()).collect()))
        }
    }

    impl Executor for TestExecutor {
        type ExitStatus = ();

        fn execute(&mut self, command: &str, args: &[&str]) -> Result<Self::ExitStatus, Error> {
            self.history.push((
                command.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
            ));
            Ok(())
        }
    }

    #[cfg(test)]
    mod command {
        use super::*;

        #[test]
        fn executes_using_the_provided_executor() {
            let mut executor = TestExecutor::new();
            let command = Command::new("foo", vec![]);

            command.execute(&mut executor).unwrap();

            assert_eq!(executor.last(), Some(("foo", vec![])));
        }
    }
}
