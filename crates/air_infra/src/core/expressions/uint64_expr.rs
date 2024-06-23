use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint32_expr::*;

pub type UInt64Binary = BinaryExpr<UInt64>;
pub type UInt64Unary = UnaryExpr<UInt64>;

// A variable of type UInt64. Holds its name, and value. It is represented as two UInt32 variables.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UInt64Var {
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<UInt64>,
    #[serde(skip)]
    pub(super) low: UInt32Expr,
    #[serde(skip)]
    pub(super) high: UInt32Expr,
    pub(super) is_const: bool,
}

impl UInt64Var {
    // Updates the low and high parts of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_parts(&mut self) {
        let mut self_copy = self.clone();
        self_copy.low = UInt32Expr::default();
        self_copy.high = UInt32Expr::default();
        let parent: ExprImpl = UInt64Expr::Var(self_copy.clone()).into();
        self.low.set_parent(parent.clone());
        self.high.set_parent(parent);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UInt64Expr {
    Var(UInt64Var),
    Binary(UInt64Binary),
    Unary(UInt64Unary),
}

impl UInt64Expr {
    pub fn low(&mut self) -> &mut UInt32Expr {
        match self {
            UInt64Expr::Var(v) => &mut v.low,
            _ => panic!("Cannot convert non-variable to UInt32"),
        }
    }

    pub fn high(&mut self) -> &mut UInt32Expr {
        match self {
            UInt64Expr::Var(v) => &mut v.high,
            _ => panic!("Cannot convert non-variable to UInt32"),
        }
    }

    // Creates a new UInt64Var.
    pub fn new_var(
        name: String,
        value: Option<UInt64>,
        ll_state_index: Option<usize>,
        lh_state_index: Option<usize>,
        hl_state_index: Option<usize>,
        hh_state_index: Option<usize>,
        is_const: bool,
    ) -> Self {
        let mut res = UInt64Var {
            name,
            value,
            low: UInt32Expr::new_var(
                "low".to_string(),
                value.map(|v| v.low()),
                ll_state_index,
                lh_state_index,
                is_const,
            ),
            high: UInt32Expr::new_var(
                "high".to_string(),
                value.map(|v| v.high()),
                hl_state_index,
                hh_state_index,
                is_const,
            ),
            is_const,
        };
        res.update_parts();
        res.into()
    }

    // Creates a new constant UInt64Var.
    pub fn new_const(value: UInt64) -> Self {
        Self::new_var(value.calc(), Some(value), None, None, None, None, true)
    }
}

impl Expr<UInt64> for UInt64Expr {
    fn value(&self) -> Option<UInt64> {
        match self {
            UInt64Expr::Var(v) => v.value,
            UInt64Expr::Binary(b) => b.value,
            UInt64Expr::Unary(u) => u.value,
        }
    }
}

impl AirVar for UInt64Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, None, None, None, false)
    }

    fn name(&self) -> String {
        match self {
            UInt64Expr::Var(v) => v.name.clone(),
            UInt64Expr::Binary(b) => b.name.clone(),
            UInt64Expr::Unary(u) => u.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            UInt64Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_parts();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, None, None, None, self.is_const()),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            UInt64Expr::Var(v) => v.low.in_state() && v.high.in_state(),
            UInt64Expr::Binary(b) => b.left.in_state() && b.right.in_state(),
            UInt64Expr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            UInt64Expr::Var(v) => {
                let mut res = vec![];
                res.append(&mut v.low.as_felts());
                res.append(&mut v.high.as_felts());
                res
            }
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }

    fn is_const(&self) -> bool {
        match self {
            UInt64Expr::Var(v) => v.is_const,
            UInt64Expr::Binary(b) => b.left.is_const() && b.right.is_const(),
            UInt64Expr::Unary(u) => u.child.is_const(),
        }
    }
}

impl Default for UInt64Expr {
    fn default() -> Self {
        UInt64Expr::Var(UInt64Var::default())
    }
}

impl From<UInt64Var> for UInt64Expr {
    fn from(v: UInt64Var) -> UInt64Expr {
        UInt64Expr::Var(v)
    }
}

impl From<UInt64Binary> for UInt64Expr {
    fn from(b: UInt64Binary) -> UInt64Expr {
        UInt64Expr::Binary(b)
    }
}

impl From<UInt64Unary> for UInt64Expr {
    fn from(u: UInt64Unary) -> UInt64Expr {
        UInt64Expr::Unary(u)
    }
}

impl From<UInt64Expr> for GenericAirVar {
    fn from(expr: UInt64Expr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<UInt64Expr> for ProcessedAirVar {
    fn from(expr: UInt64Expr) -> ProcessedAirVar {
        match expr {
            UInt64Expr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return ProcessedAirVar::Var(UInt64::r#type(), v.name);
                }
                if v.is_const {
                    return ProcessedAirVar::Const(UInt64::r#type(), v.name);
                }
                ProcessedAirVar::Var(UInt64::r#type(), v.name)
            }
            UInt64Expr::Binary(b) => b.into(),
            UInt64Expr::Unary(u) => u.into(),
        }
    }
}

impl Display for UInt64Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[macro_export]
macro_rules! const_u64_expr {
    ($val:expr) => {
        UInt64Expr::new_const($val.into())
    };
}

#[macro_export]
macro_rules! u64_expr {
    ($name:expr, $val:expr) => {
        UInt64Expr::new_var(
            $name.to_string(),
            Some(UInt64::from($val)),
            None,
            None,
            None,
            None,
            false,
        )
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            UInt64Expr::new_var(
                $name.to_string(),
                Some(UInt64::from($val)),
                Some(0),
                Some(1),
                Some(2),
                Some(3),
                false,
            )
        } else {
            UInt64Expr::new_var(
                $name.to_string(),
                Some(UInt64::from($val)),
                None,
                None,
                None,
                None,
                false,
            )
        }
    };
}
