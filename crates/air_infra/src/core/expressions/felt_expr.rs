use serde::{Serialize, Serializer};

use crate::core::Felt;

use super::super::compiled_structs::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;
use super::var_expr::*;

// Macros
use crate::const_expr;

pub type FeltOperation = OpExpr<Felt>;
pub type FeltExpr = Expr<Felt>;

impl VarExprUpdate for VarExpr<Felt> {
    fn create_children(&mut self) {
        // Felt does not have children.
    }
    fn update_children(&mut self) {
        // Felt does not have children.
    }
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change the expression
    // into a variable that has a state index.
    pub fn to_state(&mut self, index: usize, external: Option<String>) {
        let name = format!("state[{}]", index);
        let value = self.value();
        match self {
            FeltExpr::Var(v) => {
                v.name = name;
                v.complex_or_felt = if let Some(external_name) = external {
                    ComplexOrFelt::Felt(StateInfo::ExternalColumnStateIndex(external_name, index))
                } else {
                    ComplexOrFelt::Felt(StateInfo::StateIndex(index))
                };
                // A felt expression that is written to the trace is no longer an intermediate variable.
                v.intermediate_type = None;
            }
            _ => {
                let mut v = VarExpr::new(name, value, false, true, None);
                v.complex_or_felt = ComplexOrFelt::Felt(StateInfo::StateIndex(index));
                *self = Self::Var(v);
            }
        }
    }
}

impl AirVar for FeltExpr {
    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self]
    }
}

// Implemented for FeltExpr because it is the output of an external column (Seq).
impl From<Vec<FeltExpr>> for FeltExpr {
    fn from(mut v: Vec<FeltExpr>) -> Self {
        if v.len() == 1 {
            v.pop().unwrap()
        } else {
            panic!("Expected a vector of length 1, found length {}", v.len());
        }
    }
}

// Default is implemented for FeltExpr because it is stored in memory.
impl Default for FeltExpr {
    fn default() -> Self {
        const_expr!(0)
    }
}

// Serialize is implemented for FeltExpr because it appears in air body.
impl Serialize for FeltExpr {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let var: CompiledAirVar = self.clone().into();
        serializer.collect_str(&var.to_string())
    }
}

#[macro_export]
macro_rules! const_expr {
    ($val:expr) => {
        FeltExpr::Var($crate::core::expressions::var_expr::VarExpr::new_const(
            $crate::core::Felt::from_u32_unchecked($val),
        ))
    };
}

#[cfg(test)]
macro_rules! expr {
    ($name:expr, $val:expr) => {
        FeltExpr::Var($crate::core::expressions::var_expr::VarExpr::new(
            $name.to_string(),
            Some($crate::core::Felt::from($val)),
            false,
            false,
            None,
        ))
    };
}
#[cfg(test)]
pub(in crate::core) use expr;
