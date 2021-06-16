use serde::de::DeserializeOwned;
use serde::Serialize;
use std::io;

use crate::error::DynamicResult;

// marshal_yaml marshals a serializable value to a YAML string using Serde, but
// avoiding https://github.com/dtolnay/serde-yaml/issues/87.
pub fn marshal_yaml<T, W>(data: T, writer: W) -> DynamicResult<()>
where
    T: Serialize,
    W: io::Write,
{
    // First use the JSON bin.parser to convert the struct into an "intermediate representation": a Serde::Value
    let json_val = serde_json::to_value(data)?;
    // Then, marshal the intermediate representation to YAML, avoiding errors like https://github.com/dtolnay/serde-yaml/issues/87
    serde_yaml::to_writer(writer, &json_val)?;
    // All ok
    Ok(())
}

// unmarshal_yaml is the reverse operation of marshal_yaml.
pub fn unmarshal_yaml<R, T>(reader: R) -> serde_yaml::Result<T>
where
    R: io::Read,
    T: DeserializeOwned,
{
    serde_yaml::from_reader(reader)
}
