use serde::{Deserialize, Serialize};

use super::super::air_fn::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;
use crate::core::autogen_structs::*;

pub type BoolConst = ConstExpr<Bool>;
pub type BoolBinary = BinaryExpr<Bool>;

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
}

impl BoolExpr {
    pub fn as_felt(&mut self) -> &mut FeltExpr {
        match self {
            BoolExpr::Var(v) => &mut v.as_felt,
            _ => panic!("Cannot convert non-variable to Felt"),
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
        res.as_felt.set_parent(ExprImpl::Bool(res.clone().into()));
        res.into()
    }
}

impl Expr<Bool> for BoolExpr {
    fn value(&self) -> Option<Bool> {
        match self {
            BoolExpr::Const(c) => Some(c.value),
            BoolExpr::Var(v) => v.value,
            BoolExpr::Binary(b) => b.value,
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
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
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
        }
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
