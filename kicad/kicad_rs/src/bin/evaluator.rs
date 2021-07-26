use clap::{App, Arg};
use kicad_rs::error::DynamicResult;
use kicad_rs::eval;
use kicad_rs::parser::SchematicTree;

// Get crate version information from Cargo
const VERSION: Option<&'static str> = option_env!("CARGO_PKG_VERSION");

// Main function, can return different kinds of errors
fn main() -> DynamicResult<()> {
    let matches = App::new("KiCad evaluator")
        .about("Evaluates expressions in KiCad Eeschema schematics")
        .author("Dennis Marttinen (@twelho), The Racklet Project")
        .version(VERSION.unwrap_or("unknown"))
        .version_short("v")
        .arg(
            Arg::with_name("SCHEMATIC")
                .help("Path to the schematic file to process")
                .required(true),
        )
        .get_matches();

    // Calling .unwrap() is safe here because "SCHEMATIC" is required (if "SCHEMATIC"
    // wasn't required we could have used an 'if let' to conditionally get the value)
    let path = std::path::Path::new(matches.value_of("SCHEMATIC").unwrap());

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
