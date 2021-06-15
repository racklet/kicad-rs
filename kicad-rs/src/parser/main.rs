// Import types.rs
mod types;
use types::*;

use kicad_parse_gen::schematic as kicad_schematic;
use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{env, fmt};

// Main function, can return different kinds of errors
fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    let p = std::path::Path::new(args.get(1).ok_or("expected file as first argument")?);
    let sch = parse_schematic(&p, String::new())?;

    // Marshal as YAML
    // First use the JSON parser to convert the struct into an "intermediate representation": a Serde::Value
    let json_val = serde_json::to_value(&sch)?;
    // Then, marshal the intermediate representation to YAML, avoiding errors like https://github.com/dtolnay/serde-yaml/issues/87
    let serialized = serde_yaml::to_string(&json_val)?;
    println!("{}", serialized);

    Ok(())
}

// parse_schematic turns a schematic at path p into the recursive Schematic struct.
fn parse_schematic(p: &Path, id: String) -> Result<Schematic, Box<dyn Error>> {
    // Read the schematic using kicad_parse_gen
    let kisch = kicad_parse_gen::read_schematic(p)?;

    // Only include non-empty comments
    let comments = vec![
        &kisch.description.comment1,
        &kisch.description.comment2,
        &kisch.description.comment3,
        &kisch.description.comment4,
    ]
    .iter()
    .flat_map(|c| str_if_nonempty(c))
    .collect();

    // Build the metadata for this schematic, and instantiate empty vectors to be filled in
    let mut sch = Schematic {
        id: id,
        meta: SchematicMeta {
            file_name: str_borrow_unwrap(p.file_name().map(|s| s.to_str()).flatten()),
            title: str_if_nonempty(&kisch.description.title),
            date: str_if_nonempty(&kisch.description.date),
            revision: str_if_nonempty(&kisch.description.rev),
            company: str_if_nonempty(&kisch.description.comp),
            comments,
        },
        globals: vec![],
        components: vec![],
        sub_schematics: vec![],
    };

    // Walk through all components in the sheet
    for comp in kisch.components() {
        // Require comp.name to be non-empty
        if comp.name.is_empty() {
            return Err(Box::new(errorf("Every component must have a name")));
        }

        // Fill in the metadata about the component. Reference and package fields are validated to be non-empty
        // later, once we know if the component should be included in the result.
        let mut c = Component {
            reference: comp.reference.clone(),
            package: str_unwrap(
                get_component_attr(&comp, "Footprint")
                    .map(|s| s.split_once(":").map(|strs| strs.0.to_owned()))
                    .flatten(),
            ),
            category: comp.name.to_owned(),
            model: get_component_attr(&comp, "Model"),
            datasheet: get_component_attr(&comp, "UserDocLink"),
            attributes: vec![],
        };

        // m maps the lower-case representation to the whatever-cased representation
        let mut m = HashMap::new();
        // Walk through all the fields, and fill in the m map
        for f in &comp.fields {
            // Optimistically try to insert key_lower into m, and error if there was a duplicate
            let key_lower = f.name.to_lowercase().clone();
            match m.insert(key_lower, f.name.clone()) {
                None => (), // Key didn't exist before, all ok
                Some(oldval) => {
                    return Err(Box::new(errorf(&format!(
                        "duplicate keys: {} and {}",
                        oldval, f.name
                    ))))
                }
            }
        }

        // Walk through the attributes, and look for one that ends with _expr or _expression
        for f in &comp.fields {
            let fname = f.name.to_lowercase();
            // Strip the expr suffixes from the lower-cased fname, or skip it if the suffix isn't correct
            let main_key = if fname.ends_with("_expr") {
                fname.trim_end_matches("_expr")
            } else if fname.ends_with("_expression") {
                fname.trim_end_matches("_expression")
            } else {
                continue;
            };

            // The unit value can be found from the main key + the "_unit" suffix
            let unit_key = main_key.to_owned() + "_unit";

            // Create a new attribute with the given parameters
            c.attributes.push(Attribute {
                // Special case: if the main key is "value", it is the default attribute, and hence name can be ""
                name: if main_key == "value" {
                    String::new()
                } else {
                    main_key.to_owned()
                },
                // Get the main key value. It is ok if it's empty, too.
                value: str_unwrap(get_component_attr_mapped(&comp, main_key, &m)),
                // As this field corresponds to the main key expression attribute, we can get the expression directly
                expression: f.value.clone(),
                // Optionally, get the unit
                unit: get_component_attr_mapped(&comp, &unit_key, &m),
            });
        }

        // Only register to the list if it has any expressions, or if it has iccc_show = true set
        if c.attributes.len() > 0
            || is_true_str(&str_unwrap(get_component_attr_mapped(
                &comp,
                "iccc_show",
                &m,
            )))
        {
            // Validate that reference and package aren't empty
            if c.reference.is_empty() {
                return Err(Box::new(errorf(&format!(
                    "{}: Component.reference is a mandatory field",
                    &comp.name
                ))));
            }
            if c.package.is_empty() {
                return Err(Box::new(errorf(&format!(
                    "{}: Component.package is a mandatory field",
                    &comp.name
                ))));
            }
            // Grow the components vector
            sch.components.push(c);
        }
    }

    parse_globals_into(&kisch, &mut sch.globals);

    // Recursively traverse and parse the sub-schematics
    for sub_sheet in &kisch.sheets {
        // TODO: Use absolute paths, relative to the current schematic
        let p = Path::new(&sub_sheet.filename);
        sch.sub_schematics
            .push(parse_schematic(p, sub_sheet.name.clone())?);
    }

    // Finally, return the parsed schematic
    Ok(sch)
}

// parse_globals_into parses text notes from the schematic into globals
fn parse_globals_into(kisch: &kicad_schematic::Schematic, globals: &mut Vec<Attribute>) {
    // Loop through the elements of the schematic, which includes text notes as well
    for el in &kisch.elements {
        // Only match Text elements that have type Note
        let text_element = match el {
            kicad_schematic::Element::Text(t) => match t.t {
                kicad_schematic::TextType::Note => t,
                _ => continue,
            },
            _ => continue,
        };

        // The text element contains literal "\n" elements
        for line in text_element.text.split("\\n") {
            // Format: Foo[.Bar.Baz..] = <expr> [; <unit>]

            // First, split by the equals sign. If the equals sign does not exist,
            // just continue.
            let (attr_name, expr) = match line.split_once("=") {
                None => continue,
                Some(a) => a,
            };

            // Then, split the "remaining" part expr into two parts by ";", where
            // the first part overwrites expr, and the other part optionally becomes unit
            let (expr, unit) = match expr.split_once(";") {
                None => (expr, None),
                Some(a) => (a.0, Some(a.1)),
            };

            // Trim whitespace for all variables
            let (attr_name, expr) = (attr_name.trim(), expr.trim());
            let unit = unit.map(|u| u.trim().to_owned());

            // attr_name and expr must be non-empty
            if attr_name.is_empty() || expr.is_empty() {
                continue;
            }

            // Push the new attribute into the given vector
            globals.push(Attribute {
                name: attr_name.to_owned(),
                value: String::new(),
                expression: expr.to_owned(),
                unit,
            })
        }
    }
}

// is_true_str returns true is s is a "true-like" string like "true" or "1", otherwise false
fn is_true_str(s: &str) -> bool {
    s == "true" || s == "1"
}

// str_unwrap unwraps the String option such that if opt is None, a new, empty String is returned
fn str_unwrap(opt: Option<String>) -> String {
    opt.unwrap_or_else(|| String::new())
}

// str_borrow_unwrap is like str_unwrap, but for an Option containing a string reference
fn str_borrow_unwrap(opt: Option<&str>) -> String {
    opt.unwrap_or("").to_owned()
}

// get_component_attr gets the component attribute value for a case-sensitive key, but returns
// None if the value is "" or "~"
fn get_component_attr(comp: &kicad_schematic::Component, key: &str) -> Option<String> {
    str_if_nonempty_opt(comp.get_field_value(key))
}

// get_component_attr_mapped works like get_component_attr, but allows "key" to be case-insensitive, as long as
// "key" exists in hashmap "m" which maps the case-insensitive key to a case-sensitive key that can be used for
// get_component_attr
fn get_component_attr_mapped(
    comp: &kicad_schematic::Component,
    key: &str,
    m: &HashMap<String, String>,
) -> Option<String> {
    m.get(key)
        .map(|attr_key| get_component_attr(comp, attr_key))
        .flatten()
}

// str_if_nonempty trims the string reference s, and returns it as owned in an Option if non-empty.
// As a special case, "~" also counts as "empty".
fn str_if_nonempty(s: &str) -> Option<String> {
    let s = s.trim();
    if s.is_empty() || s == "~" {
        None
    } else {
        Some(String::from(s))
    }
}

// str_if_nonempty_opt passes the string through str_if_nonempty if the option is Some. In other words,
// this function maps Some("") and Some("~") -> None, and lets all other values be.
fn str_if_nonempty_opt(s_opt: Option<String>) -> Option<String> {
    s_opt.map(|s| str_if_nonempty(&s)).flatten()
}

// A struct implementing the Error trait, carrying just a simple message
#[derive(Debug)]
struct StringError {
    str: String,
}

fn errorf(s: &str) -> StringError {
    StringError {
        str: String::from(s),
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.str)
    }
}
impl Error for StringError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}
