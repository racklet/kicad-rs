use kicad_rs::codec;
use kicad_rs::error::DynamicResult;
use kicad_rs::policy;
use kicad_rs::types::Schematic;
use std::env;
use std::io;
use std::path::Path;

fn main() -> DynamicResult<()> {
    // Read the Schematic YAML from stdin
    let sch: Schematic = codec::unmarshal_yaml(io::stdin())?;

    // Read the first argument as the path to the policy file
    let args: Vec<String> = env::args().collect();
    let p = Path::new(
        args.get(1)
            .ok_or("expected policy file as first argument")?,
    );

    // Apply the policy in the given file
    let processed_sch = policy::apply(&p, sch)?;

    // Marshal the resulting schematic as YAML
    codec::marshal_yaml(&processed_sch, io::stdout())?;
    Ok(())
}
