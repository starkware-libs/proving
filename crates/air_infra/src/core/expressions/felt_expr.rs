use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::op_expr::*;
use crate::core::air_fn_registry::*;
use crate::core::autogen_structs::*;

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
    pub(super) parent: Option<(Box<ExprImpl>, Option<usize>)>,
    pub(super) is_const: bool,
}

// A felt expression can be a constant, a variable, a binary operation, or a unary operation.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FeltExpr {
    Var(FeltVar),
    Binary(FeltBinary),
    Unary(FeltUnary),
}

impl FeltExpr {
    // When an expression is written to the trace, this function is called to change the expression
    // into a variable that has a state index.
    pub fn to_state(&mut self, index: usize) {
        assert!(!self.name().starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));
        assert!(!self.is_const());

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

    pub fn set_parent(&mut self, parent: ExprImpl, index: Option<usize>) {
        if let FeltExpr::Var(v) = self {
            v.parent = Some((Box::new(parent), index));
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

    pub fn let_for_constraint(&self, name: String) -> Self {
        assert!(name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX));

        Self::new_var(name, self.value(), None, self.is_const())
    }
}

impl Expr<Felt> for FeltExpr {
    fn value(&self) -> Option<Felt> {
        match self {
            FeltExpr::Var(v) => v.value,
            FeltExpr::Binary(b) => b.value,
            FeltExpr::Unary(u) => u.value,
        }
    }
}

impl AirVar for FeltExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, false)
    }

    fn name(&self) -> String {
        match self {
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
            _ => Self::new_var(name, self.value(), None, self.is_const()),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            FeltExpr::Var(v) => {
                v.state_index.is_some()
                    || v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || v.is_const
            }
            FeltExpr::Binary(b) => b.left.in_state() && b.right.in_state(),
            FeltExpr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        vec![self]
    }

    fn is_const(&self) -> bool {
        match self {
            FeltExpr::Var(v) => v.is_const,
            FeltExpr::Binary(b) => b.left.is_const() && b.right.is_const(),
            FeltExpr::Unary(u) => u.child.is_const(),
        }
    }
}

impl Default for FeltExpr {
    fn default() -> Self {
        FeltExpr::Var(FeltVar::default())
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
        match expr {
            FeltExpr::Var(v) => {
                if v.name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                    || v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX)
                {
                    return ProcessedAirVar::Var(Felt::r#type(), v.name);
                }
                if v.is_const {
                    return ProcessedAirVar::Const(Felt::r#type(), v.name);
                }
                if let Some(i) = v.state_index {
                    return ProcessedAirVar::State(i);
                }
                if let Some((var, index)) = v.parent {
                    if let Some(i) = index {
                        let index_var = ProcessedAirVar::Const("usize".to_string(), i.to_string());
                        return ProcessedAirVar::MethodCall(
                            Box::new((*var).into()),
                            v.name,
                            vec![index_var],
                        );
                    }
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), v.name, vec![]);
                }
                ProcessedAirVar::Var(Felt::r#type(), v.name)
            }
            FeltExpr::Binary(b) => b.into(),
            FeltExpr::Unary(u) => u.into(),
        }
    }
}

impl Display for FeltExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();

        if let FeltExpr::Var(v) = self {
            if !name.starts_with(CONSTRAINT_INTERMEDIATE_VAR_PREFIX)
                && !name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX)
                && v.state_index.is_none()
            {
                if let Some((p, index)) = v.parent.clone() {
                    if let Some(i) = index {
                        return write!(f, "{}.{}({})", *p, name, i);
                    }
                    return write!(f, "{}.{}()", *p, name);
                }
            }
        }

        write!(f, "{}", name)
    }
}

#[macro_export]
macro_rules! const_expr {
    ($val:expr) => {
        FeltExpr::new_const(Felt::from_u32_unchecked($val))
    };
}

#[macro_export]
macro_rules! expr {
    ($name:expr, $val:expr) => {
        FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), None, false)
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), Some(0), false)
        } else {
            FeltExpr::new_var($name.to_string(), Some(Felt::from($val)), None, false)
        }
    };
}
