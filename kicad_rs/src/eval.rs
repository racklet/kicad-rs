mod entry;
mod index;
mod path;

use crate::error::{errorf, DynamicResult};
use crate::eval::index::{ComponentIndex, Node, SheetIndex};
use crate::eval::path::Path;
use crate::types::Schematic;

pub fn index_schematic(sch: &mut Schematic) -> DynamicResult<SheetIndex> {
    let mut index = SheetIndex::new();

    for c in sch.components.iter_mut() {
        let mut component_idx = ComponentIndex::new();
        for a in c.attributes.iter_mut() {
            if component_idx.contains_key(&a.name) {
                return Err(errorf(&format!(
                    "duplicate attribute definition: {}",
                    a.name
                )));
            }
            component_idx.insert(a.name.clone(), a.into());
        }
        index
            .map
            .insert(c.labels.reference.clone(), Node::Component(component_idx));
    }

    for sub_sch in sch.sub_schematics.iter_mut() {
        if index.map.contains_key(&sub_sch.id) {
            return Err(errorf(&format!(
                "component and schematic name collision: {}",
                sub_sch.id
            )));
        }
        index
            .map
            .insert(sub_sch.id.clone(), Node::Sheet(index_schematic(sub_sch)?));
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
    let entry = idx
        .resolve_entry(p.iter())
        .ok_or(errorf("entry not found"))?;

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
