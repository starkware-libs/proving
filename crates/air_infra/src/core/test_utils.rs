use std::fmt::Display;

use super::air_fn::*;
use super::compiled_structs::*;
use super::expressions::felt_expr::*;

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
        }
    }
}

fn felts_to_string(felts: &[CompiledAirVar]) -> String {
    let mut strs = felts.iter().map(ToString::to_string).collect::<Vec<_>>();
    let mut i = 0;
    let mut leading_zeros = false;
    for s in strs.iter().rev() {
        if s == "const_0" {
            leading_zeros = true;
            i += 1;
        } else {
            break;
        }
    }
    strs.truncate(strs.len() - i);
    let str = format!("[{}]", strs.join(", "));
    if leading_zeros {
        format!("zero_extend({})", str)
    } else {
        str
    }
}

fn felts_vec_to_string(felts: Vec<FeltExpr>) -> String {
    felts_to_string(
        &(felts
            .iter()
            .map(|f| (f.clone().into()))
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
                    felts_to_string(input_felts),
                    felts_to_string(output_felts)
                )
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
            AirBodyComponent::DeductionIntermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
            AirBodyComponent::ConstraintIntermediate(name, var) => {
                write!(f, "{} = {}", name, CompiledAirVar::from(var.clone()))
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
        }
    }
}
