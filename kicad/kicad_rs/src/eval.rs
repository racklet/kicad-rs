mod entry;
mod index;
mod path;

use crate::error::{errorf, DynamicResult};
use crate::eval::index::{ComponentIndex, Node, SheetIndex};
use crate::eval::path::Path;
use crate::types::Schematic;

pub fn index_schematic(sch: &mut Schematic) -> DynamicResult<SheetIndex> {
    // TODO: Revisit the SheetIndex idea now that Schematic contains HashMaps?
    let mut index = SheetIndex::new();

    for (_, c) in sch.components.iter_mut() {
        let mut component_idx = ComponentIndex::new();
        for (attr_name, a) in c.attributes.iter_mut() {
            if component_idx.contains_key(attr_name) {
                return Err(errorf(&format!(
                    "duplicate attribute definition: {}",
                    attr_name
                )));
            }
            component_idx.insert(attr_name.into(), (attr_name, a).into());
        }
        index
            .map
            .insert(c.labels.reference.clone(), Node::Component(component_idx));
    }

    for (sch_id, sub_sch) in sch.sub_schematics.iter_mut() {
        if index.map.contains_key(sch_id) {
            return Err(errorf(&format!(
                "component and schematic name collision: {}",
                sch_id
            )));
        }
        index
            .map
            .insert(sch_id.into(), Node::Sheet(index_schematic(sub_sch)?));
    }

    Ok(index)
}

pub fn evaluate_schematic(index: &mut SheetIndex) -> DynamicResult<()> {
    // Perform resolving recursively in depth-first order
    for node in index.map.values_mut() {
        if let Node::Sheet(sub_index) = node {
            evaluate_schematic(sub_index)?;
        }
    }

    // Collect all attributes for all components
    let mut paths = Vec::new();
    for (node_ref, node) in index.map.iter() {
        if let Node::Component(component_index) = node {
            for a in component_index.keys() {
                paths.push(vec![node_ref.into(), a.into()].into())
            }
        }
    }

    // Evaluate all the collected attributes
    for path in paths.iter() {
        evaluate(index, path)?;
    }

    Ok(())
}

fn evaluate(idx: &mut SheetIndex, p: &Path) -> DynamicResult<()> {
    // TODO: Support u, k, M, G, etc. suffixes. Now the evaluator treats them as a
    // variable.
    // TODO: Support case-insensitive referencing of attributes (e.g. C3.Value == C3.value)?
    // TODO: Decide whether we should write out the unit too in the value or not, e.g.
    // "35" vs "35 F". "35 F" looks nicer in KiCad, but also might mess up the parsing unless
    // we have a well-known "undo" method like stripping the " {}" suffix where {} is the unit
    // before parsing the rest of the string into a float or string.
    // TODO: An expression like "R7.Value/500" will perform integer division if both expressions
    // resolve to an integer, which is something to be aware of. Additionally, it seems like putting
    // just "500.0" in an expression resolves to "500" in the output, something which might be
    // desired, but just worth documenting.
    let entry = idx
        .resolve_entry(p.iter())
        .ok_or(errorf(&format!("entry not found: {}", p)))?;

    if entry.value_defined()? {
        return Ok(()); // Don't update if already set
    }

    let node = evalexpr::build_operator_tree(entry.get_expression())?;
    for dep in node.iter_variable_identifiers().map(|id| id.into()) {
        evaluate(idx, &dep)?;
    }

    let value = node.eval_with_context(idx)?;
    idx.update_entry(p.iter(), value)?;

    Ok(())
}
