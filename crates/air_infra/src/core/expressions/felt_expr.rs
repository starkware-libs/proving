use serde::{Deserialize, Serialize};

use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::expr::*;
use super::op_expr::*;
use crate::core::air_fn_registry::*;
use crate::core::autogen_structs::*;

pub type FeltConst = ConstExpr<Felt>;
pub type FeltBinary = BinaryExpr<Felt>;
pub type FeltUnary = UnaryExpr<Felt>;

// A variable of type Felt. It can be a field (attribute) of another expression, like UInt16Expr, or
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
    Binary(FeltBinary),
    Unary(FeltUnary),
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change the expression
    // into a variable that has a state index.
    pub fn to_state(&mut self, index: usize) {
        assert!(!self.name().starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));

        let name = format!("state[{}]", index);
        let value = self.value();
        match self {
            FeltExpr::Const(_) => panic!("Cannot convert a constant to a state"),
            FeltExpr::Var(v) => {
                v.name = name;
                v.state_index = Some(index)
            }
            _ => *self = Self::new_var(name, value, Some(index)),
        }
    }

    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let FeltExpr::Var(v) = self {
            v.parent = Some(Box::new(parent));
        } else {
            panic!("Cannot set parent of a non-variable");
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

    pub fn let_for_constraint(&self, name: String) -> Self {
        assert!(name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));

        match self {
            FeltExpr::Const(_) => panic!("Cannot create an intermediate variable from a constant"),
            _ => Self::new_var(name, self.value(), None),
        }
    }

    pub fn as_bool(&self) -> BoolExpr {
        let value = self.value().map(|v| v.as_bool());
        match self {
            FeltExpr::Const(_) => panic!("Cannot create an intermediate variable from a constant"),
            _ => BoolExpr::new_var(self.name(), value, None),
        }
    }
}

impl Expr<Felt> for FeltExpr {
    fn value(&self) -> Option<Felt> {
        match self {
            FeltExpr::Const(c) => Some(c.value),
            FeltExpr::Var(v) => v.value,
            FeltExpr::Binary(b) => b.value,
            FeltExpr::Unary(u) => u.value,
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
            FeltExpr::Binary(b) => b.name.clone(),
            FeltExpr::Unary(u) => u.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            FeltExpr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            FeltExpr::Const(_) => panic!("Cannot create an intermediate variable from a constant"),
            _ => Self::new_var(name, self.value(), None),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            FeltExpr::Const(_) => true,
            FeltExpr::Var(v) => {
                v.state_index.is_some()
                    || v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || (v.parent.is_some()
                        && v.parent
                            .as_ref()
                            .unwrap()
                            .name()
                            .starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX))
            }
            FeltExpr::Binary(b) => b.left.in_state() && b.right.in_state(),
            FeltExpr::Unary(u) => u.child.in_state(),
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

impl From<FeltBinary> for FeltExpr {
    fn from(binary: FeltBinary) -> FeltExpr {
        FeltExpr::Binary(binary)
    }
}

impl From<FeltExpr> for GenericAirVar {
    fn from(expr: FeltExpr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<FeltUnary> for FeltExpr {
    fn from(unary: FeltUnary) -> FeltExpr {
        FeltExpr::Unary(unary)
    }
}

impl From<FeltExpr> for ProcessedAirVar {
    fn from(expr: FeltExpr) -> ProcessedAirVar {
        let name = expr.name();
        if name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
            || name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX)
        {
            return ProcessedAirVar::Var(Felt::r#type(), name);
        }

        match expr {
            FeltExpr::Const(_) => ProcessedAirVar::Const(Felt::r#type(), name),
            FeltExpr::Var(v) => {
                if let Some(i) = v.state_index {
                    return ProcessedAirVar::State(i);
                }
                if let Some(var) = v.parent {
                    if var.name().starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX) {
                        return ProcessedAirVar::Var(Felt::r#type(), var.name());
                    }
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), name, vec![]);
                }
                ProcessedAirVar::Var(Felt::r#type(), name)
            }
            FeltExpr::Binary(b) => b.into(),
            FeltExpr::Unary(u) => u.into(),
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

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), Some(0))
        } else {
            FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), None)
        }
    };
}
