pub mod util;
mod vdiv;

use evalexpr::{EvalexprError, EvalexprResult, Value};

// Match function for all custom functions available in the kicad_rs evaluator
pub fn call_function(identifier: &str, argument: &Value) -> EvalexprResult<Value> {
    match identifier {
        "vdiv" => vdiv::voltage_divider(argument),
        other => Err(EvalexprError::FunctionIdentifierNotFound(other.into())),
    }
}
