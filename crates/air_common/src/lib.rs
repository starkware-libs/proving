pub mod utils;

/// This module defines common types and constants used by air_infra and air_compile.
/// The reason for this module is so that both the visualizer and eval_air_fn_constraints will
/// depend on the same structures, as the former used for audits, and the latter used to check
/// the generated code. Since the visualizer is a separate binary, it works on serialized
/// airs, as defined in air_compile, and we don't want it to have access to extra information
/// from air_infra.
use serde::{Deserialize, Serialize};

pub const WRITE_TRACE_FUNCTION_NAME: &str = "write_trace";
pub const CONSTRAINT_EVAL_FUNCTION_NAME: &str = "evaluate";

pub const REGISTRY_PROPERTIES_FILE_NAME: &str = "registry.json";
pub const SAMPLE_EVALUATIONS_FILE_NAME: &str = "sample_evaluations.json";

// These relation names should be taken from stwo-cairo.
pub const OPCODES_RELATION_NAME: &str = "Opcodes";
pub const GATE_RELATION_NAME: &str = "Gate";
pub const MEMORY_RELATION_NAME: &str = "MemoryIdToBig";

// A preprocessed column represented by its id in stwo-cairo. The special
// value "Seq" is used to represent the Seq column whose size equals the size
// of the current component.
pub type ExternalState = String;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PaddingType {
    // The multiplicity column is used to pad the lookups to const columns, memory, and verify
    // instruction.
    Multiplicity,
    // The enabler column is used to pad the chain lookups, as "Opcodes" and "BlakeRound", and
    // every lookup with no multiplicity.
    Enabler,
    // For air functions that are not a component in the trace, as inline air functions.
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TraceType {
    // Doesn't have its own component in the trace, always inlined into its caller.
    // Can be called only with call.
    Inline,

    // Has its own component in the trace. Each call generates a new row in that component.
    // Can be called only with lookup_call. Yields lookup data.
    Component,

    // Has its own component in the trace. The trace for this component is pre-filled with rows
    // for all possible inputs by external means. Doesn't generate deductions or constraints.
    // Has no input, only output. Can be called only with call_external_table. Doesn't yield
    // lookup data.
    Const,

    // Has its own component in the trace. Has no input and no output. Cannot be called from
    // another component. Doesn't yield lookup data.
    Builtin,

    // Has its own component in the trace. Its input and output are casm states.
    // Cannot be called from another component. Doesn't yield multiplicity column.
    // Generates accumulated sum column where the input
    // is used and the output is yielded (chain lookup constraint).
    // Their chain lookup relation is called OPCODES_RELATION_NAME.
    Opcode,

    // Memory components are pre-filled. Their trace consists of only input and output columns, or
    // only output columns, if the input is const. They don't generate deductions. They can
    // generate constraints, and they yield lookup data. They implement the IsMemory trait.
    Memory,

    // Has its own component in the trace. Its input and output are of the same type ([FeltExpr;
    // 2], S), where S is some AirVar. Doesn't yield multiplicity column.
    // Generates accumulated sum column where the input
    // is used and the output is yielded (chain lookup constraint).
    //
    // Important:
    // - A ChainRound can be called from a single caller. This is because we use the caller Seq
    //   column to identify the chain (see chain_lookup_call).
    // - A ChainRound must have consts per round that are returned from a lookup component with a
    //   const round number column in its external input. Without this the chain lookup is not
    //   sound (for example, a malicious prover can run for more rounds than intended by
    //   overflowing the round number).
    ChainRound,

    // Used by the circuit airs for AirFns that are separate components, but manage their uses /
    // yields manually.
    Gate,
    // Relation that doesn't have a table in the trace, but can be used for lookups.
    Relation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum UseOrYield {
    Use,
    Yield,
}
