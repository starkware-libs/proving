use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

pub type UInt16Const = ConstExpr<UInt16>;
pub type UInt16Binary = BinaryExpr<UInt16>;
pub type UInt16Unary = UnaryExpr<UInt16>;

// A variable of type UInt16. Holds its name, value, and Felt representation.
// It can be a field (attribute) of another expression, like UInt32Expr, or
// a standalone variable.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UInt16Var {
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<UInt16>,
    #[serde(skip)]
    pub(super) as_felt: FeltExpr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) parent: Option<Box<ExprImpl>>,
}

impl UInt16Var {
    // Updates the Felt representation of the variable.
    // Called whenever a variable is created (see new_var, let_for_deduction and set_parent).
    fn update_as_felt(&mut self) {
        let mut self_copy = self.clone();
        self_copy.as_felt = FeltExpr::default();
        self.as_felt
            .set_parent(UInt16Expr::Var(self_copy).into(), None);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UInt16Expr {
    Const(UInt16Const),
    Var(UInt16Var),
    Binary(UInt16Binary),
    Unary(UInt16Unary),
}

impl UInt16Expr {
    pub fn as_felt(&mut self) -> &mut FeltExpr {
        match self {
            UInt16Expr::Var(v) => &mut v.as_felt,
            UInt16Expr::Unary(u) => {
                if u.op == UnaryOp::UInt16FromBool {
                    if let ExprImpl::Bool(bool_expr) = &mut *u.child {
                        return bool_expr.as_felt();
                    }
                }
                panic!("Cannot convert to a Felt");
            }
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    // Called whenever a parent variable is created (see update_parts of UInt32Expr).
    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let UInt16Expr::Var(v) = self {
            v.parent = Some(Box::new(parent));
            v.update_as_felt();
        } else {
            panic!("Cannot set parent of a non-variable");
        }
    }

    // Creates a new UInt16Var.
    pub fn new_var(name: String, value: Option<UInt16>, state_index: Option<usize>) -> Self {
        let mut res = UInt16Var {
            name,
            value,
            as_felt: FeltExpr::new_var(
                "as_felt".to_string(),
                value.map(|v| v.as_felt()),
                state_index,
            ),
            parent: None,
        };
        res.update_as_felt();
        res.into()
    }
}

impl Expr<UInt16> for UInt16Expr {
    fn value(&self) -> Option<UInt16> {
        match self {
            UInt16Expr::Const(c) => Some(c.value),
            UInt16Expr::Var(v) => v.value,
            UInt16Expr::Binary(b) => b.value,
            UInt16Expr::Unary(u) => u.value,
        }
    }
}

impl AirVar for UInt16Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None)
    }

    fn name(&self) -> String {
        match self {
            UInt16Expr::Const(c) => c.name.clone(),
            UInt16Expr::Var(v) => v.name.clone(),
            UInt16Expr::Binary(b) => b.name.clone(),
            UInt16Expr::Unary(u) => u.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            UInt16Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_as_felt();
                res.into()
            }
            UInt16Expr::Const(_) => {
                panic!("Cannot create an intermediate variable from a constant")
            }
            _ => Self::new_var(name, self.value(), None),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            UInt16Expr::Const(_) => true,
            UInt16Expr::Var(v) => v.as_felt.in_state(),
            UInt16Expr::Binary(b) => b.left.in_state() && b.right.in_state(),
            UInt16Expr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt()]
    }
}

impl Default for UInt16Expr {
    fn default() -> Self {
        UInt16Expr::Var(UInt16Var::default())
    }
}

impl From<UInt16Const> for UInt16Expr {
    fn from(c: UInt16Const) -> UInt16Expr {
        UInt16Expr::Const(c)
    }
}

impl From<UInt16Var> for UInt16Expr {
    fn from(v: UInt16Var) -> UInt16Expr {
        UInt16Expr::Var(v)
    }
}

impl From<UInt16Binary> for UInt16Expr {
    fn from(b: UInt16Binary) -> UInt16Expr {
        UInt16Expr::Binary(b)
    }
}

impl From<UInt16Unary> for UInt16Expr {
    fn from(u: UInt16Unary) -> UInt16Expr {
        UInt16Expr::Unary(u)
    }
}

impl From<UInt16Expr> for GenericAirVar {
    fn from(expr: UInt16Expr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<UInt16Expr> for ProcessedAirVar {
    fn from(expr: UInt16Expr) -> ProcessedAirVar {
        let name = expr.name();
        if name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
            return ProcessedAirVar::Var(UInt16::r#type(), name);
        }

        match expr {
            UInt16Expr::Const(_) => ProcessedAirVar::Const(UInt16::r#type(), name),
            UInt16Expr::Var(v) => {
                if let Some(var) = v.parent {
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), name, vec![]);
                }

                ProcessedAirVar::Var(UInt16::r#type(), name)
            }
            UInt16Expr::Binary(b) => b.into(),
            UInt16Expr::Unary(u) => u.into(),
        }
    }
}

impl Display for UInt16Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();

        if let UInt16Expr::Var(v) = self {
            if !name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                if let Some(p) = v.parent.clone() {
                    return write!(f, "{}.{}()", *p, name);
                }
            }
        }

        write!(f, "{}", name)
    }
}

#[macro_export]
macro_rules! const_u16_expr {
    ($val:expr) => {
        UInt16Const::new_const(UInt16 { value: $val }).into()
    };
}

#[macro_export]
macro_rules! u16_expr {
    ($name:expr, $val:expr) => {
        UInt16Expr::new_var($name.to_string(), Some(UInt16::from($val)), None)
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            UInt16Expr::new_var($name.to_string(), Some(UInt16::from($val)), Some(0))
        } else {
            UInt16Expr::new_var($name.to_string(), Some(UInt16::from($val)), None)
        }
    };
}
