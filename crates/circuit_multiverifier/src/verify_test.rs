use std::fs::File;
use std::io::Write;

use cairo_air::utils::binary_deserialize_from_file;
use circuit_cairo_serialize::prepare_circuit_proof_for_cairo_verifier;
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
    prove_circuit_assignment_with_channel,
};
use circuit_serialize::deserialize::deserialize_proof_with_config;
use circuit_serialize::serialize::CircuitSerialize;
use circuit_verifier::statement::{
    all_circuit_components, circuit_component_log_sizes, circuit_verifier_proof_config,
};
use circuit_verifier::verify::CircuitPublicData;
use circuits::blake::HashValue;
use circuits::context::FinalizedContext;
use circuits::ivalue::{IValue, NoValue};
use circuits_stark_verifier::proof::{Proof, ProofConfig};
use itertools::chain;
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;

use crate::test_utils::{
    CIRCUIT_N_PREPROCESSED_COLUMNS, LOG_BLOWUP_FACTOR, MULTIVERIFIER_OF_TWO_CAIRO_PROOFS_PATH,
    MULTIVERIFIER_PREPROCESSED_ROOT, PCS_CONFIG, PRIVACY_CAIRO_VERIFIER_OUTPUT_VALUES,
    PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT, TARGET_PADDING_SIZES,
    get_preprocessed_multiverifier_from_circuit, get_preprocessed_root, leaf_circuit_hash,
    multiverifier_preprocessed_column_log_sizes, native_blake_u32s,
};
use crate::verify::{MultiverifierInput, SharedConfig, build_multiverifier_circuit};

/// The Cairo verifier proof fixture (produced by [`test_cairo_proof_regression`]).
const PRIVACY_CAIRO_VERIFIER_PROOF_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/circuit_multiverifier/proof_cairo.bin");

/// The same multiverifier proof, serialized as the felt252 `scarb execute --arguments-file` input
/// consumed by the Cairo1 verifier program: a JSON array of hex-string felts produced via
/// [`circuit_cairo_serialize`]. Regenerate with `FIX_PROOF=1`.
const CAIRO1_PROOF_PATH: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../../test_data/circuit_multiverifier/cairo1_proof.json");

/// Extracts the eight raw 32-bit words from a `HashValue<QM31>` (each word held as
/// `(low_u16, high_u16, 0, 0)`).
fn hash_value_to_u32s(hash: &HashValue<QM31>) -> [u32; 8] {
    std::array::from_fn(|i| hash[i].get().unpack_u32())
}

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
        pad_to_targets(&mut novalue_context, target_padding);
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
        eq: 1 << 13,
        qm31_ops: 1 << 19,
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
    let cairo_verifier_root =
        get_preprocessed_root(&pp_cairo_circuit, pcs_config.fri_config.log_blowup_factor);
    let multiverifier_root =
        get_preprocessed_root(&pp_multiverifier, pcs_config.fri_config.log_blowup_factor);
    assert_eq!(PRIVACY_CAIRO_VERIFIER_PREPROCESSED_ROOT, hash_value_to_u32s(&cairo_verifier_root));
    assert_eq!(MULTIVERIFIER_PREPROCESSED_ROOT, hash_value_to_u32s(&multiverifier_root));
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
    pad_to_targets(&mut novalue_context, TARGET_PADDING_SIZES.clone());
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // QM31 context with values from the proof.
    let mut context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    pad_to_targets(&mut context, TARGET_PADDING_SIZES.clone());

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
    pad_to_targets(&mut multiverifier_context, TARGET_PADDING_SIZES.clone());
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

/// Regression test producing a proof for the Cairo1 verifier program.
///
/// Proves the multiverifier circuit over two Cairo verifier proofs (as in
/// [`test_prove_multiverifier_of_two_cairo_subcircuits`]) and serializes the resulting proof into
/// the felt252 stream consumed by the Cairo1 verifier program
/// (`main(proof: CircuitProof) -> VerificationOutput`) via
/// [`prepare_circuit_proof_for_cairo_verifier`]. This is a different channel and serialization than
/// the in-repo (u32-stream) circuit verifier: the felts are sorted/transposed and the
/// verifier-config constants are hardcoded in the Cairo binary, so only the proof is serialized.
///
/// The felts are stored as a JSON array of hex strings (the `scarb execute --arguments-file`
/// format). If `FIX_PROOF` is not set, the freshly produced stream is compared against
/// [`CAIRO1_PROOF_PATH`]; when run with `FIX_PROOF` set, it regenerates and overwrites the fixture.
#[test]
fn test_serialize_multiverifier_proof_for_cairo1_verifier() {
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
    pad_to_targets(&mut multiverifier_context, TARGET_PADDING_SIZES.clone());
    multiverifier_context.validate_circuit();

    // Prove the multiverifier and serialize the proof for the Cairo1 verifier program. The Cairo1
    // verifier uses the standard (lossless) `Blake2sMerkleChannel`, not the
    // `Blake2sM31MerkleChannel` used by the in-repo circuit verifier, so we prove with that
    // channel explicitly.
    let preprocessed_multiverifier =
        PreprocessedCircuit::preprocess_circuit(&mut multiverifier_context);

    let cairo1_verifier_pcs_config = PcsConfig { min_lifting_log_size: 0, ..PCS_CONFIG };
    let multi_circuit_proof = prove_circuit_assignment_with_channel::<Blake2sMerkleChannel>(
        multiverifier_context.values(),
        &preprocessed_multiverifier,
        &BaseColumnPool::<SimdBackend>::new(),
        cairo1_verifier_pcs_config,
    )
    .unwrap();
    let component_log_sizes = circuit_component_log_sizes(
        &all_circuit_components::<NoValue>(),
        &preprocessed_multiverifier.preprocessed_trace.log_sizes(),
    );
    let felts = prepare_circuit_proof_for_cairo_verifier(multi_circuit_proof, &component_log_sizes);
    let proof_hex: Vec<String> = felts.iter().map(|felt| format!("0x{felt:x}")).collect();
    let proof_json = serde_json::to_string_pretty(&proof_hex).unwrap();

    if std::env::var("FIX_PROOF").is_err() {
        let stored = std::fs::read_to_string(CAIRO1_PROOF_PATH).unwrap();
        // Don't assert_eq to avoid printing the full felt stream to stdout.
        if proof_json != stored {
            panic!(
                "The multiverifier proof serialized for the Cairo1 verifier changed. Run again \
                 with `FIX_PROOF=1`."
            );
        }
    } else {
        let mut file = File::create(CAIRO1_PROOF_PATH).unwrap();
        file.write_all(proof_json.as_bytes()).unwrap();
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
    pad_to_targets(&mut context, TARGET_PADDING_SIZES);
    context.validate_circuit();

    // Check that the circuit hasn't changed. The `MULTIVERIFIER_PREPROCESSED_ROOT` is computed by
    // building the multiverifer on two cairo verifier proofs. The current test builds it on a
    // multiverifier proof and a cairo verifier proof.
    let preprocessed_multiverifier = PreprocessedCircuit::preprocess_circuit(&mut context);
    let preprocessed_root_multiverifier =
        get_preprocessed_root(&preprocessed_multiverifier, PCS_CONFIG.fri_config.log_blowup_factor);
    assert_eq!(
        hash_value_to_u32s(&preprocessed_root_multiverifier),
        MULTIVERIFIER_PREPROCESSED_ROOT
    );
}
