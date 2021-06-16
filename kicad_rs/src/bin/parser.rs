use kicad_rs::codec;
use kicad_rs::error::DynamicResult;
use kicad_rs::types::*;
use std::env;
use std::io;
use std::path::Path;

// Main function, can return different kinds of errors
fn main() -> DynamicResult<()> {
    // Read the first argument as the path to the .sch file
    let args: Vec<String> = env::args().collect();
    let p = Path::new(
        args.get(1)
            .ok_or("expected KiCad schematic file as first argument")?,
    );

    // Parse the schematic file
    let sch = Schematic::parse(&p)?;

    // Marshal as YAML
    codec::marshal_yaml(&sch, io::stdout())?;
    Ok(())
}
