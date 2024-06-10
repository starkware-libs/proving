use std::fmt::Display;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::bool_expr::*;
use super::expr::*;
use super::felt_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
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
            OpType::Op(op) => format!("({} {} {})", left, op, right),
            OpType::Method(op) => format!("({}.{}({}))", left, op, right),
            OpType::Static(op) => format!("({}({}, {}))", op, left, right),
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
            BinaryOp::Eq => write!(f, "eq"),
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
        match op {
            BinaryOp::Eq => OpType::Method(op.to_string()),
            // Currently, the rest of the binary operations are represented as operators.
            _ => OpType::Op(op.to_string()),
        }
    }
}

/// Unary expressions - results of unary operations on expressions.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UnaryExpr<T>
where
    T: ProverType,
{
    pub(super) name: String,
    #[allow(unused)]
    #[serde(skip)]
    pub(super) value: Option<T>,
    #[serde(skip)]
    pub(super) child: Box<ExprImpl>,
    #[serde(skip)]
    pub(super) op: UnaryOp,
}

impl<T> UnaryExpr<T>
where
    T: ProverType,
{
    #[allow(unused)]
    pub(super) fn new(op: UnaryOp, child: ExprImpl, value: Option<T>) -> Self {
        let name = match op.into() {
            OpType::Op(op) => format!("({}{})", op, child),
            OpType::Method(op) => format!("({}.{}())", child, op),
            OpType::Static(op) => format!("({}({}))", op, child),
        };

        UnaryExpr {
            name,
            value,
            child: Box::new(child),
            op,
        }
    }
}

impl<T> From<UnaryExpr<T>> for ProcessedAirVar
where
    T: ProverType,
{
    fn from(expr: UnaryExpr<T>) -> ProcessedAirVar {
        match expr.op.into() {
            OpType::Op(op) => ProcessedAirVar::UnaryOp(op, Box::new((*expr.child).into())),
            OpType::Method(op) => {
                ProcessedAirVar::MethodCall(Box::new((*expr.child).into()), op, vec![])
            }
            OpType::Static(op) => ProcessedAirVar::StaticCall(op, vec![(*expr.child).into()]),
        }
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnaryOp {
    #[default]
    Neg,
}
impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
        }
    }
}
impl From<UnaryOp> for OpType {
    fn from(op: UnaryOp) -> OpType {
        match op {
            UnaryOp::Neg => OpType::Op(op.to_string()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpType {
    Op(String),
    Method(String),
    Static(String),
}

impl_binary_op!(Eq, eq, BoolExpr, BoolExpr, BoolBinary);

impl_binary_op!(ops Add, add, FeltExpr, FeltBinary);
impl_binary_op!(ops Sub, sub, FeltExpr, FeltBinary);
impl_binary_op!(ops Mul, mul, FeltExpr, FeltBinary);
impl_binary_op!(ops Div, div, FeltExpr, FeltBinary);
impl_binary_op!(Eq, eq, FeltExpr, BoolExpr, BoolBinary);

impl_binary_op!(ops Add, add, UInt16Expr, UInt16Binary);
impl_binary_op!(ops Rem, rem, UInt16Expr, UInt16Binary);
impl_binary_op!(ops Shl, shl, UInt16Expr, UInt16Binary);
impl_binary_op!(ops Shr, shr, UInt16Expr, UInt16Binary);
impl_binary_op!(ops BitAnd, bitand, UInt16Expr, UInt16Binary);
impl_binary_op!(ops BitOr, bitor, UInt16Expr, UInt16Binary);
impl_binary_op!(ops BitXor, bitxor, UInt16Expr, UInt16Binary);
impl_binary_op!(Eq, eq, UInt16Expr, BoolExpr, BoolBinary);

impl_binary_op!(ops Add, add, UInt32Expr, UInt32Binary);
impl_binary_op!(ops Rem, rem, UInt32Expr, UInt32Binary);
impl_binary_op!(ops Shl, shl, UInt32Expr, UInt32Binary);
impl_binary_op!(ops Shr, shr, UInt32Expr, UInt32Binary);
impl_binary_op!(ops BitAnd, bitand, UInt32Expr, UInt32Binary);
impl_binary_op!(ops BitOr, bitor, UInt32Expr, UInt32Binary);
impl_binary_op!(ops BitXor, bitxor, UInt32Expr, UInt32Binary);
impl_binary_op!(Eq, eq, UInt32Expr, BoolExpr, BoolBinary);

impl_binary_op!(ops Add, add, UInt64Expr, UInt64Binary);
impl_binary_op!(ops Rem, rem, UInt64Expr, UInt64Binary);
impl_binary_op!(ops Shl, shl, UInt64Expr, UInt64Binary);
impl_binary_op!(ops Shr, shr, UInt64Expr, UInt64Binary);
impl_binary_op!(ops BitAnd, bitand, UInt64Expr, UInt64Binary);
impl_binary_op!(ops BitOr, bitor, UInt64Expr, UInt64Binary);
impl_binary_op!(ops BitXor, bitxor, UInt64Expr, UInt64Binary);
impl_binary_op!(Eq, eq, UInt64Expr, BoolExpr, BoolBinary);

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
                    .map(|(l, r)| l.$op_lower(&r).into());
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

#[macro_export]
macro_rules! impl_unary_op {
    ($op:ident, $name:ident, $op_lower:ident, $it:ident, $ot:ident) => {
        impl $it {
            pub fn $name(self) -> $ot {
                let value = self.value().map(|c| c.$op_lower());
                $ot::Unary(UnaryExpr::new(UnaryOp::$op, self.clone().into(), value))
            }
        }
    };
}
