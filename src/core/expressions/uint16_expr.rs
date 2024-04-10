use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

pub type UInt16Const = ConstExpr<UInt16>;
pub type UInt16Binary = BinaryExpr<UInt16>;

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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UInt16Expr {
    Const(UInt16Const),
    Var(UInt16Var),
    Binary(UInt16Binary),
}

impl UInt16Expr {
    pub fn as_felt(&mut self) -> &mut FeltExpr {
        match self {
            UInt16Expr::Var(v) => &mut v.as_felt,
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }

    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let UInt16Expr::Var(v) = self {
            v.parent = Some(Box::new(parent));
            v.as_felt.set_parent(UInt16Expr::Var(v.clone()).into());
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
        res.as_felt.set_parent(ExprImpl::UInt16(res.clone().into()));
        res.into()
    }
}

impl Expr<UInt16> for UInt16Expr {
    fn value(&self) -> Option<UInt16> {
        match self {
            UInt16Expr::Const(c) => Some(c.value),
            UInt16Expr::Var(v) => v.value,
            UInt16Expr::Binary(b) => b.value,
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
        }
    }

    fn create_intermediate_var(&self, name: String) -> Self {
        match self {
            UInt16Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            UInt16Expr::Binary(b) => Self::new_var(name, b.value, None),
            _ => panic!("Cannot create an intermediate variable from a constant"),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            UInt16Expr::Const(_) => true,
            UInt16Expr::Var(v) => v.as_felt.in_state(),
            UInt16Expr::Binary(b) => b.left.in_state() && b.right.in_state(),
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

impl From<UInt16Expr> for GenericAirVar {
    fn from(expr: UInt16Expr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<UInt16Expr> for ProcessedAirVar {
    fn from(expr: UInt16Expr) -> ProcessedAirVar {
        match expr {
            UInt16Expr::Const(c) => ProcessedAirVar::Const(Bool::r#type(), c.name),
            UInt16Expr::Var(v) => {
                if let Some(var) = v.parent {
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), v.name, vec![]);
                }

                ProcessedAirVar::Var(Bool::r#type(), v.name)
            }
            UInt16Expr::Binary(b) => b.into(),
        }
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
}
