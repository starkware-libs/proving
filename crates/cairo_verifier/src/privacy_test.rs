use std::fs::File;

use cairo_air::CairoProof;
use cairo_air::utils::binary_deserialize_from_file;
use circuit_common::N_RESERVED;
use circuit_common::finalize::{add_zk_blinding, pad_context};
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{
    BaseColumnPool, CircuitProof, SimdBackend, prepare_circuit_proof_for_circuit_verifier,
    prove_circuit_assignment,
};
use circuit_prover::test_utils::default_circuit_pcs_config;
use circuit_serialize::serialize::CircuitSerialize;
use circuit_verifier::statement::CircuitStatement;
use circuit_verifier::verify::{CircuitConfig, CircuitPublicData, verify_circuit};
use circuits::blake::HashValue;
use circuits::context::{Context, FinalizedContext};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use circuits_stark_verifier::proof::{ProofConfig, ProofInfo};
use circuits_stark_verifier::statement::Statement;
use itertools::Itertools;
use num_traits::Zero;
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;

use crate::privacy::{privacy_cairo_verifier_config, privacy_components};
use crate::utils::{get_proof_file_path, get_test_data_dir};
use crate::verify::{build_cairo_verifier_circuit, verify_cairo_with_component_set};

/// Verifies with a circuit a proof of execution of another circuit.
fn verify_circuit_proof(
    preprocessed_circuit: &PreprocessedCircuit,
    circuit_proof: CircuitProof<Blake2sMerkleHasher>,
    preprocessed_root: HashValue<QM31>,
) -> FinalizedContext<QM31> {
    let circuit_config = CircuitConfig {
        config: circuit_proof.pcs_config,
        n_outputs: preprocessed_circuit.n_outputs,
        preprocessed_column_log_sizes: preprocessed_circuit.preprocessed_trace.log_sizes(),
        preprocessed_root,
    };
    let (proof, public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);
    verify_circuit(circuit_config, proof, public_data).unwrap()
}

/// Compares the topology of two contexts.
/// Note that the values are not compared.
fn compare_contexts_topology<Value: IValue, OtherValue: IValue>(
    context_a: &FinalizedContext<Value>,
    context_b: &FinalizedContext<OtherValue>,
) {
    // TODO(Gali): Consider comparing unused and maybe unused vars.
    assert!(context_a.circuit() == context_b.circuit());
    assert!(context_a.stats() == context_b.stats());
    assert!(context_a.context.guessed_vars == context_b.context.guessed_vars);
    // TODO: constants is empty at this point?
    let constants_a = context_a.context.constants().iter().map(|(k, v)| (*k, v.idx)).collect_vec();
    let constants_b = context_b.context.constants().iter().map(|(k, v)| (*k, v.idx)).collect_vec();
    assert!(constants_a == constants_b);
}

#[test]
fn test_verify_privacy() {
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();

    // Verify the proof.
    let context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();

    // Build the verifier circuit via NoValue.
    let log_blowup_factor = 3;
    let const_config = privacy_cairo_verifier_config(log_blowup_factor);
    let novalue_context = build_cairo_verifier_circuit(&const_config);

    // Check that building the verifier circuit via NoValue produces the same topology.
    compare_contexts_topology(&context, &novalue_context);
}

#[test]
fn test_verify_privacy_with_recursion() {
    // The proof file was generated from the proving-utils repo using:
    // cargo run -r --bin stwo-run-and-prove -- \
    //     --prover_params_json ../stwo-circuits/test_data/privacy/prover_params.json \
    //     --program ../stwo-circuits/test_data/privacy/privacy_simple_bootloader_compiled.json \
    //     --program_input ../stwo-circuits/test_data/privacy/program_input.json \
    //     --proof_path ../stwo-circuits/test_data/privacy/proof.bin --proof-format extended-binary
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof: CairoProof<Blake2sMerkleHasher> =
        binary_deserialize_from_file(&proof_file).unwrap();

    // Build the preprocessed circuit from the NoValue topology (matching the real proving flow).
    let cairo_proof_log_blowup_factor = 3;
    let const_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);
    assert_eq!(
        const_config.preprocessed_root,
        cairo_proof.extended_stark_proof.proof.commitments.0[0].into(),
        "The preprocessed root in the config is not the same as the one in the proof"
    );
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    let mut context = verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    pad_context(&mut context);
    context.validate_circuit();
    let circuit_proof = prove_circuit_assignment(
        context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed.trace_log_size),
    )
    .unwrap();

    let preprocessed_root: HashValue<QM31> =
        circuit_proof.stark_proof.proof.commitments.0[0].into();

    // Serialize the recursion proof with the same raw `CircuitSerialize` format that
    // proving-utils' `privacy_recursive_prove` emits, before its version prefix and zstd
    // compression are applied. Assert it matches the committed fixture.
    let circuit_config = CircuitConfig {
        config: circuit_proof.pcs_config,
        n_outputs: preprocessed.n_outputs,
        preprocessed_column_log_sizes: preprocessed.preprocessed_trace.log_sizes(),
        preprocessed_root,
    };
    let (proof, public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);
    let mut proof_bytes = Vec::new();
    proof.serialize(&mut proof_bytes);

    // Regenerate with `FIX_PROOF=1 cargo test -p circuit-cairo-verifier
    // test_verify_privacy_with_recursion`.
    let fixture_path = get_test_data_dir().join("privacy").join("recursion_proof.bin");
    if std::env::var("FIX_PROOF").is_ok() {
        std::fs::write(&fixture_path, &proof_bytes).unwrap();
    }
    assert!(
        proof_bytes == std::fs::read(&fixture_path).unwrap(),
        "recursion proof fixture mismatch; regenerate with FIX_PROOF=1"
    );

    // Verify the recursion proof with the circuit verifier.
    verify_circuit(circuit_config, proof, public_data).unwrap();
}

#[test]
fn test_privacy_recursion_with_preprocessed_context() {
    // Build the verifier circuit via NoValue and preprocess it.
    let cairo_proof_log_blowup_factor = 3;
    let const_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);
    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    let preprocessed = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    // Verify the privacy proof to get a Context<QM31> with real values.
    let proof_path = get_proof_file_path("privacy");
    let proof_file = File::open(proof_path).unwrap();
    let cairo_proof = binary_deserialize_from_file(&proof_file).unwrap();
    let mut assignment_context =
        verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();

    // Prove via the assignment flow: finalize separately, then prove with pre-computed
    // preprocessed data.
    pad_context(&mut assignment_context);
    let assignment_proof = prove_circuit_assignment(
        assignment_context.values(),
        &preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed.trace_log_size),
    )
    .unwrap();

    // Prove via the full flow for comparison.
    let mut full_prove_context =
        verify_cairo_with_component_set(&cairo_proof, privacy_components()).unwrap();
    let full_preprocessed = PreprocessedCircuit::preprocess_circuit(&mut full_prove_context);
    let full_proof = prove_circuit_assignment(
        full_prove_context.values(),
        &full_preprocessed,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(full_preprocessed.trace_log_size),
    )
    .unwrap();

    // Verify both circuit proofs and compare the resulting verifier contexts.
    // TODO(Gali): Add verify fixed circuit
    let preprocessed_root: HashValue<QM31> =
        assignment_proof.stark_proof.proof.commitments.0[0].into();
    let assignment_verifier_context =
        verify_circuit_proof(&preprocessed, assignment_proof, preprocessed_root.clone());

    let full_verifier_context =
        verify_circuit_proof(&full_preprocessed, full_proof, preprocessed_root);

    // Compare the verifier contexts.
    compare_contexts_topology(&assignment_verifier_context, &full_verifier_context);
    assert!(assignment_verifier_context.values() == full_verifier_context.values());
}

#[test]
fn test_zk_padding() {
    // Build the verifier circuit via NoValue and preprocess it.
    for log_blowup_factor in 1..=3 {
        let const_config = privacy_cairo_verifier_config(log_blowup_factor);
        let mut context = build_cairo_verifier_circuit(&const_config);

        let eq_before = context.circuit().eq.len();
        let qm31_ops_before = context.circuit().n_qm31_ops_rows();

        add_zk_blinding(&mut context, [0; 32], const_config.proof_config.fri.n_queries);

        let eq_after = context.circuit().eq.len();
        let qm31_ops_after = context.circuit().n_qm31_ops_rows();

        assert_eq!(eq_after.next_power_of_two(), eq_before.next_power_of_two());
        assert_eq!(qm31_ops_after.next_power_of_two(), qm31_ops_before.next_power_of_two());
    }
}

#[test]
fn test_privacy_proof_info() {
    let cairo_proof_log_blowup_factor = 3;
    let const_config = privacy_cairo_verifier_config(cairo_proof_log_blowup_factor);

    let mut novalue_context = build_cairo_verifier_circuit(&const_config);
    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut novalue_context);

    let log_blowup_factor = 2;
    let lifting_log_size = preprocessed_circuit.trace_log_size + log_blowup_factor;
    let pcs_config = PcsConfig {
        pow_bits: 26,
        fri_config: FriConfig {
            log_blowup_factor,
            log_last_layer_degree_bound: 0,
            n_queries: 35,
            fold_step: 4,
        },
        min_lifting_log_size: lifting_log_size,
    };
    let preprocessed_root = HashValue::from([0u32; 8]);
    let circuit_config = CircuitConfig {
        config: pcs_config,
        n_outputs: preprocessed_circuit.n_outputs,
        preprocessed_column_log_sizes: preprocessed_circuit.preprocessed_trace.log_sizes(),
        preprocessed_root,
    };
    let public_data =
        CircuitPublicData { output_values: vec![QM31::zero(); preprocessed_circuit.n_outputs] };
    let mut context: Context<NoValue> = Context::new(N_RESERVED);
    let output_values = public_data
        .output_values
        .iter()
        .map(|value| NoValue::from_qm31(*value).guess(&mut context))
        .collect_vec();
    let statement = CircuitStatement::new(&mut context, &circuit_config, &output_values);

    let proof_config = ProofConfig::new(
        statement.get_components(),
        circuit_config.preprocessed_column_log_sizes.len(),
        &circuit_config.config,
        circuit_verifier::statement::INTERACTION_POW_BITS,
    );
    let proof_info = ProofInfo::from_config(&proof_config);
    println!("{proof_info}");
    // Assert the total size in bytes.
    assert_eq!(proof_info.total_bytes(), 263796);
}
