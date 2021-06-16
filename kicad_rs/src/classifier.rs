use crate::labels::LabelsMatch;
use crate::requirements::Requirement;
use crate::error::DynamicResult;
use crate::types::{Component, Schematic};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;
use std::iter::FromIterator;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Policy {
    classifiers: Vec<ComponentClassifier>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ComponentClassifier {
    // The class that shall be applied to a component matching these requirements
    pub class: String,
    // Labels matching
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub labels: Vec<Requirement>,
    // Attribute matching
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub attributes: Vec<Requirement>,
}

impl Policy {
    // apply applies the policy on a given Schematic
    pub fn apply(&self, sch: Schematic) -> DynamicResult<Schematic> {
        let mut sch = sch;
        classify_components(&mut sch, &self.classifiers);
        Ok(sch)
    }
}

// classify_components recursively walks through a Schematic, and assigns the Component.classes field
fn classify_components(sch: &mut Schematic, classifiers: &Vec<ComponentClassifier>) {
    for comp in sch.components.iter_mut() {
        comp.classes = classify_component(comp, classifiers);
    }
    for sch in sch.sub_schematics.iter_mut() {
        classify_components(sch, classifiers);
    }
}

// classify_component returns a list of classes for a given component, given the set of classifiers
fn classify_component(comp: &Component, classifiers: &Vec<ComponentClassifier>) -> Vec<String> {
    // Map all classifiers to their name if the component matches the classifier
    let matched_classes: Vec<String> = classifiers
        .iter()
        .filter_map(|classifier| {
            // Require that both all labels and attribute requirements match
            if !classifier.labels.matches(&comp.labels.to_map()) {
                return None;
            }
            if !classifier.attributes.matches(&comp.attributes) {
                return None;
            }

            // If we get all the way here, we have "matched" with this class.
            return Some(classifier.class.clone());
        })
        .collect();

    // As there might be many classifiers of the same name that have matched with a component,
    // filter all duplicates
    filter_duplicates(&matched_classes)
}

// filter_duplicates inserts all items into a HashSet, and builds a new vector without any duplicates
fn filter_duplicates<T: Eq + Clone + Hash>(list: &Vec<T>) -> Vec<T> {
    let f: HashSet<&T> = HashSet::from_iter(list.iter());
    f.iter().map(|s| s.clone().to_owned()).collect()
}
