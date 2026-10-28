use stwo::core::fri::{FriConfig, FriParams};
use stwo::core::pcs::PcsConfig;

/// Uncompressed size in bytes of the serialized recursive circuit proof.
pub const RECURSIVE_PROOF_UNCOMPRESSED_BYTES: usize = 263_812;

/// Multiplicative safety factor applied to the proof size constant to derive the decompression
/// limit used in `verify_recursive_circuit` as zip-bomb protection.
pub const PROOF_MAX_DECOMPRESSED_RATIO: usize = 2;

/// Maximum allowed uncompressed size in bytes when decompressing the recursive circuit proof.
/// Used in `verify_recursive_circuit` to prevent zip-bomb attacks.
pub const MAX_RECURSIVE_PROOF_UNCOMPRESSED_BYTES: usize =
    RECURSIVE_PROOF_UNCOMPRESSED_BYTES * PROOF_MAX_DECOMPRESSED_RATIO;

// See comment in privacy_prove::tests::test_{privacy,leaf}_bootloader_program_hash_snapshot
// for more details on how these files were compiled.
pub const PRIVACY_BOOTLOADER_JSON: &[u8] = include_bytes!(
    "../../cairo-program-runner-lib/resources/compiled_programs/bootloaders/\
     privacy_simple_bootloader_compiled.json"
);
pub const LARGE_PROOFS_CIRCUIT_REGISTRY_JSON: &str =
    include_str!("../large_proofs_circuit_registry.json");
pub const CIRCUIT_OUTPUT_ADDRESSES: [usize; 9] = [3, 4, 5, 6, 7, 8, 9, 10, 2];
pub const PRIVACY_RECURSION_CIRCUIT_PREPROCESSED_ROOT: [u32; 8] =
    [3175515145, 1914895669, 4103856623, 930131002, 1591916467, 1883363718, 613962201, 3347954523];
pub const CAIRO_LOG_BLOWUP_FACTOR: u32 = 3;
pub const CAIRO_TRACE_LOG_SIZE: u32 = 20;
pub const CIRCUIT_LOG_BLOWUP_FACTOR: u32 = 2;
pub const CIRCUIT_TRACE_LOG_SIZE: u32 = 21;

pub const CAIRO_FRI_CONFIG: FriConfig = FriConfig {
    params: FriParams {
        pow_bits: 27,
        log_blowup_factor: CAIRO_LOG_BLOWUP_FACTOR,
        log_last_layer_degree_bound: 0,
        n_queries: 23,
        fold_step: 4,
    },
    trace_log_size: CAIRO_TRACE_LOG_SIZE,
};

pub const CAIRO_PCS_CONFIG: PcsConfig = PcsConfig::from_fri_config(CAIRO_FRI_CONFIG);

pub const CIRCUIT_FRI_CONFIG: FriConfig = FriConfig {
    params: FriParams {
        pow_bits: 26,
        log_blowup_factor: CIRCUIT_LOG_BLOWUP_FACTOR,
        log_last_layer_degree_bound: 0,
        n_queries: 35,
        fold_step: 4,
    },
    trace_log_size: CIRCUIT_TRACE_LOG_SIZE,
};

pub const CIRCUIT_PCS_CONFIG: PcsConfig = PcsConfig::from_fri_config(CIRCUIT_FRI_CONFIG);

/// Log sizes (in size-sorted order, matching `PRIVACY_CIRCUIT_PREPROCESSED_IDS`) of the
/// preprocessed columns in the recursive circuit. Each entry is the `ilog2` of the number of
/// rows in the corresponding column.
pub const PRIVACY_CIRCUIT_PREPROCESSED_LOG_SIZES: [u32; 45] = [
    8, 8, 8, // bitwise_xor_4_{0,1,2}: 2^(2*4) = 2^8
    14, 14, 14, //  bitwise_xor_7_{0,1,2}: 2^(2*7) = 2^14
    16, 16, 16, 16, // bitwise_xor_8_{0,1,2}: 2^(2*8) = 2^16
    17, 17, // eq
    17, 17, 17, 17, 17, // triple_xor
    18, 18, 18, // m31_to_u32
    18, 18, 18, // bitwise_xor_9_{0,1,2}: 2^(2*9) = 2^18
    20, 20, 20, 20, 20, 20, 20, 20, 20, 20, 20, // blake_g_gate
    20, 20, 20, // bitwise_xor_10_{0,1,2}: 2^(2*10) = 2^20
    21, 21, 21, 21, 21, 21, 21, 21, // qm31 ops
];

pub const PRIVACY_CIRCUIT_PREPROCESSED_IDS: [&str; 45] = [
    "bitwise_xor_4_0",
    "bitwise_xor_4_1",
    "bitwise_xor_4_2",
    "bitwise_xor_7_0",
    "bitwise_xor_7_1",
    "bitwise_xor_7_2",
    "seq_16",
    "bitwise_xor_8_0",
    "bitwise_xor_8_1",
    "bitwise_xor_8_2",
    "eq_in0_address",
    "eq_in1_address",
    "triple_xor_input_addr_0",
    "triple_xor_input_addr_1",
    "triple_xor_input_addr_2",
    "triple_xor_output_addr",
    "triple_xor_multiplicity",
    "m31_to_u32_input_addr",
    "m31_to_u32_output_addr",
    "m31_to_u32_multiplicity",
    "bitwise_xor_9_0",
    "bitwise_xor_9_1",
    "bitwise_xor_9_2",
    "blake_g_gate_input_addr_a",
    "blake_g_gate_input_addr_b",
    "blake_g_gate_input_addr_c",
    "blake_g_gate_input_addr_d",
    "blake_g_gate_input_addr_f0",
    "blake_g_gate_input_addr_f1",
    "blake_g_gate_output_addr_a",
    "blake_g_gate_output_addr_b",
    "blake_g_gate_output_addr_c",
    "blake_g_gate_output_addr_d",
    "blake_g_gate_multiplicity",
    "bitwise_xor_10_0",
    "bitwise_xor_10_1",
    "bitwise_xor_10_2",
    "qm_31_ops_add_flag",
    "qm_31_ops_sub_flag",
    "qm_31_ops_mul_flag",
    "qm_31_ops_pointwise_mul_flag",
    "qm_31_ops_in_0_address",
    "qm_31_ops_in_1_address",
    "qm_31_ops_out_address",
    "qm_31_ops_mults",
];
