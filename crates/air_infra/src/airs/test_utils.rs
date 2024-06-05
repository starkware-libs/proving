use std::fmt::Display;

use crate::core::autogen_structs::*;

impl Display for ProcessedAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessedAirVar::StaticCall(id, args) => {
                write!(f, "{}(", id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ProcessedAirVar::MethodCall(left, id, args) => {
                write!(f, "{}.{}(", left, id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ProcessedAirVar::Const(_, id) => write!(f, "const_{}", id),
            ProcessedAirVar::Var(_, id) => write!(f, "{}", id),
            ProcessedAirVar::State(i) => write!(f, "state[{}]", i),
            ProcessedAirVar::BinaryOp(lhs, op, rhs) => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }
            ProcessedAirVar::UnaryOp(op, expr) => {
                write!(f, "({} {})", op, expr)
            }
            ProcessedAirVar::Tuple(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            ProcessedAirVar::Array(exprs) => {
                write!(f, "[")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, "]")
            }
        }
    }
}

impl Display for TraceGenerationStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TraceGenerationStep::Deduction(var) => write!(f, "{}", var),
            TraceGenerationStep::Intermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
            TraceGenerationStep::Lookup {
                fn_name,
                input,
                output_name,
            } => {
                write!(f, "{} = {}({})", output_name, fn_name, input)
            }
        }
    }
}

fn processed_air_var_vec_to_string(vec: &Vec<ProcessedAirVar>) -> String {
    let mut parts: Vec<String> = vec![];

    for pav in vec {
        parts.push(format!("{}", pav));
    }
    format!("[{}]", parts.join(","))
}

impl Display for ConstraintOrIntermediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintOrIntermediate::InInstanceConstraint(var) => write!(f, "{}", var),
            ConstraintOrIntermediate::Intermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
            ConstraintOrIntermediate::LookupConstraint {
                fn_name,
                input_felts,
                output_felts,
            } => {
                write!(
                    f,
                    "{}({}) == {}",
                    fn_name,
                    processed_air_var_vec_to_string(input_felts),
                    processed_air_var_vec_to_string(output_felts)
                )
            }
        }
    }
}
