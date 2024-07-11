use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct AutogenLists {
    pub input: ProcessedAirVar,
    pub output: ProcessedAirVar,

    // The input_num_of_felts is relevant just for non-inline components.
    #[serde(skip)]
    pub input_num_of_felts: usize,
    #[serde(skip)]
    pub output_felts: Vec<ProcessedAirVar>,

    #[serde(skip)]
    pub constraints: Vec<ConstraintOrIntermediate>,
    #[serde(skip)]
    pub deductions: Vec<TraceGenerationStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraceGenerationStep {
    Deduction(ProcessedAirVar),
    Intermediate(String, ProcessedAirVar),

    // output_name is the name of the intermediate variable into which the lookup result should
    // be placed
    Lookup {
        fn_name: String,
        input: ProcessedAirVar,
        output_name: String,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstraintOrIntermediate {
    // The argument is a polynomial in in-state values. The constraint requires it
    // to evaluate to zero.
    InInstanceConstraint(ProcessedAirVar),

    // Require a certain input-output pair to be present in a lookup component.
    LookupConstraint {
        fn_name: String,
        input_felts: Vec<ProcessedAirVar>,
        output_felts: Vec<ProcessedAirVar>,
    },

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
