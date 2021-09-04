use crate::util::err;
use evalexpr::{EvalexprError, EvalexprResult, Value};

/// `index` retrieves tuple values based on the given index.
/// - Usage: idx(<tuple>, <i>)
/// - Example: idx(("a", "b", "c"), 1) -> "b"
/// - Output: i:th value in the tuple (zero-indexed)
pub(crate) fn index(argument: &Value) -> EvalexprResult<Value> {
    let args = argument.as_tuple()?;
    if let [target, index] = &args[..] {
        let index = index.as_int()?;
        return target
            .as_tuple()?
            .get(index as usize)
            .ok_or(EvalexprError::CustomMessage(format!(
                "index out of bounds: {}",
                index
            )))
            .map(|v| v.clone());
    }

    err(&format!("unsupported argument count: {}", args.len()))
}
