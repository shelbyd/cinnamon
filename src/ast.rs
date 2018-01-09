use std::process::Command as ProcessCommand;

#[derive(PartialEq, Eq, Debug)]
pub enum AST {
    Comment(String),
    Command(Command),
}

impl AST {
    pub fn execute(self) {
        match self {
            AST::Comment(_) => {}
            AST::Command(c) => c.execute(),
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
    fn execute(self) {
        ProcessCommand::new(self.command)
            .args(self.args)
            .spawn()
            .expect("could not spawn child process")
            .wait()
            .expect("could not wait for child process");
    }
}
