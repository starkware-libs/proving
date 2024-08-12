use std::fmt::Display;

use super::air_fn::*;
use super::compiled_structs::*;
use super::expressions::felt_expr::*;
use super::utils::*;

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
