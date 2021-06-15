use evalexpr::{ContextWithMutableVariables, HashMapContext};
use std::collections::HashMap;

pub struct Expression {
    expr: String,
    namespace: String, // TODO: This
}

impl Expression {
    pub fn new(expr: String, namespace: String) -> Self {
        Self { expr, namespace }
    }
}

pub fn resolve(expressions: &HashMap<String, Expression>) -> HashMapContext {
    let mut c = HashMapContext::new();

    for id in expressions.keys() {
        evaluate(&mut c, expressions, id.into());
    }

    c
}

fn evaluate<T: ContextWithMutableVariables>(
    c: &mut T,
    e: &HashMap<String, Expression>,
    id: String,
) {
    if c.get_value(&id).is_some() {
        return;
    }

    let node = evalexpr::build_operator_tree(
        &e[&id].expr, // TODO: Error handling
    )
    .expect("no err"); // TODO: Error handling
    for dep in node.iter_variable_identifiers() {
        evaluate(c, e, String::from(dep));
    }

    let val = node.eval_with_context(c).unwrap(); // TODO: Error handling
    c.set_value(id, val).unwrap(); // TODO: Error handling
}
