use kicad_rs::error::DynamicResult;
use kicad_rs::eval;
use kicad_rs::parser::{parse_schematic, SchematicFile};
use std::env;

// Main function, can return different kinds of errors
fn main() -> DynamicResult<()> {
    let args: Vec<String> = env::args().collect();
    let path = std::path::Path::new(args.get(1).ok_or("expected file as first argument")?);

    // Load the schematic file and parse it
    let file = SchematicFile::load(path)?;
    let mut schematic = parse_schematic(&file, String::new())?;

    // Index the parsed schematic and use the index to evaluate it
    let mut index = eval::index_schematic(&mut schematic)?;
    eval::evaluate_schematic(&mut index)?;

    // TODO: Apply the internal schematic back to kicad_parse_gen::schematic::Schematic and print
    println!("{:#?}", index);

    Ok(())
}
