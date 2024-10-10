use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompiledAirFn {
    pub name: String,
    pub description: String,
    pub input: CompiledAirVar,
    pub output: CompiledAirVar,

    pub row_length: usize,
    pub lookup_relation_uses_count: IndexMap<String, usize>,

    // TODO: remove these:
    pub input_num_of_felts: usize,
    pub output_num_of_felts: usize,

    pub constraints: Vec<ConstraintEvalStep>,
    pub deductions: Vec<TraceGenStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraceGenStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock,

    Deduction(CompiledAirVar),

    Intermediate(String, CompiledAirVar),

    // Deduces the output and updates inputs / multiplicity of the component.
    LookupCall {
        fn_name: String,
        input: CompiledAirVar,
        // output_name is the name of the intermediate variable into which the lookup result should
        // be placed. If it is None, there is no output and no intermediate variable is created.
        output_name: Option<String>,
    },

    // Saves the information from the trace needed for the generation of the interaction trace.
    LookupData(LookupData),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstraintEvalStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock,

    // The argument is a polynomial in in-state values. The constraint requires it
    // to evaluate to zero.
    Constraint(CompiledAirVar),

    // Used to create the constraints between the trace and the interaction trace, and the
    // constraints on the accumulated sum (the logup).
    LookupData(LookupData),

    Intermediate(String, CompiledAirVar),
}

// Air variables as represented in the deductions and constrains lists.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum CompiledAirVar {
    // A constant expression represented by the type of the constant and its value.
    Const(String, String),
    // A variable expression represented by the type of the variable and its name.
    Var(String, String),
    // A variable written to the trace at the given index.
    State(usize),
    // A static function call. The name of the function and its arguments.
    StaticCall(String, Vec<CompiledAirVar>),
    // A method function call. The self variable, the name of the method and its arguments.
    MethodCall(Box<CompiledAirVar>, String, Vec<CompiledAirVar>),
    // A binary operation represented by the left-hand side, the operator, and the right-hand
    // side.
    BinaryOp(Box<CompiledAirVar>, String, Box<CompiledAirVar>),
    // A unary operation represented by the operator and the expression.
    UnaryOp(String, Box<CompiledAirVar>),
    Tuple(Vec<CompiledAirVar>),
    Array(Vec<CompiledAirVar>),
    Struct {
        r#type: String,
        fields: Vec<(String, CompiledAirVar)>,
    },
    // A variable written to the trace of a const table at the given index.
    ExternalState(String, usize),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct LookupData {
    pub relation_name: String,
    pub felts: Vec<CompiledAirVar>,
    pub use_or_yield: UseOrYield,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UseOrYield {
    Use,
    Yield,
}
