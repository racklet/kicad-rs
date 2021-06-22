use crate::labels::{Labels, LabelsMatch};
use serde::{Deserialize, Serialize};

// Requirement specifies a requirement for a Labels key-value set.
// The operator is the enum value, and the data to use for matching
// is in the struct.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
#[serde(tag = "op")]
pub enum Requirement {
    // At least one value
    In { key: String, values: Vec<String> },
    NotIn { key: String, values: Vec<String> },

    // One value
    Equals { key: String, values: [String; 1] },
    NotEquals { key: String, values: [String; 1] },

    // No values
    Exists { key: String },
    DoesNotExist { key: String },

    // One numeric value
    Gt { key: String, values: [f64; 1] },
    Lt { key: String, values: [f64; 1] },
}

// Implement the LabelsMatch trait for a vector of requirements ANDed together
impl LabelsMatch for Vec<Requirement> {
    fn matches<L: Labels>(&self, labels: &L) -> bool {
        self.iter().all(|r| r.matches(labels))
    }
}

// Implement the LabelsMatch trait for a single requirement
impl LabelsMatch for Requirement {
    fn matches<L: Labels>(&self, labels: &L) -> bool {
        match self {
            Requirement::In { key, values } => Requirement::match_in(labels, key, values),
            Requirement::NotIn { key, values } => !Requirement::match_in(labels, key, values),
            Requirement::Equals { key, values } => Requirement::match_in(labels, key, values),
            Requirement::NotEquals { key, values } => !Requirement::match_in(labels, key, values),
            Requirement::Exists { key } => labels.get_label(key).is_some(),
            Requirement::DoesNotExist { key } => labels.get_label(key).is_none(),
            Requirement::Gt { key, values } => {
                Requirement::match_numeric(labels, key, |n| n > values[0])
            }
            Requirement::Lt { key, values } => {
                Requirement::match_numeric(labels, key, |n| n < values[0])
            }
        }
    }
}

// Helper functions for the above matches function
impl Requirement {
    fn match_in<L>(labels: &L, key: &String, values: &[String]) -> bool
    where
        L: Labels,
    {
        labels
            .get_label(key)
            .map(|val| values.contains(&val.to_owned()))
            .unwrap_or(false)
    }
    fn match_numeric<L, P>(labels: &L, key: &String, p: P) -> bool
    where
        L: Labels,
        P: FnOnce(f64) -> bool,
    {
        labels
            .get_label(key)
            .map(|s| s.parse::<f64>().ok())
            .flatten()
            .map(|num| p(num))
            .unwrap_or(false)
    }
}
