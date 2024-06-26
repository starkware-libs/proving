use std::fmt::Display;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Not, Rem, Shl, Shr, Sub};

use serde::{Deserialize, Serialize};

use super::super::autogen_structs::*;
use super::super::prover_types::*;
use super::super::variables::*;
use super::bool_expr::*;
use super::expr::*;
use super::felt252_expr::*;
use super::felt_expr::*;
use super::uint16_expr::*;
use super::uint32_expr::*;
use super::uint64_expr::*;
// Macros
use crate::impl_binary_op;
use crate::impl_unary_op;

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
    pub(super) left: Box<GenericAirVar>,
    #[serde(skip)]
    pub(super) right: Box<GenericAirVar>,
    #[serde(skip)]
    pub(super) op: BinaryOp,
}

impl<T> BinaryExpr<T>
where
    T: ProverType,
{
    pub fn new(left: GenericAirVar, op: BinaryOp, right: GenericAirVar, value: Option<T>) -> Self {
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

// Note that all operations from the same type should have different names for the code generation.
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
    pub(super) child: Box<GenericAirVar>,
    #[serde(skip)]
    pub(super) op: UnaryOp,
}

impl<T> UnaryExpr<T>
where
    T: ProverType,
{
    #[allow(unused)]
    pub(super) fn new(op: UnaryOp, child: GenericAirVar, value: Option<T>) -> Self {
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
    BoolFromFelt,
    ConstBoolToFelt,
    UInt16FromBool,
    UInt16FromFelt,
    ConstUint16ToFelt,
    Felt252FromFeltsArray,
    Not,
}

// Note that all operations from the same type should have different names for the code generation.
impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UnaryOp::Neg => write!(f, "-"),
            UnaryOp::BoolFromFelt => write!(f, "Bool::from_felt"),
            UnaryOp::ConstBoolToFelt => write!(f, "as_felt"),
            UnaryOp::UInt16FromBool => write!(f, "UInt16::from_bool"),
            UnaryOp::UInt16FromFelt => write!(f, "UInt16::from_felt"),
            UnaryOp::ConstUint16ToFelt => write!(f, "as_felt"),
            UnaryOp::Felt252FromFeltsArray => write!(f, "Felt252::from_felts"),
            UnaryOp::Not => write!(f, "!"),
        }
    }
}
impl From<UnaryOp> for OpType {
    fn from(op: UnaryOp) -> OpType {
        match op {
            UnaryOp::Neg => OpType::Op(op.to_string()),
            UnaryOp::BoolFromFelt => OpType::Static(op.to_string()),
            UnaryOp::ConstBoolToFelt => OpType::Method(op.to_string()),
            UnaryOp::UInt16FromBool => OpType::Static(op.to_string()),
            UnaryOp::UInt16FromFelt => OpType::Static(op.to_string()),
            UnaryOp::ConstUint16ToFelt => OpType::Method(op.to_string()),
            UnaryOp::Felt252FromFeltsArray => OpType::Static(op.to_string()),
            UnaryOp::Not => OpType::Op(op.to_string()),
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
impl_unary_op!(from UInt16FromBool, from_bool, BoolExpr, UInt16Expr, UInt16);
impl_unary_op!(ops Not, not, BoolExpr);

impl_binary_op!(ops Add, add, FeltExpr, FeltBinary);
impl_binary_op!(ops Sub, sub, FeltExpr, FeltBinary);
impl_binary_op!(ops Mul, mul, FeltExpr, FeltBinary);
impl_binary_op!(ops Div, div, FeltExpr, FeltBinary);
impl_binary_op!(Eq, eq, FeltExpr, BoolExpr, BoolBinary);
impl_unary_op!(from BoolFromFelt, from_felt, FeltExpr, BoolExpr, Bool);
impl_unary_op!(from UInt16FromFelt, from_felt, FeltExpr, UInt16Expr, UInt16);

impl_binary_op!(ops Add, add, UInt16Expr, UInt16Binary);
impl_binary_op!(ops Sub, sub, UInt16Expr, UInt16Binary);
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

impl From<Vec<FeltExpr>> for Felt252Expr {
    fn from(felts: Vec<FeltExpr>) -> Felt252Expr {
        assert!(
            felts.len() <= FELT252_N_WORDS,
            "Felt252Expr can have at most {FELT252_N_WORDS} felts"
        );

        let values = felts
            .iter()
            .filter_map(|f| f.value())
            .collect::<Vec<Felt>>();
        let value = if values.len() == felts.len() {
            Some(Felt252::from_felts(values))
        } else {
            None
        };

        let arr = felts
            .into_iter()
            .map(|f| f.into())
            .collect::<Vec<GenericAirVar>>();
        Felt252Expr::Unary(UnaryExpr::new(
            UnaryOp::Felt252FromFeltsArray,
            GenericAirVar::Array(arr),
            value,
        ))
    }
}

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
    (ops $op:ident, $op_lower:ident, $t:ident) => {
        impl $op for &$t {
            type Output = $t;
            fn $op_lower(self) -> $t {
                let value = self.value().map(|c| c.$op_lower());
                $t::Unary(UnaryExpr::new(UnaryOp::$op, self.clone().into(), value))
            }
        }
    };

    (from $op:ident, $op_lower:ident, $it:ident, $ot:ident, $ot_lower: ident) => {
        impl From<$it> for $ot {
            fn from(input: $it) -> Self {
                let value = input.value().map(|c| $ot_lower::$op_lower(c));
                $ot::Unary(UnaryExpr::new(UnaryOp::$op, input.clone().into(), value))
            }
        }
    };

    ($op:ident, $name:ident, $op_lower:ident, $it:ident, $ot:ident) => {
        impl $it {
            pub fn $name(self) -> $ot {
                let value = self.value().map(|c| c.$op_lower());
                $ot::Unary(UnaryExpr::new(UnaryOp::$op, self.clone().into(), value))
            }
        }
    };

    (static $op:ident, $name:ident, $op_lower:ident, $it:ident, $ot:ident) => {
        impl $it {
            pub fn $name(self) -> $ot {
                let value = self.value().map(|c| $op_lower(c));
                $ot::Unary(UnaryExpr::new(UnaryOp::$op, self.clone().into(), value))
            }
        }
    };
}
