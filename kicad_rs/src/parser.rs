use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::fmt;

use kicad_parse_gen::schematic as kicad_schematic;

use crate::types::*;

impl Schematic {
    // type Result = Result<Schematic, Box<dyn Error>>;

    pub fn parse(p: &Path) -> Result<Schematic, Box<dyn Error>> {
        parse_schematic(p, String::new())
    }
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
    .flat_map(|c| c.as_str().filter_empty())
    .collect();

    // Build the metadata for this schematic, and instantiate empty vectors to be filled in
    let mut sch = Schematic {
        id: id,
        meta: SchematicMeta {
            file_name: p.file_name().map(|s| s.to_str()).flatten().or_empty_str(),
            title: kisch.description.title.as_str().filter_empty(),
            date: kisch.description.date.as_str().filter_empty(),
            revision: kisch.description.rev.as_str().filter_empty(),
            company: kisch.description.comp.as_str().filter_empty(),
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

        let footprint_str = get_component_attr(&comp, "Footprint");

        // Fill in the metadata about the component. Reference and package fields are validated to be non-empty
        // later, once we know if the component should be included in the result.
        let mut c = Component {
            reference: comp.reference.clone(),
            package: footprint_str.split_char_n(':', 0).or_empty_str(),
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
                value: get_component_attr_mapped(&comp, main_key, &m).or_empty_str(),
                // As this field corresponds to the main key expression attribute, we can get the expression directly
                expression: f.value.clone(),
                // Optionally, get the unit
                unit: get_component_attr_mapped(&comp, &unit_key, &m),
            });
        }

        // Only register to the list if it has any expressions, or if it has iccc_show = true set
        if c.attributes.len() > 0
            || get_component_attr_mapped(
                &comp,
                "iccc_show",
                &m,
            ).or_empty_str().is_true_like()
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

// get_component_attr gets the component attribute value for a case-sensitive key, but returns
// None if the value is "" or "~"
fn get_component_attr(comp: &kicad_schematic::Component, key: &str) -> Option<String> {
    comp.get_field_value(key).filter_empty()
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

// IsTrueLike provides an is_true_like that turns a type into a bool given type-specific heuristic
// rules.
trait IsTrueLike {
    fn is_true_like(&self) -> bool;
}

impl<T: AsRef<str>> IsTrueLike for T {
    // is_true_like returns true is s is a "true-like" string like "true" or "1", otherwise false
    fn is_true_like(&self) -> bool {
        self.as_ref() == "true" || self.as_ref() == "1"
    }
}

// OrEmptyStr provides an or_empty_str method that unwraps a string option in a safe way,
// defaulting to an empty string if None. The inner string is cloned.
trait OrEmptyStr {
    // or_empty_str unwraps the string-like option such that if opt is None, a new, empty String is returned
    fn or_empty_str(&self) -> String;
}

impl OrEmptyStr for Option<String> {
    fn or_empty_str(&self) -> String {
        self.clone().unwrap_or_else(|| String::new())
    }
}

impl OrEmptyStr for Option<&str> {
    fn or_empty_str(&self) -> String {
        self.unwrap_or("").to_owned()
    }
}

// EmptyFilter provides a filter_empty method that filters a string or string option into a
// string option, making sure "empty-like" strings make the option become None.
trait EmptyFilter {
    fn filter_empty(&self) -> Option<String>;
}

impl EmptyFilter for &str {
    // filter_empty trims the string reference s, and returns it as owned in an Option if non-empty.
    // As a special case, "~" also counts as "empty".
    fn filter_empty(&self) -> Option<String> {
        let s = self.trim();
        if s.is_empty() || s == "~" {
            None
        } else {
            Some(String::from(s))
        }
    }
}

impl<T: AsRef<str>> EmptyFilter for Option<T> {
    // filter_empty passes the string through &str.filter_empty if the option is Some. In other words,
    // this function maps Some("") and Some("~") -> None, and lets all other values be.
    fn filter_empty(&self) -> Option<String> {
        match self {
            Some(s) => s.as_ref().filter_empty(),
            None => None,
        }
    }
}

// SplitCharN provides a split_char_n method that can be used to split a string by a given
// character, and then return an option wrapping the n-th split match.
trait SplitCharN {
    fn split_char_n(&self, split_char: char, idx: usize) -> Option<String>;
}

impl SplitCharN for &str {
    fn split_char_n(&self, split_char: char, idx: usize) -> Option<String> {
        self.split(split_char).nth(idx).map(|s| s.to_owned())
    }
}

impl<T: AsRef<str>> SplitCharN for Option<T> {
    fn split_char_n(&self, split_char: char, idx: usize) -> Option<String> {
        match self {
            Some(s) => s.as_ref().split_char_n(split_char, idx),
            None => None,
        }
    }
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
