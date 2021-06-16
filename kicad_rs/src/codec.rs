use serde::Serialize;
use std::error::Error;

// marshal_yaml marshals a serializable value to a YAML string using Serde, but
// avoiding https://github.com/dtolnay/serde-yaml/issues/87.
pub fn marshal_yaml<T: Serialize>(data: T) -> Result<String, Box<dyn Error>> {
    // First use the JSON bin.parser to convert the struct into an "intermediate representation": a Serde::Value
    let json_val = serde_json::to_value(data)?;
    // Then, marshal the intermediate representation to YAML, avoiding errors like https://github.com/dtolnay/serde-yaml/issues/87
    let serialized = serde_yaml::to_string(&json_val)?;
    // Return the serialized string
    Ok(serialized)
}