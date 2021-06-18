use std::collections::HashMap;
use std::error::Error;
use std::path::Path;
use std::{env, iter};

use evalexpr::Context;

use kicad_rs::resolver;
use kicad_rs::resolver::*;

use kicad_rs::error::{errorf, DynamicResult};
use kicad_rs::parser;
use kicad_rs::types::{Attribute, Schematic, Component};
use std::iter::Chain;
use kicad_rs::parser::{SchematicFile, parse_schematic};
use kicad_parse_gen::footprint::Element::Descr;
use std::cell::RefCell;

// Main function, can return different kinds of errors
fn main() -> Result<(), Box<dyn Error>> {
    let mut args: Vec<String> = env::args().collect();
    let path = std::path::Path::new(args.get(1).ok_or("expected file as first argument")?);

    let mut file = SchematicFile::load(path)?;
    let mut schematic = parse_schematic(&file, String::new())?;

    // evaluate_schematic(&mut schematic)?;
    // print!("{}", updated);

    // let mut input = HashMap::new();
    // input.insert("a", "5");
    // input.insert("a", "5");
    // input.insert("d", "b * 2");
    // input.insert("b", "a + c");
    // input.insert("c", "6");
    //
    // let mut expr = HashMap::<String, Expression>::new();
    // for (k, v) in input {
    //     expr.insert(String::from(k), Expression::new(v.into(), String::new()));
    // }
    //
    // let c = resolver::resolve(&expr);
    // println!("{:?}", c.get_value("d"));

    Ok(())
}

fn evaluate_schematic(sch: &mut Schematic) -> Result<(), Box<dyn Error>> {
        // let attributes = components.iter().flat_map(|c| c.attributes.iter());

        // Add all component attributes to the lookup HashMap, duplicates will error
        // let mut lookup = HashMap::new();

        let mut index: IndexMap = Default::default();

        let index = index_schematic(sch)?;

        // for c in &sch.components{
        //     for a in &c.attributes {
        //         let id = create_identifier(&c.reference, &a.name);
        //         if lookup.contains_key(&id) {
        //             return Err(errorf(&format!(
        //                 "duplicate attribute definition in components: {}",
        //                 a.name
        //             ))));
        //         }
        //
        //         lookup.insert(id, Expression::new(&a, String::new()));
        //     }
        // }

        // schematic.components()[0].update_field("test", "result");

        // for a in attributes {
        //     println!("{:#?}", a);
        //     let name = a.name.to_owned();
        //     if expressions.contains_key(&name) {
        //         return Err(errorf(&format!(
        //             "duplicate attribute definition in components: {}",
        //             a.name
        //         ))));
        //     }
        //
        //     expressions.insert(name, Expression::new(&a, String::new()));
        // }

        // Add all global attributes to the lookup HashMap, here duplicates
        // are fine since we permit overriding component attributes
        // for a in &globals {
        //     lookup.insert(a.name.to_owned(), Expression::new(&a, String::new()));
        // }
        //
        // // Resolve the values
        // let c = resolver::resolve(&lookup);
        //
        // // TODO: Enable
        // for id in lookup.keys() {
        //     println!("{}: {}", id, c.get_value(id).unwrap());
        // }

        // TODO: Make attributes have `&mut value` so it can be updated and serialized again

        // Collect all attributes of all components in a single iterator

        // Iterate those attributes and insert them into the hashmap, duplicates error

        // Insert the global attributes into the hashmap, duplicates are fine (overriding)

        // Do the evaluation

        // let mut input = HashMap::new();
        // input.insert("a", "5");
        // input.insert("d", "b * 2");
        // input.insert("b", "a + c");
        // input.insert("c", "6");
        //
        // let mut expr = HashMap::<String, Expression>::new();
        // for (k, v) in input {
        //     expr.insert(String::from(k), Expression::new(v.into(), String::new()));
        // }
        //
        // let c = resolver::resolve(&expr);
        // println!("{:?}", c.get_value("d"));
    Ok(())
}

fn index_schematic(sch: &mut Schematic) -> DynamicResult<IndexMap> {
    let mut index = IndexMap::new();

    for c in sch.components.iter_mut() {
        let mut attribute_map = IndexMap::new();
        for a in c.attributes.iter_mut() {
            if attribute_map.tree.contains_key(&a.name) {
                return Err(errorf(&format!(
                    "duplicate attribute definition: {}",
                    a.name
                )));
            }
            attribute_map.tree.insert(a.name.clone(), Node::Leaf(RefCell::new(a)));
        }
        index.tree.insert(c.labels.reference.clone(), Node::Branch(attribute_map));
    }

    for sub_sch in sch.sub_schematics.iter_mut() {
        if let Some(_) = index.tree.insert(sub_sch.id.clone(), Node::Branch(index_schematic(sub_sch)?)) {
            return Err(errorf(&format!("component and schematic name collision: {}", "sub_sch.id"))); // TODO: This
        }
    }

    Ok(index)
}
