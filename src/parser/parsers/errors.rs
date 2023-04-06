use crate::Location;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind<'filedata> {
    ExpectedTag(&'static str),
    ExpectedKind(&'static str),
    ExpectedOneOfKind(&'static str),
    ExpectedOneOf(&'static str),
    InverseFailedGot(&'filedata str),
    DemoError
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsingError<'filedata>(pub Location<'filedata>, pub ErrorKind<'filedata>);
