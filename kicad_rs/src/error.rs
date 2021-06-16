use std::error::Error;
use std::fmt;

// A struct implementing the Error trait, carrying just a simple message
#[derive(Debug)]
pub struct StringError {
    str: String,
}

pub fn errorf(s: &str) -> StringError {
    StringError { str: s.into() }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}

impl Error for StringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
