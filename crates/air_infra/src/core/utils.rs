use std::fmt::Display;

use crate::core::compiled_structs::*;

impl Display for CompiledAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompiledAirVar::StaticCall(id, args) => {
                write!(f, "{}(", id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            CompiledAirVar::MethodCall(left, id, args) => {
                write!(f, "{}.{}(", left, id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            CompiledAirVar::Const(_, id) => write!(f, "const_{}", id),
            CompiledAirVar::Var(_, id) => write!(f, "{}", id),
            CompiledAirVar::State(i) => write!(f, "state[{}]", i),
            CompiledAirVar::BinaryOp(lhs, op, rhs) => {
                write!(f, "({} {} {})", lhs, op, rhs)
            }
            CompiledAirVar::UnaryOp(op, expr) => {
                write!(f, "({} {})", op, expr)
            }
            CompiledAirVar::Tuple(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            CompiledAirVar::Array(exprs) => {
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
