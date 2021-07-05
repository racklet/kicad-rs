use crate::util::err;
use evalexpr::{
    ContextWithMutableVariables, EvalexprError, EvalexprResult, HashMapContext, Node, Value,
};
use regex::Regex;
use resistor_calc::{RCalc, RRes, RSeries};
use std::cell::RefCell;
use std::collections::HashSet;
use std::ops::Deref;
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

fn resistor_identifiers(e: &Node) -> usize {
    let re = Regex::new(r"^R[1-9][0-9]*$").unwrap();
    let mut set = HashSet::new();
    e.iter_variable_identifiers()
        .filter(|i| re.is_match(i)) // Match only R? identifiers
        .for_each(|i| {
            set.insert(i);
        });
    set.len()
}

// Helper for parsing of potentially single-element tuples
fn parse_tuple(v: &Value) -> Vec<Value> {
    if let Ok(t) = v.as_tuple() {
        return t;
    }

    vec![v.clone()]
}

struct VoltageDividerConfig {
    target: f64,
    expression: Node,
    count: usize,
    series: &'static RSeries,
    resistance_min: Option<f64>,
    resistance_max: Option<f64>,
    extra_parameters: Option<Vec<Value>>,
}

impl VoltageDividerConfig {
    fn parse(v: &Value) -> EvalexprResult<Self> {
        let tuple = v.as_tuple()?;

        let resistance = tuple.get(3).map(|v| v.as_tuple()).transpose()?;
        let resistance_min = resistance
            .as_ref()
            .map(|r| r.get(0).map(|v| v.as_number()))
            .flatten()
            .transpose()?;
        let resistance_max = resistance
            .as_ref()
            .map(|r| r.get(1).map(|v| v.as_number()))
            .flatten()
            .transpose()?;

        let extra_parameters = tuple.get(4).map(|v| parse_tuple(v));

        if let [target, expression, series] = &tuple[..3] {
            let expression = evalexpr::build_operator_tree(&expression.as_string()?)?;
            let count = resistor_identifiers(&expression);

            Ok(Self {
                target: target.as_number()?,
                expression,
                count,
                series: parse_series(&series.as_string()?)?,
                resistance_min,
                resistance_max,
                extra_parameters,
            })
        } else {
            err(&format!("unsupported argument count: {}", tuple.len()))
        }
    }
}

fn calculate(config: &VoltageDividerConfig) -> Option<RRes> {
    let calc = RCalc::new(vec![config.series; config.count]);

    let mut context = HashMapContext::new();
    if let Some(v) = &config.extra_parameters {
        for (i, p) in v.iter().enumerate() {
            context
                .set_value(format!("E{}", i + 1).into(), p.clone())
                .unwrap();
        }
    }

    let context_rc = RefCell::new(context);
    calc.calc(|set| {
        if let Some(true) = config.resistance_min.map(|r| set.sum() < r) {
            return None; // Sum of resistance less than minimum
        }

        if let Some(true) = config.resistance_max.map(|r| set.sum() > r) {
            return None; // Sum of resistance larger than maximum
        }

        for i in 1..=config.count {
            context_rc
                .borrow_mut()
                .set_value(format!("R{}", i).into(), Value::Float(set.r(i)))
                .unwrap();
        }

        match config
            .expression
            .eval_with_context(context_rc.borrow().deref())
        {
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
                    panic_any(e.to_string()) // No graceful way to handle this from the closure
                }
                _ => panic_any(e.to_string()),
            },
        }
    })
}

/// `voltage_divider` computes values for resistor-based voltage dividers.
/// - Usage: vdiv(\<target voltage\>, \<divider expression\>, \<resistor series\>,
///               {(\<min resistance\>, \<max resistance\>)}, ({extra 1}, {extra 2}, ...))
/// - Example: vdiv(5.1, "(R1+R2)/R2*E1", "E96", (500e3, 700e3), (0.8))
/// - Output: (\<closest voltage\>, \<R1 value\>, \<R2 value\>, ...)
/// There can be arbitrary many resistors in the divider, but they must be named "R1", "R2", etc.
/// The computed optimal resistance values are also presented in this order. The minimal and maximal
/// resistance pair is an optional parameter, and the limits only consider the sum of resistance of
/// all resistors defined in the expression. The "extra" parameters are optional external inputs for
/// the divider expression, and will be made available as "E1", "E2", etc. in order.
pub(crate) fn voltage_divider(argument: &Value) -> EvalexprResult<Value> {
    let config = VoltageDividerConfig::parse(argument)?;
    if let Some(res) = calculate(&config) {
        // Take the first result, these are ordered by increasing error
        if let Some((v, set)) = res.iter().next() {
            let voltage = config.target + fixed_to_floating(*v);
            let mut tuple = vec![Value::from(voltage)];
            for i in 1..=config.count {
                tuple.push(Value::from(set.r(i)));
            }
            return Ok(Value::from(tuple));
        }
    }

    err(&format!("no solution found: {}", argument))
}

// resistor_calc outputs error quantities as "fixed point" numbers by multiplying
// a float by 1e9, rounding the result and converting to u64. We need to undo
// that procedure here.
fn fixed_to_floating(value: u64) -> f64 {
    (value as f64) / 1e9
}
