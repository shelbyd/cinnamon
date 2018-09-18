use std::process::{Command as ProcessCommand, ExitStatus};

use failure::*;

#[derive(PartialEq, Eq, Debug)]
pub enum AST {
    Comment(String),
    Command(Command),
    If(Command, Command),
}

impl AST {
    pub fn execute(self) -> Result<(), Error> {
        match self {
            AST::Comment(_) => Ok(()),
            AST::Command(c) => c.execute().map(|_| ()),
            AST::If(predicate, block) => {
                if predicate.execute()?.success() {
                    block.execute().map(|_| ())
                } else {
                    Ok(())
                }
            }
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
    fn execute(self) -> Result<ExitStatus, Error> {
        let exit = ProcessCommand::new(self.command)
            .args(self.args)
            .spawn()?
            .wait()?;
        Ok(exit)
    }
}
