use std::fs::File;
use std::io::Write;

use cairo_air::utils::binary_deserialize_from_file;
use circuit_cairo_verifier::privacy::{privacy_cairo_verifier_config, privacy_components};
use circuit_cairo_verifier::utils::get_proof_file_path;
use circuit_cairo_verifier::verify::{
    build_cairo_verifier_circuit, verify_cairo_with_component_set,
};
use circuit_common::N_RESERVED;
use circuit_common::finalize::{ComponentSizes, compute_padded_sizes, pad_to_targets};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    prepare_circuit_proof_for_circuit_verifier, prove_circuit_assignment,
};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_serialize::serialize::CircuitSerialize;
use circuit_verifier::statement::circuit_verifier_proof_config;
use circuit_verifier::verify::CircuitPublicData;
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::{IValue, NoValue};
use circuits::utils::le_u32s_from_bytes;
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use itertools::chain;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;

use crate::test_utils::{
    CIRCUIT_N_PREPROCESSED_COLUMNS, LOG_BLOWUP_FACTOR, MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH,
    MULTIVERIFIER_PREPROCESSED_ROOT, PCS_CONFIG, PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES,
    PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT, TARGET_PADDING_SIZES,
    get_preprocessed_multiverifier_from_circuit, leaf_circuit_hash,
    multiverifier_preprocessed_column_log_sizes, native_blake_u32s,
};
use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};

/// The Cairo verifier proof fixture (produced by [`test_cairo_proof_regression`]).
const PRIVACY_CAIRO_VERIFIER_PROOF_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/circuit_multiverifier/proof_cairo.bin");

/// Builds the `ProofConfig` for the proofs of the inner verifiers.
fn inner_verifier_proof_config() -> ProofConfig {
    circuit_verifier_proof_config(&multiverifier_preprocessed_column_log_sizes(), &PCS_CONFIG)
}

/// Builds a `NoValue` Cairo verifier circuit (with configs of privacy) and preprocesses it.
///
/// If `target_padding` is `None`, we don't pad the circuits (note that even in this case, the
/// default padding to the next power of 2 happens in [`PreprocessedCircuit::preprocess_circuit`]).
fn get_preprocessed_cairo_verifier(
    pcs_config: PcsConfig,
    target_padding: Option<ComponentSizes>,
) -> (PreprocessedCircuit, FinalizedContext<NoValue>) {
    let const_config = privacy_cairo_verifier_config(pcs_config.fri_config.log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    if let Some(target_padding) = target_padding {
        pad_to_targets(&mut novalue_context, &target_padding);
    }
    let preprocessed_cairo_verifier = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);
    (preprocessed_cairo_verifier, novalue_context)
}

/// Regression test on the padded sizes of the Cairo verifier and the multiverifier circuits.
///
/// Builds both circuits (unpadded) for `PCS_CONFIG` and prints their `compute_padded_sizes`
/// breakdowns side by side. For the multiverifier to be able to verify executions of the cairo
/// verifier and itself without changing, the sizes of the AIR components for both circuits must be
/// the same. This is achieved by padding each component to the next power of two of the max between
/// the component's size in the two AIRs (note that by doing this we may end up padding both
/// circuits). This test is needed to deduce the target padding whenever the `PCS_CONFIG` changes.
#[test]
fn test_regression_cairo_and_multiverifier_component_log_sizes() {
    let pcs_config = PCS_CONFIG;
    let (preprocessed_cairo_verifier, novalue_cairo_context) =
        get_preprocessed_cairo_verifier(pcs_config, None);

    let cairo_component_sizes = compute_padded_sizes(&novalue_cairo_context);
    let expected_sizes = ComponentSizes {
        eq: 1 << 17,
        qm31_ops: 1 << 21,
        m31_to_u32: 1 << 18,
        triple_xor: 1 << 17,
        blake_g_gate: 1 << 20,
    };
    assert_eq!(cairo_component_sizes, expected_sizes);

    let (_, novalue_multiverifier_context) =
        get_preprocessed_multiverifier_from_circuit(&preprocessed_cairo_verifier, pcs_config, None);
    let multiverifier_component_sizes = compute_padded_sizes(&novalue_multiverifier_context);
    let expected_sizes = ComponentSizes {
        eq: 1 << 15,
        qm31_ops: 1 << 20,
        m31_to_u32: 1 << 18,
        triple_xor: 1 << 17,
        blake_g_gate: 1 << 20,
    };
    assert_eq!(multiverifier_component_sizes, expected_sizes);
}

#[test]
fn test_padding_is_correct() {
    let pcs_config = PCS_CONFIG;
    let target_padding = TARGET_PADDING_SIZES;
    let (pp_cairo_circuit, _context) =
        get_preprocessed_cairo_verifier(pcs_config, Some(target_padding.clone()));
    let (pp_multiverifier_circuit, _context) = get_preprocessed_multiverifier_from_circuit(
        &pp_cairo_circuit,
        pcs_config,
        Some(target_padding),
    );
    assert_eq!(
        pp_multiverifier_circuit.preprocessed_trace.ids(),
        pp_cairo_circuit.preprocessed_trace.ids()
    );
    assert_eq!(pp_multiverifier_circuit.trace_log_size, pp_cairo_circuit.trace_log_size);
}

#[test]
fn test_regression_constants() {
    let pcs_config = PCS_CONFIG;
    let target_padding = TARGET_PADDING_SIZES;

    let (pp_cairo_circuit, _context) =
        get_preprocessed_cairo_verifier(pcs_config, Some(target_padding.clone()));
    let (pp_multiverifier, _context) = get_preprocessed_multiverifier_from_circuit(
        &pp_cairo_circuit,
        pcs_config,
        Some(target_padding),
    );
    let cairo_verifier_root: [u32; 8] = le_u32s_from_bytes(
        pp_cairo_circuit.preprocessed_root(pcs_config.fri_config.log_blowup_factor).0,
    );
    let multiverifier_root: [u32; 8] = le_u32s_from_bytes(
        pp_multiverifier.preprocessed_root(pcs_config.fri_config.log_blowup_factor).0,
    );
    assert_eq!(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT, cairo_verifier_root);
    assert_eq!(MULTIVERIFIER_PREPROCESSED_ROOT, multiverifier_root);
    assert_eq!(CIRCUIT_N_PREPROCESSED_COLUMNS, pp_multiverifier.preprocessed_trace.ids().len());
    assert_eq!(
        multiverifier_preprocessed_column_log_sizes(),
        pp_multiverifier.preprocessed_trace.log_sizes()
    );
}

/// Builds a `MultiverifierInput` that wraps a proof of the privacy cairo verifier (i.e. the circuit
/// with preprocessed root = `PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT`).
fn build_cairo_input(proof: &Proof<QM31>) -> MultiverifierInput<QM31> {
    MultiverifierInput {
        proof: proof.clone(),
        preprocessed_root: HashValue::<QM31>::from(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT),
        output_values: PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES,
    }
}

/// Builds and proves the Cairo verifier circuit on a privacy proof.
fn prove_privacy_with_recursion_and_prepare() -> (Proof<QM31>, CircuitPublicData) {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).expect("test_data/privacy/proof.bin must exist");
    let cairo_proof = binary_deserialize_from_file(&proof_file).expect("read cairo proof");

    let const_config = privacy_cairo_verifier_config(LOG_BLOWUP_FACTOR);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    pad_to_targets(&mut novalue_context, &TARGET_PADDING_SIZES);
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // QM31 context with values from the proof.
    let mut context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    pad_to_targets(&mut context, &TARGET_PADDING_SIZES);

    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        PCS_CONFIG,
    )
    .unwrap();

    prepare_circuit_proof_for_circuit_verifier(circuit_proof)
}

/// Regression test on the Cairo verifier proof.
///
/// Builds and proves the Cairo verifier circuit on the privacy proof (via
/// [`prove_privacy_with_recursion_and_prepare`]) and asserts its public outputs match
/// [`PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES`]. If `FIX_PROOF` env var is not set it deserializes the
/// proof stored at [`PRIVACY_CAIRO_VERIFIER_PROOF_PATH`] and asserts equality with the freshly
/// produced proof. When run with the `FIX_PROOF` env var set, it regenerates and overwrites the
/// proof.
#[test]
fn test_cairo_proof_regression() {
    let (proof, public_data) = prove_privacy_with_recursion_and_prepare();
    assert_eq!(public_data.output_values, PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES.map(QM31::pack_u32));
    if std::env::var("FIX_PROOF").is_err() {
        let proof_config = inner_verifier_proof_config();
        let bytes = std::fs::read(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
        let stored_proof =
            deserialize_proof_with_config(&mut bytes.as_slice(), &proof_config).unwrap();
        // Don't assert_eq to avoid printing large structs to stdout.
        if proof != stored_proof {
            panic!("The cairo verifier proof changed. Run again with `FIX_PROOF=1`.");
        };
    } else {
        let mut serialized = vec![];
        proof.serialize(&mut serialized);
        let mut file = File::create(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
        file.write_all(&serialized).unwrap();
    }
}

/// Builds the multiverifier over two Cairo verifier proofs.
///
/// Reads the stored Cairo verifier proof fixture (produced by [`test_cairo_proof_regression`]) and
/// builds a multiverifier circuit that verifies two copies of it.
///
/// When run with the `FIX_PROOF` env var set, it additionally preprocesses and proves the
/// multiverifier circuit, then serializes the resulting proof to
/// [`MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH`] as a fixture for downstream tests.
#[test]
fn test_prove_multiverifier_of_two_cairo_subcircuits() {
    let shared_config = SharedConfig {
        pcs_config: PCS_CONFIG,
        proof_config: inner_verifier_proof_config(),
        preprocessed_column_log_sizes: multiverifier_preprocessed_column_log_sizes(),
    };
    let bytes = std::fs::read(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
    let proof =
        deserialize_proof_with_config(&mut bytes.as_slice(), &shared_config.proof_config).unwrap();
    let mut multiverifier_context = build_multiverifier_circuit::<QM31>(
        build_cairo_input(&proof),
        build_cairo_input(&proof),
        &shared_config,
    );
    pad_to_targets(&mut multiverifier_context, &TARGET_PADDING_SIZES);
    multiverifier_context.validate_circuit();

    if std::env::var("FIX_PROOF").is_ok() {
        // Prove the multiverifier.
        let preprocessed_multiverifier =
            PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);
        let multi_circuit_proof = prove_circuit_assignment(
            multiverifier_context.values(),
            &preprocessed_multiverifier,
            &BaseColumnPool::<SimdBackend>::new(),
            PCS_CONFIG,
        )
        .unwrap();
        let (multi_proof, _multi_public_data) =
            prepare_circuit_proof_for_circuit_verifier(multi_circuit_proof);
        let mut serialized = vec![];
        multi_proof.serialize(&mut serialized);
        let mut file = File::create(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
        file.write_all(&serialized).unwrap();
    }
}

#[test]
fn test_verify_cairo_proof_and_multiverifier_proof() {
    // Build common config.
    let shared_config = SharedConfig {
        pcs_config: PCS_CONFIG,
        proof_config: inner_verifier_proof_config(),
        preprocessed_column_log_sizes: multiverifier_preprocessed_column_log_sizes(),
    };

    // Read the multiverifier proof of (the verification of) two cairo proofs.
    let bytes = std::fs::read(MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH).unwrap();
    let multiverifier_proof =
        deserialize_proof_with_config(&mut bytes.as_slice(), &shared_config.proof_config).unwrap();
    // Mirror the in-circuit preimage `build_multiverifier_circuit` hashes for each verified input:
    // the eight full 32-bit words of the circuit hash `blake2s(config_words || preprocessed_root)`,
    // followed by the eight raw output words.
    let cairo_input_preimage_words: Vec<u32> =
        leaf_circuit_hash(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT.into(), &shared_config)
            .into_iter()
            .chain(PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES)
            .collect();
    // The proven multiverifier verified two identical Cairo verifier inputs.
    let payload_words: Vec<u32> =
        chain!(&cairo_input_preimage_words, &cairo_input_preimage_words).copied().collect();
    // The circuit keeps the full unreduced eight-word digest as its output.
    let hash_of_payload: [u32; N_RESERVED] = native_blake_u32s(&payload_words);

    let multiverifier_of_two_cairo_input = MultiverifierInput {
        proof: multiverifier_proof,
        output_values: hash_of_payload,
        preprocessed_root: MULTIVERIFIER_PREPROCESSED_ROOT.into(),
    };

    // Read the cairo verifier proof.
    let bytes = std::fs::read(PRIVACY_CAIRO_VERIFIER_PROOF_PATH).unwrap();
    let proof =
        deserialize_proof_with_config(&mut bytes.as_slice(), &shared_config.proof_config).unwrap();

    // Build the multiverifier circuit that verifies a proof of itself and a proof of the cairo
    // verifier.
    let mut context = build_multiverifier_circuit::<QM31>(
        multiverifier_of_two_cairo_input,
        build_cairo_input(&proof),
        &shared_config,
    );
    pad_to_targets(&mut context, &TARGET_PADDING_SIZES);
    context.validate_circuit();

    // Check that the circuit hasn't changed. The `MULTIVERIFIER_PREPROCESSED_ROOT` is computed by
    // building the multiverifer on two cairo verifier proofs. The current test builds it on a
    // multiverifier proof and a cairo verifier proof.
    let preprocessed_multiverifier = PreprocessedCircuit::preprocess_circuit(&mut context);
    let preprocessed_root_multiverifier: [u32; 8] = le_u32s_from_bytes(
        preprocessed_multiverifier.preprocessed_root(PCS_CONFIG.fri_config.log_blowup_factor).0,
    );
    assert_eq!(preprocessed_root_multiverifier, MULTIVERIFIER_PREPROCESSED_ROOT);
}
