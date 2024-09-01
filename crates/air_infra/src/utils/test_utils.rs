use serde_json::Value;
use std::fmt::Display;
use std::fs;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::compiled_structs::*;
use crate::core::expressions::expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::core::utils::*;
use crate::core::variables::*;

impl Display for TraceGenStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceGenStep::Deduction(var) => write!(f, "{}", var),
            TraceGenStep::Intermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
            TraceGenStep::Lookup {
                fn_name,
                input,
                output_name,
            } => {
                write!(f, "{} = {}({})", output_name, fn_name, input)
            }
            TraceGenStep::AccessExternalColumn {
                fn_name,
                output_name,
            } => {
                write!(f, "{} = external({})", output_name, fn_name)
            }
        }
    }
}

fn felts_vec_to_string(felts: Vec<FeltExpr>) -> String {
    vars_arr_to_string(
        &(felts
            .iter()
            .map(|f| f.clone().into())
            .collect::<Vec<CompiledAirVar>>()),
    )
}

pub fn compare_test_json(registry: AirFnRegistry, air_fn_name: &String, file_path: &String) {
    let is_fix_mode = std::env::var("FIX") == Ok("1".to_string());
    if is_fix_mode {
        registry.dump_to_file(Some(air_fn_name), Some(file_path));
    } else {
        let json_file = fs::read_to_string(file_path.clone()).unwrap();
        let expected_entry_json: Value =
            serde_json::from_str(&json_file).expect("Invalid JSON file for the expected entry");
        let entry = registry.get_air_fn_entry(air_fn_name);
        let entry_json =
            serde_json::to_value(&entry).expect("Failed to convert current entry to JSON value");
        assert_eq!(entry_json, expected_entry_json);
    };
}

impl Display for ConstraintEvalStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintEvalStep::InInstanceConstraint(var) => write!(f, "{}", var),
            ConstraintEvalStep::Intermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
            ConstraintEvalStep::LookupConstraint {
                fn_name,
                input_felts,
                output_felts,
            } => {
                write!(
                    f,
                    "{}({}) == {}",
                    fn_name,
                    vars_arr_to_string(input_felts),
                    vars_arr_to_string(output_felts)
                )
            }
            ConstraintEvalStep::AccessExternalColumn {
                fn_name,
                output_name,
            } => {
                write!(f, "{} = external({})", output_name, fn_name)
            }
        }
    }
}

impl Display for AirBodyComponent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirBodyComponent::Constraint(var) => {
                write!(f, "Constraint: {}", CompiledAirVar::from(var.clone()))
            }
            AirBodyComponent::Deduction(var) => {
                write!(f, "Deduction: {}", CompiledAirVar::from(var.clone()))
            }
            AirBodyComponent::Assignment {
                constraint,
                deduction: _,
            } => {
                write!(
                    f,
                    "Assignment: {}",
                    CompiledAirVar::from(constraint.clone())
                )
            }
            AirBodyComponent::Intermediate(name, var, _ty) => {
                write!(f, "{} = {}", name, var)
            }
            AirBodyComponent::Call(var) => {
                write!(f, "{} = {}({})", var.output, var.air_fn_name, var.input_arg)
            }
            AirBodyComponent::LookupCall(var) => {
                write!(
                    f,
                    "{} = {}({})",
                    var.output_name, var.air_fn_name, var.input_arg
                )
            }
            AirBodyComponent::LookupConstraint(var) => write!(
                f,
                "{}({}) == {}",
                var.air_fn_name,
                felts_vec_to_string(var.input_felts.clone()),
                felts_vec_to_string(var.output_felts.clone())
            ),
            AirBodyComponent::AccessExternalColumn(access) => {
                write!(
                    f,
                    "{} = external({})",
                    access.output_name, access.air_fn_name
                )
            }
        }
    }
}

impl Display for AirVarImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AirVarImpl::Expr(expr) => {
                write!(f, "{}", expr)
            }
            _ => {
                write!(f, "{}", CompiledAirVar::from(self.clone()))
            }
        }
    }
}

impl Display for ExprImpl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CompiledAirVar::from(self.clone()))
    }
}

impl<T> Display for Expr<T>
where
    T: ProverType,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", CompiledAirVar::from(self.clone()))
    }
}
