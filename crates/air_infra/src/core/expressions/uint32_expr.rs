use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint16_expr::*;

pub type UInt32Const = ConstExpr<UInt32>;
pub type UInt32Binary = BinaryExpr<UInt32>;
pub type UInt32Unary = UnaryExpr<UInt32>;

// A variable of type UInt32. Holds its name, and value. It is represented as two UInt16 variables.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UInt32Var {
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<UInt32>,
    #[serde(skip)]
    pub(super) low: UInt16Expr,
    #[serde(skip)]
    pub(super) high: UInt16Expr,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) parent: Option<Box<ExprImpl>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UInt32Expr {
    Const(UInt32Const),
    Var(UInt32Var),
    Binary(UInt32Binary),
    Unary(UInt32Unary),
}

impl UInt32Expr {
    pub fn low(&mut self) -> &mut UInt16Expr {
        match self {
            UInt32Expr::Var(v) => &mut v.low,
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }

    pub fn high(&mut self) -> &mut UInt16Expr {
        match self {
            UInt32Expr::Var(v) => &mut v.high,
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }

    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let UInt32Expr::Var(v) = self {
            v.parent = Some(Box::new(parent));
            v.low.set_parent(UInt32Expr::Var(v.clone()).into());
            v.high.set_parent(UInt32Expr::Var(v.clone()).into());
        } else {
            panic!("Cannot set parent of a non-variable");
        }
    }

    // Creates a new UInt32Var.
    pub fn new_var(
        name: String,
        value: Option<UInt32>,
        low_state_index: Option<usize>,
        high_state_index: Option<usize>,
    ) -> Self {
        let mut res = UInt32Var {
            name,
            value,
            low: UInt16Expr::new_var("low".to_string(), value.map(|v| v.low()), low_state_index),
            high: UInt16Expr::new_var(
                "high".to_string(),
                value.map(|v| v.high()),
                high_state_index,
            ),
            parent: None,
        };
        res.low.set_parent(ExprImpl::UInt32(res.clone().into()));
        res.high.set_parent(ExprImpl::UInt32(res.clone().into()));
        res.into()
    }
}

impl Expr<UInt32> for UInt32Expr {
    fn value(&self) -> Option<UInt32> {
        match self {
            UInt32Expr::Const(c) => Some(c.value),
            UInt32Expr::Var(v) => v.value,
            UInt32Expr::Binary(b) => b.value,
            UInt32Expr::Unary(u) => u.value,
        }
    }
}

impl AirVar for UInt32Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, None)
    }

    fn name(&self) -> String {
        match self {
            UInt32Expr::Const(c) => c.name.clone(),
            UInt32Expr::Var(v) => v.name.clone(),
            UInt32Expr::Binary(b) => b.name.clone(),
            UInt32Expr::Unary(u) => u.name.clone(),
        }
    }

    fn create_intermediate_var_for_deduction(&self, name: String) -> Self {
        match self {
            UInt32Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            UInt32Expr::Binary(b) => Self::new_var(name, b.value, None, None),
            UInt32Expr::Unary(u) => Self::new_var(name, u.value, None, None),
            _ => panic!("Cannot create an intermediate variable from a constant"),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            UInt32Expr::Const(_) => true,
            UInt32Expr::Var(v) => v.low.in_state() && v.high.in_state(),
            UInt32Expr::Binary(b) => b.left.in_state() && b.right.in_state(),
            UInt32Expr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            UInt32Expr::Var(v) => vec![v.low.as_felt(), v.high.as_felt()],
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }
}

impl Default for UInt32Expr {
    fn default() -> Self {
        UInt32Expr::Var(UInt32Var::default())
    }
}

impl From<UInt32Const> for UInt32Expr {
    fn from(c: UInt32Const) -> UInt32Expr {
        UInt32Expr::Const(c)
    }
}

impl From<UInt32Var> for UInt32Expr {
    fn from(v: UInt32Var) -> UInt32Expr {
        UInt32Expr::Var(v)
    }
}

impl From<UInt32Binary> for UInt32Expr {
    fn from(b: UInt32Binary) -> UInt32Expr {
        UInt32Expr::Binary(b)
    }
}

impl From<UInt32Unary> for UInt32Expr {
    fn from(u: UInt32Unary) -> UInt32Expr {
        UInt32Expr::Unary(u)
    }
}

impl From<UInt32Expr> for GenericAirVar {
    fn from(expr: UInt32Expr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<UInt32Expr> for ProcessedAirVar {
    fn from(expr: UInt32Expr) -> ProcessedAirVar {
        match expr {
            UInt32Expr::Const(c) => ProcessedAirVar::Const(UInt32::r#type(), c.name),
            UInt32Expr::Var(v) => {
                if let Some(var) = v.parent {
                    return ProcessedAirVar::MethodCall(Box::new((*var).into()), v.name, vec![]);
                }

                ProcessedAirVar::Var(UInt32::r#type(), v.name)
            }
            UInt32Expr::Binary(b) => b.into(),
            UInt32Expr::Unary(u) => u.into(),
        }
    }
}

#[macro_export]
macro_rules! const_u32_expr {
    ($val:expr) => {
        UInt32Const::new_const($val.into()).into()
    };
}

#[macro_export]
macro_rules! u32_expr {
    ($name:expr, $val:expr) => {
        UInt32Expr::new_var($name.to_string(), Some(UInt32::from($val)), None, None)
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            UInt32Expr::new_var(
                $name.to_string(),
                Some(UInt32::from($val)),
                Some(0),
                Some(1),
            )
        } else {
            UInt32Expr::new_var($name.to_string(), Some(UInt32::from($val)), None, None)
        }
    };
}
