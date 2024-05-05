use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct AutogenLists {
    pub input: ProcessedAirVar,
    #[serde(skip)]
    pub constraints: Vec<ConstraintOrIntermediate>,
    #[serde(skip)]
    pub deductions: Vec<DeductionOrIntermediate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DeductionOrIntermediate {
    Deduction(ProcessedAirVar),
    Intermediate(String, ProcessedAirVar),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstraintOrIntermediate {
    Constraint(ProcessedAirVar),
    Intermediate(String, ProcessedAirVar),
}

// Air variables as represented in the deductions and constrains lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ProcessedAirVar {
    // A constant expression represented by the type of the constant and its value.
    Const(String, String),
    // A variable expression represented by the type of the variable and its name.
    Var(String, String),
    // A variable written to the trace at the given index.
    State(usize),
    // A static function call. The name of the function and its arguments.
    StaticCall(String, Vec<ProcessedAirVar>),
    // A method function call. The self variable, the name of the method and its arguments.
    MethodCall(Box<ProcessedAirVar>, String, Vec<ProcessedAirVar>),
    // A binary operation represented by the left-hand side, the operator, and the right-hand
    // side.
    BinaryOp(Box<ProcessedAirVar>, String, Box<ProcessedAirVar>),
    // A unary operation represented by the operator and the expression.
    UnaryOp(String, Box<ProcessedAirVar>),
    Tuple(Vec<ProcessedAirVar>),
    Array(Vec<ProcessedAirVar>),
}
