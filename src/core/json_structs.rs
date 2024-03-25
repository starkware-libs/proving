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
    #[serde(skip)]
    pub constraints: Vec<ProcessedFeltExpr>,
    #[serde(skip)]
    pub deductions: Vec<ProcessedAirVar>,
}

// Air variables as represented in the deductions list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessedAirVar {
    Struct {
        // The name of the struct.
        id: String,
        // The fields of the struct. Each field is a tuple of the field type and the field name.
        fields: Vec<(String, String)>,
    },
    // A function call. The name of the function and its arguments.
    Call(String, Vec<ProcessedAirVar>),
    // A constant expression represented by the type of the constant and its value.
    ConstExpr(String, String),
    // A variable expression represented by the type of the variable and its name.
    VarExpr(String, String),
    // A binary expression represented by the left-hand side, the operator, and the right-hand side.
    BinaryExpr(Box<ProcessedAirVar>, String, Box<ProcessedAirVar>),
    // A unary expression represented by the operator and the expression.
    UnaryExpr(String, Box<ProcessedAirVar>),
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
            ProcessedAirVar::ConstExpr(_, id) => write!(f, "const_{}", id),
            ProcessedAirVar::VarExpr(_, id) => write!(f, "{}", id),
            ProcessedAirVar::BinaryExpr(lhs, op, rhs) => write!(f, "({}.{}({}))", lhs, op, rhs),
            ProcessedAirVar::UnaryExpr(op, expr) => write!(f, "({}.{}())", expr, op),
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

// Expressions as represented in the constraints list.
// All these expressions are written to the trace, so they are already defined with their types as part of the deduction list.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessedFeltExpr {
    // A constant expression represented by the value of the constant.
    ConstExpr(String),
    // A variable expression represented by the name of the variable.
    VarExpr(String),
    // A binary expression represented by the left-hand side, the operator, and the right-hand side.
    BinaryExpr(Box<ProcessedFeltExpr>, String, Box<ProcessedFeltExpr>),
    // A unary expression represented by the operator and the expression.
    UnaryExpr(String, Box<ProcessedFeltExpr>),
}
impl Display for ProcessedFeltExpr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessedFeltExpr::ConstExpr(id) => write!(f, "const_{}", id),
            ProcessedFeltExpr::VarExpr(id) => write!(f, "{}", id),
            ProcessedFeltExpr::BinaryExpr(lhs, op, rhs) => write!(f, "({}.{}({}))", lhs, op, rhs),
            ProcessedFeltExpr::UnaryExpr(op, expr) => write!(f, "({}.{}())", expr, op),
        }
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
