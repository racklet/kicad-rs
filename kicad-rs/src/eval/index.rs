use crate::eval::entry::Entry;
use crate::eval::path::Path;
use crate::parser::VALUE_FIELD_KEY;
use evalexpr::{Context, ContextWithMutableVariables, EvalexprError, EvalexprResult, Value};
use std::collections::HashMap;

pub type ComponentIndex<'a> = HashMap<String, Entry<'a>>;

#[derive(Default, Debug)]
pub struct SheetIndex<'a> {
    pub(crate) map: HashMap<String, Node<'a>>,
}

#[derive(Debug)]
pub enum Node<'a> {
    Sheet(SheetIndex<'a>),
    Component(ComponentIndex<'a>),
}

impl<'a> SheetIndex<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn resolve_entry<'b>(
        &self,
        mut path: impl ExactSizeIterator<Item = &'b String>,
    ) -> Option<&Entry> {
        self.map
            .get(path.next()?)
            .map(|n| match n {
                Node::Sheet(idx) => idx.resolve_entry(path),
                Node::Component(idx) => {
                    if path.len() > 1 {
                        None // There's more elements, an incomplete path was given
                    } else {
                        idx.get(path.next().unwrap_or(&String::from(VALUE_FIELD_KEY)))
                    }
                }
            })
            .flatten()
    }

    pub fn update_entry<'b>(
        &mut self,
        mut path: impl ExactSizeIterator<Item = &'b String>,
        value: Value,
    ) -> EvalexprResult<Option<Value>> {
        match self
            .map
            .get_mut(path.next().ok_or(err("path exhausted"))?)
            .ok_or(err("entry not found"))?
        {
            Node::Sheet(idx) => idx.update_entry(path, value),
            Node::Component(idx) => {
                if path.len() > 1 {
                    Err(err("component encountered during traversal"))
                } else {
                    idx.get_mut(path.next().unwrap_or(&String::from(VALUE_FIELD_KEY)))
                        .ok_or(err("attribute not found"))?
                        .update(value)
                }
            }
        }
    }
}

impl<'a> Context for SheetIndex<'a> {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.resolve_entry(Path::from(identifier).iter())
            .map(|e| e.get_value())
            .flatten()
    }

    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value> {
        kicad_functions::call_function(identifier, argument)
    }
}

impl<'a> ContextWithMutableVariables for SheetIndex<'a> {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        self.update_entry(Path::from(identifier).iter(), value)
            .map(|_| ())
    }
}

fn err(msg: &str) -> EvalexprError {
    EvalexprError::CustomMessage(msg.into())
}
