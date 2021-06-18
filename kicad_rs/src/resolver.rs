use std::collections::HashMap;
use crate::error::{DynamicResult};
use evalexpr::{Context, ContextWithMutableVariables, Value, EvalexprResult, EvalexprError, HashMapContext};
use crate::resolver::entry::*;

mod entry {
    use crate::types::Attribute;
    use crate::types::Value as TypesValue;
    use evalexpr::Value;
    use crate::error::{DynamicResult, errorf};
    use std::cell::RefCell;

    #[derive(Debug)]
    pub struct Entry<'a> {
        set_in_progress: RefCell<bool>,
        attribute: &'a mut Attribute,
        value: Option<Value>,
    }

    impl<'a> Entry<'a> {
        pub fn update(&mut self, value: Value) -> Option<Value> {
            *self.set_in_progress.borrow_mut() = false;
            self.attribute.value = TypesValue::parse(value.to_string());
            self.value.replace(value)
        }

        pub fn value_defined(&self) -> DynamicResult<bool> {
            if *self.set_in_progress.borrow() {
                // TODO: More precise error reporting
                return Err(errorf("dependency loop detected"));
            }
            *self.set_in_progress.borrow_mut() = true;
            Ok(self.value.is_some())
        }

        pub fn get_value(&self) -> Option<&Value> {
            self.value.as_ref()
        }

        pub fn get_expression(&self) -> &str {
            &self.attribute.expression
        }
    }

    impl<'a> From<&'a mut Attribute> for Entry<'a> {
        fn from(attribute: &'a mut Attribute) -> Self {
            Self {
                set_in_progress: RefCell::new(false),
                attribute,
                value: None,
            }
        }
    }
}

pub type ComponentIndex<'a> = HashMap<String, Entry<'a>>;

#[derive(Debug)]
pub struct SheetIndex<'a> {
    pub map: HashMap<String, Node<'a>>,
}

#[derive(Debug)]
pub enum Node<'a> {
    Sheet(SheetIndex<'a>),
    Component(ComponentIndex<'a>),
}

// TODO: A trait for resolving canonical paths in the component namespace structure
// trait Asdf {
//     fn test(&self);
// }
//
// impl Asdf for &str {
//     fn test(&self) {
//         unimplemented!();
//     }
// }

impl<'a> SheetIndex<'a> {
    pub fn new() -> Self {
        Self {
            map: HashMap::new()
        }
    }

    fn resolve_depth(&self, path: &[&str]) -> Option<&Entry> {
        self.map.get(*path.first()?).map(|n| {
            match n {
                Node::Sheet(idx) => idx.resolve_depth(&path[1..]),
                Node::Component(idx) => {
                    if path.len() > 2 { None } else {
                        idx.get(*path.get(2).unwrap_or(&""))
                    }
                }
            }
        }).flatten()
    }

    fn update_entry_depth_mut(&mut self, path: &[&str], value: Value) -> Option<Value> {
        // let head = path.first()?;
        // let attribute_ref = path.last().unwrap_or(&""); // "" is the default attribute

        self.map.get_mut(*path.first()?).map(|n| {
            match n {
                Node::Sheet(idx) => idx.update_entry_depth_mut(&path[1..], value),
                Node::Component(idx) =>
                    if path.len() > 2 { None } else {
                        idx.get_mut(*path.get(2).unwrap_or(&"")).map(|e| e.update(value)
                        ).flatten()
                    },
            }
        }).flatten()

        // match path.len() {
        //     0 => panic!("what?"),
        //     l => self.map.get_mut(path[0]).map(|n| {
        //         match n {
        //             Node::Sheet(m) => if l != 1 { m.update_entry_depth_mut(&path[1..], value) } else { panic!() },
        //             Node::Component(e) => if l == 1 { e.value = value } else { panic!() },
        //         }
        //     }),
        // };
    }

    // fn resolve_entry_mut(&'a mut self, identifier: &str) -> Option<&'a mut Entry> {
    //     self.resolve_depth_mut(identifier.split('.').collect::<Vec<_>>().as_slice())
    // }

    fn resolve_entry(&self, identifier: &str) -> Option<&Entry> {
        self.resolve_depth(identifier.split('.').collect::<Vec<_>>().as_slice())
    }

    fn update_entry(&mut self, identifier: &str, value: Value) {
        self.update_entry_depth_mut(identifier.split('.').collect::<Vec<_>>().as_slice(), value);
    }
}

// fn resolve_depth_mut<'b>(im: &'b mut SheetIndex<'b>, path: &[&str]) -> Option<&'b mut Entry<'b>> {
//     match path.len() {
//         0 => None,
//         l => im.map.get_mut(path[0]).map(|n| {
//             match n {
//                 Node::Sheet(m) => if l != 1 { resolve_depth_mut(m, &path[1..]) } else { None },
//                 Node::Component(e) => if l == 1 { Some(e) } else { None },
//             }
//         }).flatten(),
//     }
// }
//
// fn resolve_entry_mut<'b>(im: &'b mut SheetIndex<'b>, identifier: &str) -> Option<&'b mut Entry<'b>> {
//     resolve_depth_mut(im, identifier.split('.').collect::<Vec<_>>().as_slice())
// }

impl<'a> Context for SheetIndex<'a> {
    fn get_value(&self, identifier: &str) -> Option<&Value> {
        self.resolve_entry(identifier).map(|e| e.get_value()).flatten()
    }

    fn call_function(&self, identifier: &str, argument: &Value) -> EvalexprResult<Value> {
        // TODO: Fixed function set (voltage divider etc.)
        unimplemented!("functions are currently unsupported");
    }
}

impl<'a> ContextWithMutableVariables for SheetIndex<'a> {
    fn set_value(&mut self, identifier: String, value: Value) -> EvalexprResult<()> {
        println!("Updating entry: {}", identifier);
        self.update_entry(&identifier, value);

        // TODO: Enable the type safety checks below
        // let path = identifier.split('.').collect::<Vec<_>>().as_slice();
        //
        // let t = self.map.get_mut(path[0]);
        //
        // match path.len() {
        //     0 => None,
        //     l => t.map(|n| {
        //         match n {
        //             Node::Branch(m) => if l != 1 { Some(m.set_value(identifier, value).unwrap()) } else { None },
        //             Node::Leaf(e) => if l == 1 { Some(()) } else { None },
        //         }
        //     }).flatten(),
        // };

        // if let Node::Leaf(existing_value) = t {
        //     // if let Some(existing_value) = a {
        //         if ValueType::from(&existing_value.value) == ValueType::from(&value) {
        //             existing_value.value = value;
        //             return Ok(());
        //         } else {
        //             // TODO: EvalexprError::expected_type is private...
        //             return Err(EvalexprError::CustomMessage("type mismatch".into()));
        //             // return Err(errorf("Mismatching types"));
        //             // return Err(EvalexprError::expected_type(&existing_value.value, value));
        //         }
        //     // }
        // }

        // let a: Option<&mut Entry> = self.resolve_entry_mut(&identifier);
        // let a: Option<&mut Entry> = resolve_depth_mut(&mut self, vec![].as_slice());


        //
        //
        //
        // Err(EvalexprError::CustomMessage("stuff".into()))
        //
        // // Implicit else, because `self.variables` and `identifier` are not unborrowed in else
        // // self.variables.insert(identifier, value);
        // // unimplemented!();
        Ok(())
    }
}

// TODO: Some experimentation

pub fn resolve_test(index: &mut SheetIndex) {
    // Perform resolving recursively in depth-first order
    for v in index.map.values_mut() {
        if let Node::Sheet(sub_index) = v {
            resolve_test(sub_index);
        }
    }

    // Evaluate attributes for all components
    let keys: Vec<String> = index.map.keys().map(|s| s.into()).collect();
    for k in keys {
        evaluate_test(index, k).unwrap();
    }
}

fn evaluate_test(a: &mut SheetIndex, k: String) -> DynamicResult<()> {
    println!("{}", k);
    // TODO: Error handling for entries not found
    let entry = a.resolve_entry(&k).unwrap();

    if entry.value_defined()? {
        return Ok(()); // Don't update if already set
    }

    // if let Node::Leaf(entry) = &a.map[&k] {
    let node = evalexpr::build_operator_tree(
        entry.get_expression(),
        // a.get_attribute(k.as_str()).borrow().expression.as_str(),
    ).expect("no err"); // TODO: Error handling for invalid expressions

    for dep in node.iter_variable_identifiers() {
        evaluate_test(a, dep.to_string())?;
    }

    let val = node.eval_with_context(a).unwrap(); // TODO: Error handling

    a.set_value(k, val).unwrap(); // TODO: Error handling

    // for v in a.tree.values_mut() {
    //     evaluate2_test(v);
    // }
    // }
    Ok(())
}

// fn evaluate2_test(a: &mut Node) {
//     evaluate2_test(a);
// }

// pub fn resolve(index: &IndexMap) -> DynamicResult<()> {
//     resolve_depth(index, &[""]);
//
//     // for id in index.keys() {
//     //     evaluate(&mut c, index, id.into());
//     // }
//
//     Ok(())
// }

// fn resolve_depth(index: &IndexMap, prefix: &[&str]) {
//     // Perform resolving recursively in depth-first order
//     for (k, v) in index.tree.iter() {
//         if let Node::Branch(sub_index) = v {
//             resolve_depth(sub_index, &[prefix, &[k]].concat());
//         }
//     }
//
//     // Evaluate all attributes
//     for v in index.tree.values() {
//         if let Node::Leaf(attribute) = v {
//             evaluate(index, *attribute.borrow_mut());
//         }
//     }
//
//     // for r in refs.iter() {
//     //     evaluate(index, prefix, r);
//     // }
// }

// fn evaluate<T: ContextWithMutableVariables>(c: &T, target: String) {
//     if c.get_value(&target.name).is_some() {
//         return;
//     }
//
//     let node = evalexpr::build_operator_tree(
//         &target.expression,
//     ).expect("no err"); // TODO: Error handling
//
//     for dep in node.iter_variable_identifiers() {
//         // TODO: Error if there's a dependency on target itself
//         evaluate(c, target);
//     }
//
//     target.value = node.eval_with_context(c).unwrap(); // TODO: Error handling
//     // c.update_cache(target);
//     // TODO: Update the cache somehow?
//
//     // // TODO: This set should directly update values inside attributes
//     // c.set_value(target.name, val).unwrap(); // TODO: Error handling
// }
