use crate::types::Attribute;
use std::collections::HashMap;
use crate::error::DynamicResult;
use evalexpr::{HashMapContext, Context, ContextWithMutableVariables, Value, EvalexprResult};
use std::cell::RefCell;
use std::borrow::Borrow;

type AttributeRef<'a> = RefCell<&'a mut Attribute>;

pub enum Node<'a> {
    Branch(IndexMap<'a>),
    Leaf(AttributeRef<'a>), // TODO: The whole node might need to be put into a RefCell instead
}

#[derive(Default)]
pub struct IndexMap<'a> {
    pub tree: HashMap<String, Node<'a>>,
    cache: HashMap<AttributeRef<'a>, Value>, // TODO: Use this to cache the Values instead of hosting them in Attribute?
}

impl<'a> IndexMap<'a> {
    pub fn new() -> Self {
        Default::default()
    }
}

trait Index<'a> {
    fn get_attribute(&self, r: &str) -> &AttributeRef<'a>;
    fn update_cache(&self, a: &AttributeRef<'a>);
}

impl<'a> Index<'a> for IndexMap<'a> {
    fn get_attribute(&self, r: &str) -> &'a AttributeRef<'a> {
        unimplemented!("TODO");
    }

    fn update_cache(&self, a: &AttributeRef<'a>) {
        unimplemented!("TODO");
    }
}

impl<'a> Context for IndexMap<'a> {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        // TODO: Use get_attribute to fetch the attribute, then return the value

        unimplemented!("TODO: Resolving using the namespaced identifier");
    }

    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value> {
        unimplemented!("functions are unsupported for now");
    }
}

impl<'a> ContextWithMutableVariables for IndexMap<'a> {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        // if let Some(existing_value) = self.variables.get_mut(&identifier) {
        //     if ValueType::from(&existing_value) == ValueType::from(&value) {
        //         *existing_value = value;
        //         return Ok(());
        //     } else {
        //         return Err(EvalexprError::expected_type(existing_value, value));
        //     }
        // }

        // Implicit else, because `self.variables` and `identifier` are not unborrowed in else
        // self.variables.insert(identifier, value);
        Ok(())
    }
}

// TODO: Some experimentation

pub fn resolve_test(index: &mut IndexMap) -> DynamicResult<()> {
    resolve_depth_test(index);

    Ok(())
}

fn resolve_depth_test(index: &mut IndexMap) {
    // Perform resolving recursively in depth-first order
    for (k, v) in index.tree.iter_mut() {
        if let Node::Branch(sub_index) = v {
            resolve_depth_test(sub_index);
        }
    }

    // Evaluate all attributes
    // for v in index.tree.values() {
    //     if let Node::Leaf(attribute) = v {
    //     }
    // }

    let keys: Vec<&String> = index.tree.keys().collect();
    let b = keys.clone();

    let keys = vec!["key_one".to_string(), "key_two".to_string()];

    for i in keys.iter() {
        evaluate_test(index, i);
    }

    // for r in refs.iter() {
    //     evaluate(index, prefix, r);
    // }
}

fn evaluate_test(a: &mut IndexMap, k: &String) {
    let node = evalexpr::build_operator_tree(
        a.get_attribute(k.as_str()).borrow().expression.as_str(),
    ).expect("no err"); // TODO: Error handling

    for dep in node.iter_variable_identifiers() {
        // TODO: Error if there's a dependency on target itself
        evaluate_test(a, &dep.to_string());
    }

    let val = node.eval_with_context(a).unwrap(); // TODO: Error handling
    // c.update_cache(target);
    // TODO: Update the cache somehow?

    // // TODO: This set should directly update values inside attributes
    a.set_value(k.clone(), val).unwrap(); // TODO: Error handling

    // for v in a.tree.values_mut() {
    //     evaluate2_test(v);
    // }
}

fn evaluate2_test(a: &mut Node) {
    evaluate2_test(a);
}

pub fn resolve(index: &IndexMap) -> DynamicResult<()> {
    resolve_depth(index, &[""]);

    // for id in index.keys() {
    //     evaluate(&mut c, index, id.into());
    // }

    Ok(())
}

fn resolve_depth(index: &IndexMap, prefix: &[&str]) {
    // Perform resolving recursively in depth-first order
    for (k, v) in index.tree.iter() {
        if let Node::Branch(sub_index) = v {
            resolve_depth(sub_index, &[prefix, &[k]].concat());
        }
    }

    // Evaluate all attributes
    for v in index.tree.values() {
        if let Node::Leaf(attribute) = v {
            evaluate(index, *attribute.borrow_mut());
        }
    }

    // for r in refs.iter() {
    //     evaluate(index, prefix, r);
    // }
}

fn evaluate<'a, T: Context + Index<'a>>(c: &T, target: &mut Attribute) {
    if c.get_value(&target.name).is_some() {
        return;
    }

    let node = evalexpr::build_operator_tree(
        &target.expression,
    ).expect("no err"); // TODO: Error handling

    for dep in node.iter_variable_identifiers() {
        // TODO: Error if there's a dependency on target itself
        evaluate(c, *c.get_attribute(dep).borrow_mut());
    }

    target.value = node.eval_with_context(c).unwrap(); // TODO: Error handling
    // c.update_cache(target);
    // TODO: Update the cache somehow?

    // // TODO: This set should directly update values inside attributes
    // c.set_value(target.name, val).unwrap(); // TODO: Error handling
}
