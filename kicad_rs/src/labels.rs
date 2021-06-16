use std::collections::HashMap;

// Labels is a trait describing a string-string of labels describing some object
pub trait Labels {
    fn get_label(&self, key: &str) -> Option<&str>;
}

// LabelsMatch is a trait that allows deciding whether a given requirement matches
// the set of labels given
pub trait LabelsMatch {
    fn matches<L: Labels>(&self, labels: &L) -> bool;
}

// Implement the Labels trait for a string-string HashMap
impl Labels for HashMap<&str, &str> {
    fn get_label(&self, key: &str) -> Option<&str> {
        self.get(key).map(|s| s.to_owned())
    }
}
