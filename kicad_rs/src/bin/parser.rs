use std::error::Error;
use std::path::Path;
use std::env;
use kicad_rs::types::*;

// Main function, can return different kinds of errors
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let p = Path::new(args.get(1).ok_or("expected file as first argument")?);
    let sch = Schematic::parse(&p)?;

    // Marshal as YAML
    // First use the JSON bin.parser to convert the struct into an "intermediate representation": a Serde::Value
    let json_val = serde_json::to_value(&sch)?;
    // Then, marshal the intermediate representation to YAML, avoiding errors like https://github.com/dtolnay/serde-yaml/issues/87
    let serialized = serde_yaml::to_string(&json_val)?;
    println!("{}", serialized);

    Ok(())
}
