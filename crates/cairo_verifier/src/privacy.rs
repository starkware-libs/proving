use std::collections::HashSet;

use cairo_air::verifier::INTERACTION_POW_BITS;
use circuits::ivalue::NoValue;
use circuits_stark_verifier::constraint_eval::CircuitEval;
use circuits_stark_verifier::proof::ProofConfig;
use indexmap::IndexMap;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;

use crate::all_components::all_components;
use crate::utils::{get_test_data_dir, load_program};
use crate::verify::{CairoVerifierConfig, get_preprocessed_root};

#[cfg(test)]
#[path = "privacy_test.rs"]
pub mod test;

/// The log size of the trace produced by the privacy setup. It is equal to the log size of the
/// [`PreProcessedTraceVariant::CanonicalSmall`] preprocessed trace.
const PRIVACY_CAIRO_TRACE_LOG_SIZE: u32 = 20;

/// Returns a [CairoVerifierConfig] for the privacy proof setup with the given log blowup factor.
pub fn privacy_cairo_verifier_config(log_blowup_factor: u32) -> CairoVerifierConfig {
    let preprocessed_trace_variant = PreProcessedTraceVariant::CanonicalSmall;
    let privacy_set = privacy_components();
    // Build `enabled_bits` (one flag per component in the full list) and `components` (only the
    // enabled entries, as expected by `ProofConfig::new`) in a single pass.
    let (enabled_bits, components): (Vec<bool>, Vec<_>) = all_components::<NoValue>()
        .into_iter()
        .map(|(name, component)| {
            let enabled = privacy_set.contains(name);
            (enabled, enabled.then_some((name, component)))
        })
        .unzip();
    let components: IndexMap<&'static str, Box<dyn CircuitEval<NoValue>>> =
        components.into_iter().flatten().collect();

    // Derive proof config parameters from the log blowup factor, targeting 96-bit security.
    let pcs_config = get_pcs_config(PRIVACY_CAIRO_TRACE_LOG_SIZE, log_blowup_factor);

    let proof_config = ProofConfig::new(
        &components,
        preprocessed_trace_variant.n_columns(),
        &pcs_config,
        INTERACTION_POW_BITS,
    );

    let program_path = get_test_data_dir().join("privacy/privacy_simple_bootloader_compiled.json");
    let program = load_program(&program_path);

    CairoVerifierConfig {
        preprocessed_root: get_preprocessed_root(pcs_config.min_lifting_log_size),
        proof_config,
        enabled_bits,
        program,
        preprocessed_trace_variant,
    }
}

// The set of components that are used to verify the privacy transaction.
// The order of the components is determend by the order in all_components()
// TODO(ilya): Fix the privacy bootloader to use the commented out components.
pub fn privacy_components() -> HashSet<&'static str> {
    HashSet::from_iter([
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
    ])
}

pub const fn get_pcs_config(trace_log_size: u32, log_blowup_factor: u32) -> PcsConfig {
    let (pow_bits, n_queries) = match log_blowup_factor {
        1 => (26, 70),
        2 => (26, 35),
        3 => (27, 23),
        _ => panic!("Unsupported log blowup factor."),
    };
    assert!(
        pow_bits + n_queries as u32 * log_blowup_factor >= 96_u32,
        "The config is not secure enough."
    );
    // Note that `fold_step` is hardcoded to 4.
    let fri_config =
        FriConfig { log_blowup_factor, log_last_layer_degree_bound: 0, n_queries, fold_step: 4 };
    PcsConfig { pow_bits, fri_config, min_lifting_log_size: trace_log_size + log_blowup_factor }
}
