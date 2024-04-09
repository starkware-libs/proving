use std::fmt::Display;
use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::expr::*;
use super::felt_expr::*;
// Macros
use crate::impl_binary_op;

/// Binary expressions - results of binary operations on expressions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct BinaryExpr<T>
where
    T: ProverType,
{
    pub(super) name: String,
    #[serde(skip)]
    pub(super) value: Option<T>,
    #[serde(skip)]
    pub(super) left: Box<ExprImpl>,
    #[serde(skip)]
    pub(super) right: Box<ExprImpl>,
    #[serde(skip)]
    pub(super) op: BinaryOp,
}

impl<T> BinaryExpr<T>
where
    T: ProverType,
{
    pub fn new(left: ExprImpl, op: BinaryOp, right: ExprImpl, value: Option<T>) -> Self {
        let name = match op.into() {
            OpType::Op(op) => format!("({} {} {})", left.name(), op, right.name()),
            OpType::Method(op) => format!("({}.{}({}))", left.name(), op, right.name()),
            OpType::Static(op) => format!("({}({}, {}))", op, left.name(), right.name()),
        };

        BinaryExpr {
            name,
            value,
            left: Box::new(left),
            right: Box::new(right),
            op,
        }
    }
}

impl<T> From<BinaryExpr<T>> for ProcessedAirVar
where
    T: ProverType,
{
    fn from(expr: BinaryExpr<T>) -> ProcessedAirVar {
        match expr.op.into() {
            OpType::Op(op) => ProcessedAirVar::BinaryOp(
                Box::new((*expr.left).into()),
                op,
                Box::new((*expr.right).into()),
            ),
            OpType::Method(op) => ProcessedAirVar::MethodCall(
                Box::new((*expr.left).into()),
                op,
                vec![(*expr.right).into()],
            ),
            OpType::Static(op) => {
                ProcessedAirVar::StaticCall(op, vec![(*expr.left).into(), (*expr.right).into()])
            }
        }
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum BinaryOp {
    #[default]
    Add,
    Sub,
    Mul,
    Div,
    Eq,
    Rem,
    Shl,
    Shr,
    BitAnd,
    BitOr,
    BitXor,
}
impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryOp::Add => write!(f, "+"),
            BinaryOp::Sub => write!(f, "-"),
            BinaryOp::Mul => write!(f, "*"),
            BinaryOp::Div => write!(f, "//"),
            BinaryOp::Eq => write!(f, "=="),
            BinaryOp::Rem => write!(f, "%"),
            BinaryOp::Shl => write!(f, "<<"),
            BinaryOp::Shr => write!(f, ">>"),
            BinaryOp::BitAnd => write!(f, "&"),
            BinaryOp::BitOr => write!(f, "|"),
            BinaryOp::BitXor => write!(f, "^"),
        }
    }
}
impl From<BinaryOp> for OpType {
    fn from(op: BinaryOp) -> OpType {
        // Currently, all binary operations are represented as operators.
        OpType::Op(op.to_string())
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    Op(String),
    Method(String),
    Static(String),
}

impl_binary_op!(ops Add, add, FeltExpr, FeltBinary);
impl_binary_op!(ops Sub, sub, FeltExpr, FeltBinary);
impl_binary_op!(ops Mul, mul, FeltExpr, FeltBinary);
impl_binary_op!(ops Div, div, FeltExpr, FeltBinary);

#[macro_export]
macro_rules! impl_binary_op {
    (ops $op:ident, $op_lower:ident, $t:ident, $b:ident) => {
        impl $op for &$t {
            type Output = $t;
            fn $op_lower(self, other: &$t) -> $t {
                let value = self.value().zip(other.value()).map(|(l, r)| l.$op_lower(r));
                $t::Binary($b::new(
                    self.clone().into(),
                    BinaryOp::$op,
                    other.clone().into(),
                    value,
                ))
            }
        }
    };

    ($op:ident, $op_lower:ident, $it:ident, $ot:ident, $b:ident) => {
        impl $it {
            pub fn $op_lower(self, other: $it) -> $ot {
                let value = self
                    .value()
                    .zip(other.value())
                    .map(|(l, r)| l.$op_lower(&r));
                $ot::Binary($b::new(
                    self.clone().into(),
                    BinaryOp::$op,
                    other.clone().into(),
                    value,
                ))
            }
        }
    };
}
