use std::error::Error;
use std::fmt;

// A result that can carry any Error implementation
pub type DynamicResult<T> = Result<T, Box<dyn Error>>;

// A struct implementing the Error trait, carrying just a simple message
#[derive(Debug)]
pub struct StringError {
    str: String,
}

pub fn errorf(s: &str) -> Box<dyn Error> {
    Box::new(StringError { str: s.into() })
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
