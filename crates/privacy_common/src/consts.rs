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
/// The components used to verify the privacy transaction, in the order of
/// `circuit_cairo_verifier::all_components::all_components`.
pub const PRIVACY_TRANSACTION_COMPONENTS: [&str; 57] = [
    "add_opcode",
    "add_opcode_small",
    "add_ap_opcode",
    "assert_eq_opcode",
    "assert_eq_opcode_imm",
    "assert_eq_opcode_double_deref",
    "blake_compress_opcode",
    "call_opcode_abs",
    "call_opcode_rel_imm",
    "jnz_opcode_non_taken",
    "jnz_opcode_taken",
    "jump_opcode_abs",
    "jump_opcode_double_deref",
    "jump_opcode_rel",
    "jump_opcode_rel_imm",
    "mul_opcode",
    "mul_opcode_small",
    "ret_opcode",
    "verify_instruction",
    "blake_round",
    "blake_g",
    "blake_round_sigma",
    "triple_xor_32",
    "verify_bitwise_xor_12",
    "bitwise_builtin",
    "pedersen_builtin_narrow_windows",
    "poseidon_builtin",
    "range_check_builtin",
    "pedersen_aggregator_window_bits_9",
    "partial_ec_mul_window_bits_9",
    "pedersen_points_table_window_bits_9",
    "poseidon_aggregator",
    "poseidon_3_partial_rounds_chain",
    "poseidon_full_round_chain",
    "cube_252",
    "poseidon_round_keys",
    "range_check_252_width_27",
    "memory_address_to_id",
    "memory_id_to_big",
    "memory_id_to_small",
    "range_check_6",
    "range_check_8",
    "range_check_11",
    "range_check_12",
    "range_check_18",
    "range_check_20",
    "range_check_4_3",
    "range_check_4_4",
    "range_check_9_9",
    "range_check_7_2_5",
    "range_check_3_6_6_3",
    "range_check_4_4_4_4",
    "range_check_3_3_3_3_3",
    "verify_bitwise_xor_4",
    "verify_bitwise_xor_7",
    "verify_bitwise_xor_8",
    "verify_bitwise_xor_9",
];
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

/// The preprocessed columns of the recursive circuit, in size-sorted order (the order of the
/// prover's preprocessed trace): each column's id and the `ilog2` of its number of rows.
pub const PRIVACY_CIRCUIT_PREPROCESSED_COLUMNS: [(&str, u32); 45] = [
    ("bitwise_xor_4_0", 8),
    ("bitwise_xor_4_1", 8),
    ("bitwise_xor_4_2", 8),
    ("bitwise_xor_7_0", 14),
    ("bitwise_xor_7_1", 14),
    ("bitwise_xor_7_2", 14),
    ("seq_16", 16),
    ("bitwise_xor_8_0", 16),
    ("bitwise_xor_8_1", 16),
    ("bitwise_xor_8_2", 16),
    ("eq_in0_address", 17),
    ("eq_in1_address", 17),
    ("triple_xor_input_addr_0", 17),
    ("triple_xor_input_addr_1", 17),
    ("triple_xor_input_addr_2", 17),
    ("triple_xor_output_addr", 17),
    ("triple_xor_multiplicity", 17),
    ("m31_to_u32_input_addr", 18),
    ("m31_to_u32_output_addr", 18),
    ("m31_to_u32_multiplicity", 18),
    ("bitwise_xor_9_0", 18),
    ("bitwise_xor_9_1", 18),
    ("bitwise_xor_9_2", 18),
    ("blake_g_gate_input_addr_a", 20),
    ("blake_g_gate_input_addr_b", 20),
    ("blake_g_gate_input_addr_c", 20),
    ("blake_g_gate_input_addr_d", 20),
    ("blake_g_gate_input_addr_f0", 20),
    ("blake_g_gate_input_addr_f1", 20),
    ("blake_g_gate_output_addr_a", 20),
    ("blake_g_gate_output_addr_b", 20),
    ("blake_g_gate_output_addr_c", 20),
    ("blake_g_gate_output_addr_d", 20),
    ("blake_g_gate_multiplicity", 20),
    ("bitwise_xor_10_0", 20),
    ("bitwise_xor_10_1", 20),
    ("bitwise_xor_10_2", 20),
    ("qm_31_ops_add_flag", 21),
    ("qm_31_ops_sub_flag", 21),
    ("qm_31_ops_mul_flag", 21),
    ("qm_31_ops_pointwise_mul_flag", 21),
    ("qm_31_ops_in_0_address", 21),
    ("qm_31_ops_in_1_address", 21),
    ("qm_31_ops_out_address", 21),
    ("qm_31_ops_mults", 21),
];
