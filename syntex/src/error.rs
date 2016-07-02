use std::error;
use std::fmt;
use std::io;

use errors::DiagnosticBuilder;

#[derive(Debug)]
pub enum Error {
    Parse,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Parse => write!(f, "failed to parse input"),
            Error::Io(ref err) => err.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn description(&self) -> &str {
        match *self {
            Error::Parse => "failed to parse input",
            Error::Io(ref err) => err.description(),
        }
    }
}

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}

impl<'a> From<DiagnosticBuilder<'a>> for Error {
    fn from(mut diagnostic: DiagnosticBuilder<'a>) -> Self {
        diagnostic.emit();
        Error::Parse
    }
}
