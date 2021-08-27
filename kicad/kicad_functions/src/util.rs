use evalexpr::{EvalexprError, EvalexprResult};

/// Returns an `EvalexprResult` with a `EvalexprError::CustomMessage` error
pub fn err<T>(msg: &str) -> EvalexprResult<T> {
    Err(EvalexprError::CustomMessage(msg.into()))
}
