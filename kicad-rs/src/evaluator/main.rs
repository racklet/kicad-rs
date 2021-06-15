use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::path::Path;

mod resolver;
use evalexpr::Context;
use resolver::Expression;

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

        // m maps the lower-case representation to the whatever-cased representation
        // let mut m = HashMap::new();
        // Walk through all the fields, and fill in the m map
        for f in comp.fields.iter().filter(|&f| is_expression(&f.name)) {
            println!("{}: {}", comp.reference, f.name);

            // Optimistically try to insert key_lower into m, and error if there was a duplicate
            // let key_lower = f.name.to_lowercase().clone();
            // match m.insert(key_lower, f.name.clone()) {
            // 	None => (), // Key didn't exist before, all ok
            // 	Some(oldval) => {
            // 		return Err(Box::new(errorf(&format!(
            // 			"duplicate keys: {} and {}",
            // 			oldval, f.name
            // 		))))
            // 	}
            // }
        }

        // Walk through the attributes, and look for one that ends with _expr or _expression
        // for f in &comp.fields {
        // 	let fname = f.name.to_lowercase();
        // 	// Strip the expr suffixes from the lower-cased fname, or skip it if the suffix isn't correct
        // 	let main_key = if fname.ends_with("_expr") {
        // 		fname.trim_end_matches("_expr")
        // 	} else if fname.ends_with("_expression") {
        // 		fname.trim_end_matches("_expression")
        // 	} else {
        // 		continue;
        // 	};
        //
        // 	// The unit value can be found from the main key + the "_unit" suffix
        // 	let unit_key = main_key.to_owned() + "_unit";
        //
        // 	// Create a new attribute with the given parameters
        // 	c.attributes.push(Attribute {
        // 		// Special case: if the main key is "value", it is the default attribute, and hence name can be ""
        // 		name: if main_key == "value" {
        // 			String::new()
        // 		} else {
        // 			main_key.to_owned()
        // 		},
        // 		// Get the main key value. It is ok if it's empty, too.
        // 		value: str_unwrap(get_component_attr_mapped(&comp, main_key, &m)),
        // 		// As this field corresponds to the main key expression attribute, we can get the expression directly
        // 		expression: f.value.clone(),
        // 		// Optionally, get the unit
        // 		unit: get_component_attr_mapped(&comp, &unit_key, &m),
        // 	});
        // }
        //
        // // Only register to the list if it has any expressions, or if it has iccc_show = true set
        // if c.attributes.len() > 0
        // 	|| is_true_str(&str_unwrap(get_component_attr_mapped(
        // 	&comp,
        // 	"iccc_show",
        // 	&m,
        // )))
        // {
        // 	// Validate that reference and package aren't empty
        // 	if c.reference.is_empty() {
        // 		return Err(Box::new(errorf(&format!(
        // 			"{}: Component.reference is a mandatory field",
        // 			&comp.name
        // 		))));
        // 	}
        // 	if c.package.is_empty() {
        // 		return Err(Box::new(errorf(&format!(
        // 			"{}: Component.package is a mandatory field",
        // 			&comp.name
        // 		))));
        // 	}
        // 	// Grow the components vector
        // 	sch.components.push(c);
        // }
    }

    Ok(schematic.to_string())
}

fn is_expression(s: &String) -> bool {
    s.ends_with("_expr") || s.ends_with("_expression")
}
