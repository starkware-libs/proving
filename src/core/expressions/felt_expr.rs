use serde::{Deserialize, Serialize};

use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use crate::core::autogen_structs::*;

pub type FeltConst = ConstExpr<Felt>;

// A variable of type felt. It can be a field (attribute) of another expression, like UInt16Expr, or
// a standalone variable. It can represent a felt expression that was written to the trace.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct FeltVar {
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<Felt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) state_index: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) parent: Option<Box<ExprImpl>>,
}

// A felt expression can be a constant, a variable, a binary operation, or a unary operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeltExpr {
    Const(FeltConst),
    Var(FeltVar),
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change the expression
    // into a variable that has a state index.
    pub fn to_state(&mut self, index: usize) {
        match self {
            FeltExpr::Var(v) => {
                v.name = format!("state{}", index);
                v.state_index = Some(index);
                v.parent = None;
            }
            _ => panic!("Cannot convert a constant to a state"),
        }
    }

    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let FeltExpr::Var(v) = self {
            v.parent = Some(Box::new(parent));
        }
    }

    // Creates a new FeltVar.
    pub fn new_var(name: String, value: Option<Felt>, state_index: Option<usize>) -> Self {
        FeltVar {
            name,
            value,
            state_index,
            parent: None,
        }
        .into()
    }
}

impl Expr<Felt> for FeltExpr {
    fn value(&self) -> Option<Felt> {
        match self {
            FeltExpr::Const(c) => Some(c.value),
            FeltExpr::Var(v) => v.value,
        }
    }
}

impl AirVar for FeltExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None)
    }

    fn name(&self) -> String {
        match self {
            FeltExpr::Const(c) => c.name.clone(),
            FeltExpr::Var(v) => v.name.clone(),
        }
    }

    fn create_intermediate_var(&self, _name: String) -> Self {
        match self {
            FeltExpr::Var(v) => v.clone().into(),
            _ => panic!("Cannot create an intermediate variable from a constant"),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            FeltExpr::Const(_) => true,
            FeltExpr::Var(v) => v.state_index.is_some(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        vec![self]
    }
}

impl Default for FeltExpr {
    fn default() -> Self {
        FeltExpr::Var(FeltVar::default())
    }
}

impl From<FeltConst> for FeltExpr {
    fn from(constant: FeltConst) -> FeltExpr {
        FeltExpr::Const(constant)
    }
}

impl From<FeltVar> for FeltExpr {
    fn from(variable: FeltVar) -> FeltExpr {
        FeltExpr::Var(variable)
    }
}

impl From<FeltExpr> for ProcessedAirVar {
    fn from(expr: FeltExpr) -> ProcessedAirVar {
        match expr {
            FeltExpr::Const(c) => ProcessedAirVar::Const(Felt::r#type(), c.name),
            FeltExpr::Var(v) => {
                if let Some(i) = v.state_index {
                    return ProcessedAirVar::State(i);
                }
                if let Some(var) = v.parent {
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), v.name, vec![]);
                }
                ProcessedAirVar::Var(Felt::r#type(), v.name)
            }
        }
    }
}

#[macro_export]
macro_rules! const_expr {
    ($val:expr) => {
        FeltConst::new_const(Felt { value: $val }).into()
    };
}

#[macro_export]
macro_rules! expr {
    ($name:expr, $val:expr) => {
        FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), None)
    };
}
