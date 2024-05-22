use std::array::from_fn;

use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
use super::op_expr::*;

pub type Felt252Const = ConstExpr<Felt252>;
pub type Felt252Binary = BinaryExpr<Felt252>;
pub type Felt252Unary = UnaryExpr<Felt252>;

// A variable of type Felt252. Holds its name, and value. It is represented as FELT252_N_WORDS felts,
// FELT252_BITS_PER_WORD bits each.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Felt252Var {
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<Felt252>,
    #[serde(skip)]
    pub(super) felts: [FeltExpr; FELT252_N_WORDS],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Felt252Expr {
    Const(Felt252Const),
    Var(Felt252Var),
    Binary(Felt252Binary),
    Unary(Felt252Unary),
}

impl Felt252Expr {
    pub fn as_felt_exprs(&mut self) -> &mut [FeltExpr; FELT252_N_WORDS] {
        match self {
            Felt252Expr::Var(v) => &mut v.felts,
            _ => panic!("Cannot convert non-variable to felts."),
        }
    }

    // Creates a new Felt252Var.
    pub fn new_var(
        name: String,
        value: Option<Felt252>,
        state_indices: Option<[usize; FELT252_N_WORDS]>,
    ) -> Self {
        let felts = value.map(|v| v.as_felts());
        let mut res = Felt252Var {
            name,
            value,
            felts: from_fn(|i| {
                FeltExpr::new_var(
                    format!("felt_{}", i),
                    value.map(|_| felts.unwrap()[i]),
                    state_indices.map(|is| is[i]),
                )
            }),
        };
        let res_expr: Felt252Expr = res.clone().into();
        for felt in res.felts.iter_mut() {
            felt.set_parent(ExprImpl::Felt252(res_expr.clone()));
        }
        res.into()
    }
}

impl Expr<Felt252> for Felt252Expr {
    fn value(&self) -> Option<Felt252> {
        match self {
            Felt252Expr::Const(c) => Some(c.value),
            Felt252Expr::Var(v) => v.value,
            Felt252Expr::Binary(b) => b.value,
            Felt252Expr::Unary(u) => u.value,
        }
    }
}

impl AirVar for Felt252Expr {
    fn new(name: String) -> Self {
        Self::new_var(name, None, None)
    }

    fn name(&self) -> String {
        match self {
            Felt252Expr::Const(c) => c.name.clone(),
            Felt252Expr::Var(v) => v.name.clone(),
            Felt252Expr::Binary(b) => b.name.clone(),
            Felt252Expr::Unary(u) => u.name.clone(),
        }
    }

    fn let_for_deduction(&self, name: String) -> Self {
        match self {
            Felt252Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.into()
            }
            Felt252Expr::Const(_) => {
                panic!("Cannot create an intermediate variable from a constant")
            }
            _ => Self::new_var(name, self.value(), None),
        }
    }

    fn in_state(&self) -> bool {
        match self {
            Felt252Expr::Const(_) => true,
            Felt252Expr::Var(v) => v.felts.iter().all(|f| f.in_state()),
            Felt252Expr::Binary(b) => b.left.in_state() && b.right.in_state(),
            Felt252Expr::Unary(u) => u.child.in_state(),
        }
    }

    fn as_felts(&mut self) -> Vec<&mut FeltExpr> {
        match self {
            Felt252Expr::Var(v) => {
                let mut res = Vec::new();
                for felt in v.felts.iter_mut() {
                    res.push(felt);
                }
                res
            }
            _ => panic!("Cannot convert non-variable to Felt"),
        }
    }
}

impl Default for Felt252Expr {
    fn default() -> Self {
        Felt252Expr::Var(Felt252Var::default())
    }
}

impl From<Felt252Const> for Felt252Expr {
    fn from(c: Felt252Const) -> Felt252Expr {
        Felt252Expr::Const(c)
    }
}

impl From<Felt252Var> for Felt252Expr {
    fn from(v: Felt252Var) -> Felt252Expr {
        Felt252Expr::Var(v)
    }
}

impl From<Felt252Binary> for Felt252Expr {
    fn from(b: Felt252Binary) -> Felt252Expr {
        Felt252Expr::Binary(b)
    }
}

impl From<Felt252Unary> for Felt252Expr {
    fn from(u: Felt252Unary) -> Felt252Expr {
        Felt252Expr::Unary(u)
    }
}

impl From<Felt252Expr> for GenericAirVar {
    fn from(expr: Felt252Expr) -> GenericAirVar {
        let expr_impl: ExprImpl = expr.into();
        expr_impl.into()
    }
}

impl From<Felt252Expr> for ProcessedAirVar {
    fn from(expr: Felt252Expr) -> ProcessedAirVar {
        match expr {
            Felt252Expr::Const(c) => ProcessedAirVar::Const(Felt252::r#type(), c.name),
            Felt252Expr::Var(v) => ProcessedAirVar::Var(Felt252::r#type(), v.name),
            Felt252Expr::Binary(b) => b.into(),
            Felt252Expr::Unary(u) => u.into(),
        }
    }
}

#[macro_export]
macro_rules! const_felt252_expr {
    ($low:expr, $high:expr) => {
        Felt252Const::new_const(($low, $high).into()).into()
    };
}

#[macro_export]
macro_rules! felt252_expr {
    ($name:expr, $low:expr, $high:expr) => {
        Felt252Expr::new_var($name.to_string(), Some(Felt252::from(($low, $high))), None)
    };

    ($name:expr, $low:expr, $high:expr,  $in_trace:literal) => {
        if $in_trace {
            Felt252Expr::new_var(
                $name.to_string(),
                Some(Felt252::from(($low, $high))),
                Some((0..FELT252_N_WORDS).collect::<Vec<usize>>().as_slice()),
            )
        } else {
            Felt252Expr::new_var($name.to_string(), Some(Felt252::from(($low, $high))), None)
        }
    };
}
