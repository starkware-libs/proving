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

impl Display for DeductionOrIntermediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeductionOrIntermediate::Deduction(var) => write!(f, "{}", var),
            DeductionOrIntermediate::Intermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
        }
    }
}

impl Display for ConstraintOrIntermediate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConstraintOrIntermediate::Constraint(var) => write!(f, "{}", var),
            ConstraintOrIntermediate::Intermediate(name, var) => {
                write!(f, "{} = {}", name, var)
            }
        }
    }
}
