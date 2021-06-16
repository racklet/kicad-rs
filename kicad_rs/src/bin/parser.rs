use kicad_rs::codec;
use kicad_rs::types::*;
use std::env;
use std::error::Error;
use std::path::Path;

// Main function, can return different kinds of errors
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let p = Path::new(args.get(1).ok_or("expected file as first argument")?);
    let sch = Schematic::parse(&p)?;

    // Marshal as YAML
    let serialized = codec::marshal_yaml(&sch)?;
    println!("{}", serialized);

    Ok(())
}
