use crate::error::{errorf, DynamicResult};
use crate::types;
use crate::types::Attribute;
use evalexpr::{EvalexprError, EvalexprResult, Value, ValueType};
use std::cell::RefCell;

#[derive(Debug)]
pub struct Entry<'a> {
    set_in_progress: RefCell<bool>,
    attribute: &'a mut Attribute,
    value: Option<Value>,
}

// This (slightly modified) function's origin is for some reason marked as private for
// external consumers in evalexpr, even though all the type-specific variants are exposed.
fn expected_type(expected_type: &ValueType, actual: Value) -> EvalexprError {
    match expected_type {
        ValueType::String => EvalexprError::expected_string(actual),
        ValueType::Int => EvalexprError::expected_int(actual),
        ValueType::Float => EvalexprError::expected_float(actual),
        ValueType::Boolean => EvalexprError::expected_boolean(actual),
        ValueType::Tuple => EvalexprError::expected_tuple(actual),
        ValueType::Empty => EvalexprError::expected_empty(actual),
    }
}

impl<'a> Entry<'a> {
    pub fn get_expression(&self) -> &str {
        &self.attribute.expression
    }

    pub fn get_value(&self) -> Option<&Value> {
        self.value.as_ref()
    }

    pub fn update(&mut self, value: Value) -> EvalexprResult<Option<Value>> {
        *self.set_in_progress.borrow_mut() = false;

        let mut str = value.to_string();
        if let Some(unit) = self.attribute.unit.as_ref() {
            str.push(' ');
            str.push_str(unit);
        }

        self.attribute.value = types::Value::parse(str);
        if let Some(t) = self.value.as_ref().map(|v| ValueType::from(v)) {
            if t != ValueType::from(&value) {
                return Err(expected_type(&t, value));
            }
        }

        Ok(self.value.replace(value))
    }

    pub fn value_defined(&self) -> DynamicResult<bool> {
        if self.value.is_none() && *self.set_in_progress.borrow() {
            return Err(errorf("dependency loop detected"));
        }

        *self.set_in_progress.borrow_mut() = true;
        Ok(self.value.is_some())
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
