use kicad_rs::error::DynamicResult;
use kicad_rs::eval;
use kicad_rs::parser::SchematicTree;
use std::env;

// Main function, can return different kinds of errors
fn main() -> DynamicResult<()> {
    let args: Vec<String> = env::args().collect();
    let path = std::path::Path::new(args.get(1).ok_or("expected file as first argument")?);

    // Load the hierarchical schematic tree and parse it
    let mut tree = SchematicTree::load(path)?;
    let mut schematic = tree.parse()?;

    // Index the parsed schematic and use the index to evaluate it. The
    // index links to the schematic using mutable references, so that's
    // why the schematic itself needs to be passed in as mutable here.
    let mut index = eval::index_schematic(&mut schematic)?;
    eval::evaluate_schematic(&mut index)?;

    // Update the fields of the components in the schematic tree based
    // on the newly computed values and write the updated schematics
    // back into the respective files
    tree.update(&schematic)?;
    tree.write()?;

    Ok(())
}
