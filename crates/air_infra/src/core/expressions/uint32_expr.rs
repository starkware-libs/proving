use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::super::air_fn_registry::*;
use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use super::uint16_expr::*;

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
    pub(super) is_const: bool,
}

impl UInt32Var {
    // Updates the low and high parts of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_parts(&mut self) {
        let mut self_copy = self.clone();
        self_copy.low = UInt16Expr::default();
        self_copy.high = UInt16Expr::default();
        let parent: ExprImpl = UInt32Expr::Var(self_copy.clone()).into();
        self.low.set_parent(parent.clone());
        self.high.set_parent(parent);
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UInt32Expr {
    Var(UInt32Var),
    Binary(UInt32Binary),
    Unary(UInt32Unary),
}

impl UInt32Expr {
    pub fn low(&mut self) -> &mut UInt16Expr {
        match self {
            UInt32Expr::Var(v) => &mut v.low,
            _ => panic!("Cannot convert non-variable to UInt16"),
        }
    }

    pub fn high(&mut self) -> &mut UInt16Expr {
        match self {
            UInt32Expr::Var(v) => &mut v.high,
            _ => panic!("Cannot convert non-variable to UInt16"),
        }
    }

    // Called whenever a parent variable is created (see update_parts of UInt64Expr).
    pub fn set_parent(&mut self, parent: ExprImpl) {
        if let UInt32Expr::Var(v) = self {
            v.parent = Some(Box::new(parent));
            v.update_parts();
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
        is_const: bool,
    ) -> Self {
        if is_const {
            assert!(value.is_some());
        }

        let mut res = UInt32Var {
            name,
            value,
            low: UInt16Expr::new_var(
                "low".to_string(),
                value.map(|v| v.low()),
                low_state_index,
                is_const,
            ),
            high: UInt16Expr::new_var(
                "high".to_string(),
                value.map(|v| v.high()),
                high_state_index,
                is_const,
            ),
            parent: None,
            is_const,
        };
        res.update_parts();
        res.into()
    }

    // Creates a new constant UInt32Var.
    pub fn new_const(value: UInt32) -> Self {
        Self::new_var(value.calc(), Some(value), None, None, true)
    }
}

impl Expr<UInt32> for UInt32Expr {
    fn value(&self) -> Option<UInt32> {
        match self {
            UInt32Expr::Var(v) => v.value,
            UInt32Expr::Binary(b) => b.value,
            UInt32Expr::Unary(u) => u.value,
        }
    }
}

impl AirVar for UInt32Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None, None, false)
    }

    fn name(&self) -> String {
        match self {
            UInt32Expr::Var(v) => v.name.clone(),
            UInt32Expr::Binary(b) => b.name.clone(),
            UInt32Expr::Unary(u) => u.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            UInt32Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_parts();
                res.into()
            }
            _ => Self::new_var(name, self.value(), None, None, self.is_const()),
        }
    }

    fn in_state(&self) -> bool {
        match self {
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

    fn is_const(&self) -> bool {
        match self {
            UInt32Expr::Var(v) => v.is_const,
            UInt32Expr::Binary(b) => b.left.is_const() && b.right.is_const(),
            UInt32Expr::Unary(u) => u.child.is_const(),
        }
    }
}

impl Default for UInt32Expr {
    fn default() -> Self {
        UInt32Expr::Var(UInt32Var::default())
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
            UInt32Expr::Var(v) => {
                if v.name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
                    return ProcessedAirVar::Var(UInt32::r#type(), v.name);
                }
                if v.is_const {
                    return ProcessedAirVar::Const(UInt32::r#type(), v.value.unwrap().calc());
                }
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

impl Display for UInt32Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = self.name();

        if let UInt32Expr::Var(v) = self {
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
macro_rules! const_u32_expr {
    ($val:expr) => {
        UInt32Expr::new_const($val.into())
    };
}

#[macro_export]
macro_rules! u32_expr {
    ($name:expr, $val:expr) => {
        UInt32Expr::new_var(
            $name.to_string(),
            Some(UInt32::from($val)),
            None,
            None,
            false,
        )
    };

    ($name:expr, $val:expr, $in_trace:literal) => {
        if $in_trace {
            UInt32Expr::new_var(
                $name.to_string(),
                Some(UInt32::from($val)),
                Some(0),
                Some(1),
                false,
            )
        } else {
            UInt32Expr::new_var(
                $name.to_string(),
                Some(UInt32::from($val)),
                None,
                None,
                false,
            )
        }
    };
}
