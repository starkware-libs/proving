use std::fmt::Display;

use super::compiled_structs::*;

impl Display for CompiledAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompiledAirVar::StaticCall(id, args) => {
                write!(f, "{id}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            CompiledAirVar::MethodCall(left, id, args) => {
                write!(f, "{left}.{id}(")?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{arg}")?;
                }
                write!(f, ")")
            }
            CompiledAirVar::Const(_, id) => write!(f, "{id}"),
            CompiledAirVar::Var(_, id) => {
                write!(f, "{id}")
            }
            CompiledAirVar::State(name) => {
                write!(f, "{name}")
            }
            CompiledAirVar::BinaryOp(lhs, op, rhs) => {
                write!(f, "({lhs} {op} {rhs})")
            }
            CompiledAirVar::UnaryOp(op, expr) => {
                write!(f, "({op} {expr})")
            }
            CompiledAirVar::Tuple(exprs) => {
                let strs = exprs.iter().map(ToString::to_string).collect::<Vec<_>>();
                write!(f, "{}", &format!("({})", strs.join(", ")))
            }
            CompiledAirVar::Array(exprs) => {
                write!(f, "{}", vars_arr_to_string(exprs))
            }
            CompiledAirVar::Struct { fields, .. } => {
                let strs = fields
                    .iter()
                    .map(|(name, expr)| format!("{name}: {expr}"))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{{{strs}}}")
            }
            CompiledAirVar::ExternalState(col_id) => {
                write!(f, "ExternalState({col_id})")
            }
            CompiledAirVar::PublicParam(name) => {
                write!(f, "public_params.{name}")
            }
            CompiledAirVar::Enabler => write!(f, "Enabler"),
            CompiledAirVar::Multiplicity(idx) => write!(f, "Multiplicity({idx})"),
        }
    }
}

pub fn vars_arr_to_string(felts: &[CompiledAirVar]) -> String {
    let mut strs = felts.iter().map(ToString::to_string).collect::<Vec<_>>();
    let mut i = 0;
    let mut leading_zeros = false;
    for s in strs.iter().rev() {
        if s == "0" {
            leading_zeros = true;
            i += 1;
        } else {
            break;
        }
    }
    strs.truncate(strs.len() - i);
    let str = format!("[{}]", strs.join(", "));
    if leading_zeros { format!("zero_extend({str})") } else { str }
}
