use nom::*;

#[derive(Debug)]
pub enum ParseTree {
    Null,
}

pub trait Parse: Sized {
    fn parse(self) -> Result<ParseTree, IError>;
}

impl Parse for Vec<u8> {
    fn parse(self) -> Result<ParseTree, IError> {
        Ok(ParseTree::Null)
    }
}
