use serde::{Deserialize, Serialize};
use std::fmt::Display;

use super::super::air_fn_registry::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use crate::core::autogen_structs::*;

pub type BoolConst = ConstExpr<Bool>;
pub type BoolBinary = BinaryExpr<Bool>;
pub type BoolUnary = UnaryExpr<Bool>;

// A variable of type Bool. Holds its name, value, and Felt representation.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BoolVar {
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<Bool>,
    #[serde(skip)]
    pub(super) as_felt: FeltExpr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoolExpr {
    Const(BoolConst),
    Var(BoolVar),
    Binary(BoolBinary),
    Unary(BoolUnary),
}

impl BoolExpr {
    pub fn as_felt(&mut self) -> &mut FeltExpr {
        match self {
            BoolExpr::Var(v) => &mut v.as_felt,
            BoolExpr::Unary(u) => {
                if u.op == UnaryOp::FeltAsBool {
                    if let ExprImpl::Felt(felt_expr) = &mut *u.child {
                        if let FeltExpr::Var(_) = felt_expr {
                            return felt_expr;
                        }
                    }
                }
                panic!("Cannot convert to a Felt");
            }
            _ => panic!("Cannot convert to a Felt"),
        }
    }

    // Creates a new BoolVar.
    pub fn new_var(name: String, value: Option<Bool>, state_index: Option<usize>) -> Self {
        let mut res = BoolVar {
            name,
            value,
            as_felt: FeltExpr::new_var(
                "as_felt".to_string(),
                value.map(|v| v.as_felt()),
                state_index,
            ),
        };
        res.as_felt
            .set_parent(ExprImpl::Bool(res.clone().into()), None);
        res.into()
    }
}

impl Expr<Bool> for BoolExpr {
    fn value(&self) -> Option<Bool> {
        match self {
            BoolExpr::Const(c) => Some(c.value),
            BoolExpr::Var(v) => v.value,
            BoolExpr::Binary(b) => b.value,
            BoolExpr::Unary(u) => u.value,
        }
    }
}

impl AirVar for BoolExpr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None)
    }

    fn name(&self) -> String {
        match self {
            BoolExpr::Const(c) => c.name.clone(),
            BoolExpr::Var(v) => v.name.clone(),
            BoolExpr::Binary(b) => b.name.clone(),
            BoolExpr::Unary(u) => u.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            BoolExpr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            BoolExpr::Const(_) => panic!("Cannot create an intermediate variable from a constant"),
            _ => Self::new_var(name, self.value(), None),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            BoolExpr::Const(_) => true,
            BoolExpr::Var(v) => v.as_felt.in_state(),
            BoolExpr::Binary(b) => b.left.in_state() && b.right.in_state(),
            BoolExpr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        vec![self.as_felt()]
    }
}

impl Default for BoolExpr {
    fn default() -> Self {
        BoolExpr::Var(BoolVar::default())
    }
}

impl From<BoolConst> for BoolExpr {
    fn from(c: BoolConst) -> BoolExpr {
        BoolExpr::Const(c)
    }
}

impl From<BoolVar> for BoolExpr {
    fn from(v: BoolVar) -> BoolExpr {
        BoolExpr::Var(v)
    }
}

impl From<BoolBinary> for BoolExpr {
    fn from(b: BoolBinary) -> BoolExpr {
        BoolExpr::Binary(b)
    }
}

impl From<BoolUnary> for BoolExpr {
    fn from(u: BoolUnary) -> BoolExpr {
        BoolExpr::Unary(u)
    }
}

impl From<BoolExpr> for GenericAirVar {
    fn from(expr: BoolExpr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<BoolExpr> for ProcessedAirVar {
    fn from(expr: BoolExpr) -> ProcessedAirVar {
        let name = expr.name();
        if name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX) {
            return ProcessedAirVar::Var(Bool::r#type(), name);
        }

        match expr {
            BoolExpr::Const(_) => ProcessedAirVar::Const(Bool::r#type(), name),
            BoolExpr::Var(_) => ProcessedAirVar::Var(Bool::r#type(), name),
            BoolExpr::Binary(b) => b.into(),
            BoolExpr::Unary(u) => u.into(),
        }
    }
}

impl Display for BoolExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[macro_export]
macro_rules! const_bool_expr {
    ($val:expr) => {
        BoolConst::new_const(Bool { value: $val }).into()
    };
}

#[macro_export]
macro_rules! bool_expr {
    ($name:expr, $val:expr) => {
        BoolExpr::new_var($name.to_string(), Some(Bool::from($val)), None)
    };
}
