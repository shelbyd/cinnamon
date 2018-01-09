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
pub struct Command(pub String);

impl Command {
    fn execute(self) {
        ProcessCommand::new(self.0)
            .spawn()
            .expect("could not spawn child process")
            .wait()
            .expect("could not wait for child process");
    }
}
