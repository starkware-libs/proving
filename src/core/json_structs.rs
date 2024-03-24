use std::collections::BTreeMap;
use std::fmt::Display;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AirFnEntry {
    pub name: String,
    pub inst_def: BTreeMap<String, String>,
    pub input: (ProcessedAirVar, bool),
    pub output: (ProcessedAirVar, bool),
    pub air_body: Vec<AirBodyComponent>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessedAirVar {
    Struct {
        id: String,
        fields: Vec<(String, ProcessedAirVar)>,
    },
    Call(String, Vec<ProcessedAirVar>),
    VarExpr(String, String),
    BinaryExpr(Box<ProcessedAirVar>, BinaryOp, Box<ProcessedAirVar>),
    UnaryExpr(UnaryOp, Box<ProcessedAirVar>),
    Tuple(Vec<ProcessedAirVar>),
    Array(Vec<ProcessedAirVar>),
}
impl Display for ProcessedAirVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessedAirVar::Struct { id, fields } => {
                write!(f, "{} {{", id)?;
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", field.0, field.1)?;
                }
                write!(f, "}}")?;
                Ok(())
            }
            ProcessedAirVar::Call(id, args) => {
                write!(f, "{}(", id)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
            ProcessedAirVar::VarExpr(_, id) => write!(f, "{}", id),
            ProcessedAirVar::BinaryExpr(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            ProcessedAirVar::UnaryExpr(op, expr) => write!(f, "({}({}))", op, expr),
            ProcessedAirVar::Tuple(exprs) => {
                write!(f, "(")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, ")")
            }
            ProcessedAirVar::Array(exprs) => {
                write!(f, "[")?;
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", expr)?;
                }
                write!(f, "]")
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessedFeltExpr {
    VarExpr(String),
    BinaryExpr(Box<ProcessedFeltExpr>, BinaryOp, Box<ProcessedFeltExpr>),
    UnaryExpr(UnaryOp, Box<ProcessedFeltExpr>),
}
impl Display for ProcessedFeltExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessedFeltExpr::VarExpr(id) => write!(f, "{}", id),
            ProcessedFeltExpr::BinaryExpr(lhs, op, rhs) => write!(f, "({} {} {})", lhs, op, rhs),
            ProcessedFeltExpr::UnaryExpr(op, expr) => write!(f, "({}({}))", op, expr),
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
}
impl Display for BinaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "//",
            BinaryOp::Eq => "==",
        };
        write!(f, "{}", op)
    }
}

#[derive(Copy, Clone, Default, Debug, Eq, PartialEq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum UnaryOp {
    #[default]
    AsFelt,
}
impl Display for UnaryOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let op = match self {
            UnaryOp::AsFelt => "as_felt",
        };
        write!(f, "{}", op)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Constraint {
    pub expr: ProcessedFeltExpr,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DeductionRule {
    pub var: ProcessedAirVar,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Call {
    pub name: String,
    pub input_arg: String,
    pub output_arg: String,
}

// TODO add local vars
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AirBodyComponent {
    Constraint(Constraint),
    DeductionRule(DeductionRule),
    Subroutine(Call),
}
