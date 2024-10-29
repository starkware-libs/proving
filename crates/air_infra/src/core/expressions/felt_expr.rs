use compiled_casm_air::public_params::PublicParam;
use serde::{Serialize, Serializer};

use compiled_casm_air::compiled_structs::CompiledAirVar;

use crate::core::Felt;

use super::super::state::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;
use super::var_expr::*;

// Macros
use crate::const_expr;

pub type FeltOperation = OpExpr<Felt>;
pub type FeltExpr = Expr<Felt>;

// Describes where in the state this FeltExpr resides
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StateInfo {
    // The felt is in the state of the current component, at the specified index.
    // The second argument is the description of the trace cell. It is used only for compilation.
    // Consider moving to a compilation context.
    StateIndex(usize, Option<String>),
    // If the <bool> value is true, the felt is a polynomial expression in the state. It is unspecified
    // what this polynomial is. If the <bool> is false, the felt is not a polynomial expression in the
    // state (for example, a value read from the memory and not written to the state yet).
    IsPolyOfState(bool),
    // The felt is in the state of another component. The arguments are the component name and the index
    // inside that component.
    ExternalColumnStateIndex(String, usize),
    // The felt is one of the public parameters.
    PublicParam(PublicParam),
}

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
    pub fn to_state(&mut self, new_state_info: StateInfo) {
        let name = match &new_state_info {
            StateInfo::StateIndex(index, desc) => State::get_cell_name(*index, desc),
            StateInfo::IsPolyOfState(_) => {
                panic!("to_state shouldn't be used to make a FeltExpr an IsPolyOfState")
            }
            StateInfo::ExternalColumnStateIndex(name, index) => {
                format!("{}_state[{}]", name, index)
            }
            StateInfo::PublicParam(public_param) => public_param.name(),
        };
        let value = self.value();

        match self {
            FeltExpr::Var(v) => {
                v.name = name;
                v.complex_or_felt = ComplexOrFelt::Felt(new_state_info);
                // A felt expression that is written to the trace is no longer an intermediate variable.
                v.intermediate_type = None;
            }
            _ => {
                let mut v = VarExpr::new(name, value, false, true, None);
                v.complex_or_felt = ComplexOrFelt::Felt(new_state_info);
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
