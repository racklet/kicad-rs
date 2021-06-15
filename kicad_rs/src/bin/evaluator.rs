use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::Path;

use evalexpr::Context;

use kicad_rs::resolver;
use kicad_rs::resolver::*;

// Main function, can return different kinds of errors
fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    let p = std::path::Path::new(args.get(1).ok_or("expected file as first argument")?);
    let updated = evaluate_schematic(&p)?;
    // print!("{}", updated);

    let mut input = HashMap::new();
    input.insert("a", "5");
    input.insert("d", "b * 2");
    input.insert("b", "a + c");
    input.insert("c", "6");

    let mut expr = HashMap::<String, Expression>::new();
    for (k, v) in input {
        expr.insert(String::from(k), Expression::new(v.into(), String::new()));
    }

    let c = resolver::resolve(&expr);
    println!("{:?}", c.get_value("d"));

    Ok(())
}

fn evaluate_schematic(p: &Path) -> Result<String, Box<dyn Error>> {
    // Read the schematic using kicad_parse_gen
    let schematic = kicad_parse_gen::read_schematic(p)?;

    // Walk through all components in the sheet
    for comp in schematic.components() {
        // Require comp.name to be non-empty
        // if comp.name.is_empty() {
        // 	return Err(Box::new(errorf("Every component must have a name")));
        // }

        // Walk through all the fields
        for f in comp.fields.iter().filter(|&f| is_expression(&f.name)) {
            println!("{}: {}", comp.reference, f.name);
        }
    }

    Ok(schematic.to_string())
}

fn is_expression(s: &String) -> bool {
    s.ends_with("_expr") || s.ends_with("_expression")
}
