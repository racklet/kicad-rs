use std::fmt;
use std::slice::Iter;

const PATH_SEPARATOR: &str = ".";

#[derive(Debug)]
pub struct Path {
    components: Vec<String>,
}

impl Path {
    pub(crate) fn iter(&self) -> Iter<'_, String> {
        self.components.iter()
    }
}

impl From<String> for Path {
    fn from(s: String) -> Self {
        let components = s.split(PATH_SEPARATOR).map(|s| s.into()).collect();
        Self { components }
    }
}

impl From<&str> for Path {
    fn from(s: &str) -> Self {
        let components = s.split(PATH_SEPARATOR).map(|s| s.into()).collect();
        Self { components }
    }
}

impl From<Vec<String>> for Path {
    fn from(v: Vec<String>) -> Self {
        let components = v.into_iter().filter(|s| !s.is_empty()).collect();
        Self { components }
    }
}

impl fmt::Display for Path {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.components.join(PATH_SEPARATOR))
    }
}
