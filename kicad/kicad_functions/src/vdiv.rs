use crate::util::err;
use evalexpr::{
    ContextWithMutableVariables, EvalexprError, EvalexprResult, HashMapContext, Node, Value,
};
use resistor_calc::{RCalc, RRes, RSeries};
use std::collections::HashSet;
use std::panic::panic_any;

fn parse_series(str: &str) -> EvalexprResult<&'static RSeries> {
    match str.trim() {
        "E3" => Ok(&resistor_calc::E3),
        "E6" => Ok(&resistor_calc::E6),
        "E12" => Ok(&resistor_calc::E12),
        "E24" => Ok(&resistor_calc::E24),
        "E48" => Ok(&resistor_calc::E48),
        "E96" => Ok(&resistor_calc::E96),
        _ => err(&format!("unknown resistor series: {}", str)),
    }
}

// evalexpr is not smart enough to distinguish identifiers on its own
fn unique_identifiers(e: &Node) -> usize {
    let mut set = HashSet::new();
    e.iter_variable_identifiers().for_each(|i| {
        set.insert(i);
    });
    set.len()
}

struct VoltageDividerConfig {
    target: f64,
    expression: Node,
    count: usize,
    series: &'static RSeries,
    resistance_min: Option<f64>,
    resistance_max: Option<f64>,
}

impl VoltageDividerConfig {
    fn parse(v: &Value) -> EvalexprResult<Self> {
        let tuple = v.as_tuple()?;

        let resistance_min = tuple.get(3).map(|v| v.as_number()).transpose()?;
        let resistance_max = tuple.get(4).map(|v| v.as_number()).transpose()?;

        // TODO: Support external constants
        if let [target, expression, series] = &tuple[..3] {
            let expression = evalexpr::build_operator_tree(&expression.as_string()?)?;
            let count = unique_identifiers(&expression);

            Ok(Self {
                target: target.as_number()?,
                expression,
                count,
                series: parse_series(&series.as_string()?)?,
                resistance_min,
                resistance_max,
            })
        } else {
            err(&format!("unsupported argument count: {}", tuple.len()))
        }
    }
}

fn calculate(config: &VoltageDividerConfig) -> Option<RRes> {
    let calc = RCalc::new(vec![config.series; config.count]);

    calc.calc(|set| {
        if let Some(true) = config.resistance_min.map(|r| set.sum() < r) {
            return None; // Sum of resistance less than minimum
        }

        if let Some(true) = config.resistance_max.map(|r| set.sum() > r) {
            return None; // Sum of resistance larger than maximum
        }

        // TODO: Storing this externally and using interior mutability
        //  could be helpful to avoid reallocating on every invocation.
        let mut context = HashMapContext::new();
        for i in 1..=config.count {
            context
                .set_value(format!("R{}", i).into(), Value::Float(set.r(i)))
                .unwrap();
        }

        match config.expression.eval_with_context(&context) {
            Ok(v) => Some((config.target - v.as_number().unwrap()).abs()),
            Err(e) => match &e {
                EvalexprError::DivisionError { divisor: d, .. } => {
                    if let Ok(n) = d.as_number() {
                        if n == 0.0 {
                            // This soft-catch may be a bit redundant. Based on some testing the
                            // internal conversions in evalexpr cause zero values to deviate
                            // slightly from zero, thus avoiding division by zero even if you
                            // explicitly write a zero division into the voltage divider equation.
                            return None;
                        }
                    }
                    panic_any(e) // No graceful way to handle this from the closure
                }
                _ => panic_any(e),
            },
        }
    })
}

/// `voltage_divider` computes values for resistor-based voltage dividers.
/// - Usage: vdiv(<target voltage>, <divider expression>, <resistor series>, (min resistance), (max resistance))
/// - Example: vdiv(5.1, "(R1+R2)/R2*0.8", "E96", 500e3, 700e3)
/// - Output: (<closest voltage>, <R1 value>, <R2 value>, ...)
/// There can be arbitrary many resistors in the divider, but they must be named "R1", "R2", etc.
/// The computed optimal resistance values are also presented in this order. The minimal and maximal
/// resistance limits are optional parameters, and only consider the sum of the resistances of all
/// resistors defined in the expression.
pub(crate) fn voltage_divider(argument: &Value) -> EvalexprResult<Value> {
    let config = VoltageDividerConfig::parse(argument)?;
    if let Some(res) = calculate(&config) {
        // Take the first result, these are ordered by increasing error
        if let Some((v, set)) = res.iter().next() {
            let voltage = config.target + *v as f64 / 1e9;
            let mut tuple = vec![Value::from(voltage)];
            for i in 1..=config.count {
                tuple.push(Value::from(set.r(i)));
            }
            return Ok(Value::from(tuple));
        }
    }

    err(&format!("no solution found: {}", argument))
}
