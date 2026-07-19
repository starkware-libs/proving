use std::collections::HashMap;
use std::rc::Rc;

use air_compile::compiled_structs::CompiledAirVar;
use stwo_cairo_common::prover_types::cpu::{M31, QM31};

use crate::util::Environment;

/// The set of variables (both local and global) available inside a certain AirFn
pub struct Scope {
    var_values: HashMap<String, QM31>,

    // The enabler value, for inline components that receive it.
    enabler: Option<QM31>,
    environment: Rc<Environment>,
}

impl Scope {
    pub fn new(environment: Rc<Environment>, enabler: Option<QM31>) -> Scope {
        Scope { var_values: Default::default(), enabler, environment }
    }

    pub fn add_new_var(&mut self, name: String, value: QM31) {
        assert!(!self.var_values.contains_key(&name), "Duplicate var {name}");
        self.var_values.insert(name, value);
    }

    pub fn environment(&self) -> Rc<Environment> {
        Rc::clone(&self.environment)
    }

    /// Compute the value of a CompiledAirVar that evaluates to a single felt.
    pub fn evaluate(&self, expr: &CompiledAirVar) -> QM31 {
        match expr {
            CompiledAirVar::Const(r#type, value) => {
                assert_eq!(r#type, "M31");
                value.parse::<u32>().expect("Const value is not a number").into()
            }
            CompiledAirVar::Var(_, name) | CompiledAirVar::State(name) => {
                *self.var_values.get(name).unwrap_or_else(|| panic!("Unknown name {name}"))
            }
            CompiledAirVar::BinaryOp(left, op, right) => match op.as_str() {
                "+" => self.evaluate(left) + self.evaluate(right),
                "-" => self.evaluate(left) - self.evaluate(right),
                "*" => self.evaluate(left) * self.evaluate(right),
                _ => panic!("Unknown operation {op}"),
            },
            CompiledAirVar::ExternalState(external_state) => *self
                .environment
                .external_states
                .get(external_state)
                .expect("External state not found"),
            CompiledAirVar::PublicParam(name) => <M31 as Into<QM31>>::into(
                *self.environment.public_params.get(name).expect("External state not found"),
            ),
            CompiledAirVar::Enabler => self.enabler.expect("Missing enabler value"),
            _ => panic!("Unexpected expression {expr}"),
        }
    }
}
