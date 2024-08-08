use serde::{Serialize, Serializer};

use crate::core::Felt;

use super::super::air_fn_registry::*;
use super::super::compiled_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;

// Macros
use crate::const_expr;

pub type FeltOperation = OpExpr<Felt>;

// A variable of type Felt. It can be a field (attribute) of another expression, like UInt16Expr, or
// a standalone variable. It can represent a felt expression that was written to the trace.
#[derive(Clone, Debug)]
pub struct FeltVar {
    pub(super) name: String,
    pub(super) value: Option<Felt>,
    pub(super) state_index: Option<usize>,
    pub(super) parent: Option<ParentExpr>,
    pub(super) is_const: bool,
}

// A felt expression can be a constant, a variable, a binary operation, or a unary operation.
#[derive(Clone, Debug)]
pub enum FeltExpr {
    Var(FeltVar),
    Op(FeltOperation),
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change the expression
    // into a variable that has a state index.
    pub fn to_state(&mut self, index: usize) {
        assert!(!self.name().starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));

        let name = format!("state[{}]", index);
        let value = self.value();
        match self {
            FeltExpr::Var(v) => {
                v.name = name;
                v.state_index = Some(index)
            }
            _ => *self = Self::new_var(name, value, Some(index), false),
        }
    }

    pub(super) fn set_parent(&mut self, parent: ParentExpr) {
        if let FeltExpr::Var(v) = self {
            v.parent = Some(parent);
        } else {
            panic!("Cannot set parent of a non-variable");
        }
    }

    // Creates a new FeltVar.
    pub fn new_var(
        name: String,
        value: Option<Felt>,
        state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        FeltVar {
            name,
            value,
            state_index,
            parent: None,
            is_const,
        }
        .into()
    }

    // Creates a new constant FeltVar.
    pub fn new_const(value: Felt) -> Self {
        Self::new_var(value.calc(), Some(value), None, true)
    }
}

impl Expr<Felt> for FeltExpr {
    fn value(&self) -> Option<Felt> {
        match self {
            FeltExpr::Var(v) => v.value,
            FeltExpr::Op(b) => b.value,
        }
    }
}

impl AirVar for FeltExpr {
    fn name(&self) -> String {
        match self {
            FeltExpr::Var(v) => v.name.clone(),
            FeltExpr::Op(b) => b.name.clone(),
        }
    }

    fn as_felts_mut(&mut self) -> Vec<&mut FeltExpr> {
        vec![self]
    }
}

impl InternalAirVarActions for FeltExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn let_(&self, name: String) -> Self {
        match self {
            FeltExpr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, self.is_const()),
        }
    }
}

impl InternalAirVarInfo for FeltExpr {
    fn in_state(&self) -> bool {
        if self.is_const() {
            return true;
        }

        match self {
            FeltExpr::Var(v) => {
                v.state_index.is_some()
                    || v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || v.name.starts_with(BOTH_INTERMEDIATE_VAR_PREFIX)
            }
            FeltExpr::Op(op) => op.children.iter().all(|c| c.in_state()),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            FeltExpr::Var(v) => v.is_const,
            FeltExpr::Op(op) => op.children.iter().all(|c| c.is_const()),
        }
    }
}

// Default is implemented for FeltExpr because it is stored in memory.
impl Default for FeltExpr {
    fn default() -> Self {
        const_expr!(0)
    }
}

impl From<FeltVar> for FeltExpr {
    fn from(variable: FeltVar) -> FeltExpr {
        FeltExpr::Var(variable)
    }
}

impl From<FeltOperation> for FeltExpr {
    fn from(binary: FeltOperation) -> FeltExpr {
        FeltExpr::Op(binary)
    }
}

impl From<FeltExpr> for CompiledAirVar {
    fn from(expr: FeltExpr) -> CompiledAirVar {
        match expr {
            FeltExpr::Var(v) => {
                // v is an intermediate variable
                if v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX)
                    || v.name.starts_with(BOTH_INTERMEDIATE_VAR_PREFIX)
                {
                    return CompiledAirVar::Var(Felt::r#type(), v.name);
                }

                // v is a constant
                if v.is_const {
                    return CompiledAirVar::Const(Felt::r#type(), v.value.unwrap().calc());
                }

                // v was written to the trace
                if let Some(i) = v.state_index {
                    return CompiledAirVar::State(i);
                }

                // v is a field of another variable
                if let Some(parent) = v.parent {
                    return parent.get_compiled_child();
                }

                // v is a standalone variable
                CompiledAirVar::Var(Felt::r#type(), v.name)
            }
            FeltExpr::Op(op) => op.into(),
        }
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
        FeltExpr::new_const($crate::core::Felt::from_u32_unchecked($val))
    };
}

#[cfg(test)]
#[macro_export]
macro_rules! expr {
    ($name:expr, $val:expr) => {
        FeltExpr::new_var(
            $name.to_string(),
            Some($crate::core::Felt::from($val)),
            None,
            false,
        )
    };
}
