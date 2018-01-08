#[derive(PartialEq, Eq, Debug)]
pub enum AST {
    Comment(String),
    Command(Command),
}

#[derive(PartialEq, Eq, Debug)]
pub struct Command(pub String);
