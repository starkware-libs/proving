use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct CompiledAirFn {
    pub name: String,
    pub description: String,
    pub input: CompiledAirVar,
    pub output: CompiledAirVar,

    // The input_num_of_felts is relevant just for non-inline components.
    #[serde(skip)]
    pub input_num_of_felts: usize,
    #[serde(skip)]
    pub output_felts: Vec<CompiledAirVar>,

    #[serde(skip)]
    pub constraints: Vec<ConstraintEvalStep>,
    #[serde(skip)]
    pub deductions: Vec<TraceGenStep>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TraceGenStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock(),

    Deduction(CompiledAirVar),

    Intermediate(String, CompiledAirVar),

    // output_name is the name of the intermediate variable into which the lookup result should
    // be placed. If it is None, there is no output and no intermediate variable is created.
    Lookup {
        fn_name: String,
        input: CompiledAirVar,
        output_name: Option<String>,
    },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ConstraintEvalStep {
    // Constains a description of the following code block.
    StartBlock(String),

    EndBlock(),

    // The argument is a polynomial in in-state values. The constraint requires it
    // to evaluate to zero.
    InInstanceConstraint(CompiledAirVar),

    // Require a certain input-output pair to be present in a lookup component.
    LookupConstraint {
        fn_name: String,
        input_felts: Vec<CompiledAirVar>,
        output_felts: Vec<CompiledAirVar>,
    },

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
