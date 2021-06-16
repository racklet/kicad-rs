use serde::{Deserialize, Serialize};

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub attributes: Vec<Attribute>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Attribute {
    pub name: String,
    pub value: String,
    pub expression: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub comment: Option<String>,
}
