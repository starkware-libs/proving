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

fn felts_to_string(felts: &[ProcessedAirVar]) -> String {
    let mut strs = felts.iter().map(ToString::to_string).collect::<Vec<_>>();
    let mut i = 0;
    for s in strs.iter().rev() {
        if s == "const_0" {
            i += 1;
        } else {
            break;
        }
    }
    strs.truncate(strs.len() - i);
    format!("[{}]", strs.join(", "))
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
                    felts_to_string(input_felts),
                    felts_to_string(output_felts)
                )
            }
        }
    }
}
