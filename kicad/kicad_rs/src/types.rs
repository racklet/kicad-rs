use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::iter::FromIterator;

use crate::labels::Labels;

// These types are used to structure the YAML-formatted output

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Schematic {
    // The "top-level" schematic has id ""
    pub id: String,
    pub meta: SchematicMeta,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub globals: Vec<Attribute>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub components: Vec<Component>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub sub_schematics: Vec<Schematic>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct SchematicMeta {
    pub file_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub company: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub comments: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub labels: ComponentLabels,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub classes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ComponentLabels {
    pub reference: String,
    pub footprint_name: String,
    pub footprint_library: String,
    pub symbol_name: String,
    pub symbol_library: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub datasheet: Option<String>,
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub extra: HashMap<String, String>,
}

impl ComponentLabels {
    pub fn to_map(&self) -> HashMap<&str, &str> {
        let mut m = HashMap::from_iter(self.extra.iter().map(|s| (s.0.as_str(), s.1.as_str())));
        m.insert("reference", self.reference.as_str());
        m.insert("footprintLibrary", self.footprint_library.as_str());
        m.insert("footprintName", self.footprint_name.as_str());
        m.insert("symbolLibrary", self.symbol_library.as_str());
        m.insert("symbolName", self.symbol_name.as_str());
        m
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    pub name: String,
    #[serde(flatten)]
    pub value: Value,
    pub expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comment: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type", content = "value")]
pub enum Value {
    String(String),
    Float(f64),
}

impl Value {
    pub fn parse(s: String) -> Value {
        if let Ok(f) = s.parse::<f64>() {
            f.into()
        } else {
            s.into()
        }
    }
}

impl ToString for Value {
    fn to_string(&self) -> String {
        match self {
            Value::String(s) => s.clone(),
            Value::Float(f) => f.to_string(),
        }
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.into())
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

// A vector of Attributes implements the Labels trait
impl Labels for Vec<Attribute> {
    fn get_label(&self, key: &str) -> Option<String> {
        self.iter()
            .find(|&a| a.name == key)
            .map(|a| a.value.to_string())
    }
}
