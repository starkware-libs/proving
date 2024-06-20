use std::array::from_fn;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

use super::super::air_fn_registry::*;
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

impl Felt252Var {
    // Updates the Felts representation of the variable.
    // Called whenever a variable is created (see new_var and let_for_deduction).
    fn update_parts(&mut self) {
        let mut self_copy = self.clone();
        self_copy.felts = from_fn(|_| FeltExpr::default());
        let parent: ExprImpl = Felt252Expr::Var(self_copy.clone()).into();
        for (index, felt) in self.felts.iter_mut().enumerate() {
            felt.set_parent(parent.clone(), Some(index));
        }
    }
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
    // Creates a new Felt252Var.
    pub fn new_var(
        name: String,
        value: Option<Felt252>,
        state_indices: Option<[usize; FELT252_N_WORDS]>,
    ) -> Self {
        let mut res = Felt252Var {
            name,
            value,
            felts: from_fn(|i| {
                FeltExpr::new_var(
                    "get_felt".to_string(),
                    value.map(|v| v.get_felt(i)),
                    state_indices.map(|is| is[i]),
                )
            }),
        };
        res.update_parts();
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
        assert!(name.starts_with(DEDUCTION_INTERMEDIATE_VAR_PREFIX));

        match self {
            Felt252Expr::Var(v) => {
                let mut res = v.clone();
                res.name = name;
                res.update_parts();
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
            Felt252Expr::Unary(u) => {
                if u.op == UnaryOp::Felt252FromFelt {
                    if let GenericAirVar::Expr(ExprImpl::Felt(felt_expr)) = &mut *u.child {
                        if let FeltExpr::Var(_) = felt_expr {
                            // Should we return FELT252_N_WORDS felts?
                            return vec![felt_expr];
                        }
                    }
                }
                panic!("Cannot convert to felts");
            }
            _ => panic!("Cannot convert to felts"),
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

impl Display for Felt252Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
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
                Some(from_fn(|i| i)),
            )
        } else {
            Felt252Expr::new_var($name.to_string(), Some(Felt252::from(($low, $high))), None)
        }
    };
}
