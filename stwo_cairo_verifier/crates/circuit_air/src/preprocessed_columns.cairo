// Preprocessed-column index constants for the multiverifier circuit.
//
// Source of truth: the production registry's layout — `layout_from_component_sizes` over
// the committed registry, rendered by `circuit_params/tests/cairo_consts_test.rs` (run it with
// FIX=1 to regenerate). Columns are sorted by size (ascending, stable on insertion order), so
// these indices change whenever the registry's component sizes change.
//
// The component files reference some IDX symbols under legacy names (e.g.
// `ADD_FLAG_IDX` corresponds to the prover-side column `qm31_ops_add_flag`); the constants
// keep those names to match the ported code.

use stwo_constraint_framework::{INVALID_COLUMN_IDX, PreprocessedColumnIdx};

// === BEGIN GENERATED (see cairo_consts_test.rs; running it with FIX=1 regenerates) ===

pub const NUM_PREPROCESSED_COLUMNS: u32 = 45;

pub const BITWISE_XOR_4_0_IDX: PreprocessedColumnIdx = 0;
pub const BITWISE_XOR_4_1_IDX: PreprocessedColumnIdx = 1;
pub const BITWISE_XOR_4_2_IDX: PreprocessedColumnIdx = 2;

pub const BITWISE_XOR_7_0_IDX: PreprocessedColumnIdx = 3;
pub const BITWISE_XOR_7_1_IDX: PreprocessedColumnIdx = 4;
pub const BITWISE_XOR_7_2_IDX: PreprocessedColumnIdx = 5;

pub const SEQ_16_IDX: PreprocessedColumnIdx = 6;

pub const BITWISE_XOR_8_0_IDX: PreprocessedColumnIdx = 7;
pub const BITWISE_XOR_8_1_IDX: PreprocessedColumnIdx = 8;
pub const BITWISE_XOR_8_2_IDX: PreprocessedColumnIdx = 9;

pub const BITWISE_XOR_9_0_IDX: PreprocessedColumnIdx = 10;
pub const BITWISE_XOR_9_1_IDX: PreprocessedColumnIdx = 11;
pub const BITWISE_XOR_9_2_IDX: PreprocessedColumnIdx = 12;

pub const EQ_IN0_ADDRESS_IDX: PreprocessedColumnIdx = 13;
pub const EQ_IN1_ADDRESS_IDX: PreprocessedColumnIdx = 14;

pub const TRIPLE_XOR_INPUT_ADDR_0_IDX: PreprocessedColumnIdx = 15;
pub const TRIPLE_XOR_INPUT_ADDR_1_IDX: PreprocessedColumnIdx = 16;
pub const TRIPLE_XOR_INPUT_ADDR_2_IDX: PreprocessedColumnIdx = 17;
pub const TRIPLE_XOR_OUTPUT_ADDR_IDX: PreprocessedColumnIdx = 18;
pub const TRIPLE_XOR_MULTIPLICITY_IDX: PreprocessedColumnIdx = 19;

pub const BITWISE_XOR_10_0_IDX: PreprocessedColumnIdx = 20;
pub const BITWISE_XOR_10_1_IDX: PreprocessedColumnIdx = 21;
pub const BITWISE_XOR_10_2_IDX: PreprocessedColumnIdx = 22;

pub const M_31_TO_U_32_INPUT_ADDR_IDX: PreprocessedColumnIdx = 23;
pub const M_31_TO_U_32_OUTPUT_ADDR_IDX: PreprocessedColumnIdx = 24;
pub const M_31_TO_U_32_MULTIPLICITY_IDX: PreprocessedColumnIdx = 25;

// qm31_ops_* (log_size=23). Hand-ported components reference these without the
// `qm31_ops_` prefix and with legacy names (`OP_0/OP_1/DST` for `in0/in1/out`).
pub const ADD_FLAG_IDX: PreprocessedColumnIdx = 26;
pub const SUB_FLAG_IDX: PreprocessedColumnIdx = 27;
pub const MUL_FLAG_IDX: PreprocessedColumnIdx = 28;
pub const POINTWISE_MUL_FLAG_IDX: PreprocessedColumnIdx = 29;
pub const OP_0_ADDR_IDX: PreprocessedColumnIdx = 30;
pub const OP_1_ADDR_IDX: PreprocessedColumnIdx = 31;
pub const DST_ADDR_IDX: PreprocessedColumnIdx = 32;
pub const QM_31_OPS_MULTIPLICITY_IDX: PreprocessedColumnIdx = 33;

pub const BLAKE_G_GATE_INPUT_ADDR_A_IDX: PreprocessedColumnIdx = 34;
pub const BLAKE_G_GATE_INPUT_ADDR_B_IDX: PreprocessedColumnIdx = 35;
pub const BLAKE_G_GATE_INPUT_ADDR_C_IDX: PreprocessedColumnIdx = 36;
pub const BLAKE_G_GATE_INPUT_ADDR_D_IDX: PreprocessedColumnIdx = 37;
pub const BLAKE_G_GATE_INPUT_ADDR_F_0_IDX: PreprocessedColumnIdx = 38;
pub const BLAKE_G_GATE_INPUT_ADDR_F_1_IDX: PreprocessedColumnIdx = 39;
pub const BLAKE_G_GATE_OUTPUT_ADDR_A_IDX: PreprocessedColumnIdx = 40;
pub const BLAKE_G_GATE_OUTPUT_ADDR_B_IDX: PreprocessedColumnIdx = 41;
pub const BLAKE_G_GATE_OUTPUT_ADDR_C_IDX: PreprocessedColumnIdx = 42;
pub const BLAKE_G_GATE_OUTPUT_ADDR_D_IDX: PreprocessedColumnIdx = 43;
pub const BLAKE_G_GATE_MULTIPLICITY_IDX: PreprocessedColumnIdx = 44;

// === END GENERATED ===

// Make sure INVALID_COLUMN_IDX is not the ID of any column
const INVALID_IDX_CHECK: () = if NUM_PREPROCESSED_COLUMNS >= INVALID_COLUMN_IDX {
    core::panic_with_felt252('invalid idx too small')
};

/// Maps a `log_size` to the index of the corresponding `seq_<log_size>` preprocessed
/// column. Only sizes used by the privacy recursive circuit are supported.
pub fn seq_column_idx(log_size: u32) -> PreprocessedColumnIdx {
    match log_size {
        16 => SEQ_16_IDX,
        _ => panic!("unsupported seq column log_size"),
    }
}
