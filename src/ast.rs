use std::process::{Command as ProcessCommand, ExitStatus};

use failure::*;

#[derive(PartialEq, Eq, Debug)]
pub enum AST {
    Comment(String),
    Command(Command),
    If(Conditional),
}

impl AST {
    pub fn execute(self) -> Result<Option<ExitStatus>, Error> {
        match self {
            AST::Comment(_) => Ok(None),
            AST::Command(c) => c.execute().map(Some),
            AST::If(c) => c.execute(),
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

#[derive(Debug, PartialEq, Eq)]
pub struct Conditional {
    predicate: Command,
    if_block: Block,
    else_block: Option<Block>,
}

impl Conditional {
    pub fn new(
        predicate: Command,
        if_block: Vec<AST>,
        else_block: Option<Vec<AST>>,
    ) -> Conditional {
        Conditional {
            predicate,
            if_block: Block(if_block),
            else_block: else_block.map(Block),
        }
    }

    fn execute(self) -> Result<Option<ExitStatus>, Error> {
        let maybe_exit = self.predicate.execute()?;
        if maybe_exit.success() {
            self.if_block.execute()
        } else {
            match self.else_block {
                None => Ok(None),
                Some(b) => b.execute(),
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Block(Vec<AST>);

impl Block {
    fn execute(self) -> Result<Option<ExitStatus>, Error> {
        let mut last_exit = None;
        for ast in self.0 {
            last_exit = ast.execute()?.or(last_exit);
            if let Some(e) = last_exit {
                if !e.success() {
                    break;
                }
            }
        }
        Ok(last_exit)
    }
}
