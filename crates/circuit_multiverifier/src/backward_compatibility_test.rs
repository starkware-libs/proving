use circuit_common::N_RESERVED;
use circuit_common::finalize::pad_to_targets;
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_verifier::statement::circuit_verifier_proof_config;
use itertools::chain;
use stwo::core::fields::qm31::QM31;

use crate::test_utils::{
    MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH, MULTIVERIFIER_PREPROCESSED_ROOT, PCS_CONFIG,
    PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES, PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT,
    TARGET_PADDING_SIZES, leaf_circuit_hash, multiverifier_preprocessed_column_log_sizes,
    native_blake_u32s,
};
use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};

/// The preprocessed root of the old Cairo verifier circuit.
const BACKWARD_COMPATIBILITY_CAIRO_VERIFIER_PREPROCESSED_ROOT: [u32; 8] =
    [695775592, 831947430, 3864682957, 1778749033, 1073148880, 3248306553, 1968525874, 3767461582];
/// The output values of the old Cairo verifier circuit.
const BACKWARD_COMPATIBILITY_CAIRO_VERIFIER_OUTPUT_VALUES: [u32; 8] =
    [3035180123, 3555538090, 587798257, 1881776298, 3385462846, 2102605012, 3369268656, 403460632];
/// An older Cairo verifier proof (generated in #648).
const BACKWARD_COMPATIBILITY_CAIRO_PROOF_PATH: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../test_data/circuit_multiverifier/backward_compatibility_cairo_proof.bin"
);

/// Backward-compatibility regression test.
///
/// Checks that the *current* multiverifier can verify, in the same circuit, both a Cairo verifier
/// proof produced by an older version of the prover/circuit.
/// in slot 0, and a multiverifier proof produced by the current version in slot 1.
#[test]
fn test_backward_compatibility() {
    let preprocessed_column_log_sizes = multiverifier_preprocessed_column_log_sizes();
    let shared_config = SharedConfig {
        pcs_config: PCS_CONFIG,
        proof_config: circuit_verifier_proof_config(&preprocessed_column_log_sizes, &PCS_CONFIG),
        preprocessed_column_log_sizes,
    };

    // Slot 0: the older Cairo verifier proof (backward-compatibility fixture), deserialized with
    // the current proof config.
    let old_bytes = std::fs::read(BACKWARD_COMPATIBILITY_CAIRO_PROOF_PATH).unwrap();
    let old_cairo_proof =
        deserialize_proof_with_config(&mut old_bytes.as_slice(), &shared_config.proof_config)
            .unwrap();
    let old_input = MultiverifierInput {
        proof: old_cairo_proof,
        output_values: BACKWARD_COMPATIBILITY_CAIRO_VERIFIER_OUTPUT_VALUES,
        preprocessed_root: BACKWARD_COMPATIBILITY_CAIRO_VERIFIER_PREPROCESSED_ROOT.into(),
    };

    // Slot 1: the current multiverifier proof. Its inputs are hashed as
    // `circuit_hash(preprocessed_root) || output_values`.
    let cairo_input_preimage_words: Vec<u32> =
        leaf_circuit_hash(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT.into(), &shared_config)
            .into_iter()
            .chain(PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES)
            .collect();
    let payload_words: Vec<u32> =
        chain!(&cairo_input_preimage_words, &cairo_input_preimage_words).copied().collect();
    let current_output_values: [u32; N_RESERVED] = native_blake_u32s(&payload_words);

    let current_bytes = std::fs::read(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
    let current_proof =
        deserialize_proof_with_config(&mut current_bytes.as_slice(), &shared_config.proof_config)
            .unwrap();
    let current_input = MultiverifierInput {
        proof: current_proof,
        output_values: current_output_values,
        preprocessed_root: MULTIVERIFIER_PREPROCESSED_ROOT.into(),
    };

    // Verifying both the old Cairo verifier proof and the current multiverifier proof inside a
    // current multiverifier must succeed.
    let mut context = build_multiverifier_circuit::<QM31>(old_input, current_input, &shared_config);
    pad_to_targets(&mut context, TARGET_PADDING_SIZES);
    context.validate_circuit();
}
