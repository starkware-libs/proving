use circuit_common::finalize::pad_context;
use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_verifier::circuit_claim::{
    CircuitInteractionElements, column_log_sizes_per_tree, lookup_sum, mix_circuit_hash,
};
use circuit_verifier::statement::{
    INTERACTION_POW_BITS, all_circuit_components, circuit_component_log_sizes,
};
use circuit_verifier::verify::{CircuitConfig, verify_circuit};
use circuits::blake::{blake_g_gate, blake2s_m31, m31_to_u32, triple_xor};
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::{IValue, NoValue, qm31_from_u32s};
use circuits::ops::{Guess, guess, permute};
use circuits::utils::le_u32s_from_bytes;
use circuits::wrappers::U32Wrapper;
use expect_test::expect;
use num_traits::{One, Zero};
use stwo::core::channel::{Blake2sM31Channel, Channel};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::CommitmentSchemeVerifier;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sMerkleHasher};

use crate::circuit_air::circuit_components::CircuitComponents;
use crate::circuit_hash::compute_circuit_hash;
use crate::prover::{
    BaseColumnPool, CircuitProof, SimdBackend, prepare_circuit_proof_for_circuit_verifier,
    prove_circuit_assignment,
};
use crate::test_utils::default_circuit_pcs_config;
// Not a power of 2 so that we can test component padding.
const N: usize = 1030;

pub fn build_fibonacci_context() -> Context<QM31> {
    let mut context = Context::<QM31>::new(1);

    let (mut a, mut b) = (guess(&mut context, QM31::zero()), guess(&mut context, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, eval!(&mut context, (a) + (b)));
    }

    expect![["
        (809871181 + 0i) + (0 + 0i)u
    "]]
    .assert_debug_eq(&context.get(b));
    context.set_outputs(&[b]);

    context
}

pub fn build_permutation_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let a = guess(&mut context, qm31_from_u32s(0, 2, 0, 2));
    let b = guess(&mut context, qm31_from_u32s(1, 1, 1, 1));

    let outputs = permute(&mut context, &[a, b], IValue::sort_by_u_coordinate);
    let _outputs = permute(&mut context, &outputs, IValue::sort_by_u_coordinate);

    context
}

pub fn build_blake_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();
    context.enable_assert_eq_on_eval();

    let mut inputs: Vec<Var> = vec![];
    let n_inputs = 9;
    let n_bytes = n_inputs * 16;
    let n_blakes = 15;
    for i in 0..n_inputs {
        inputs.push(guess(
            &mut context,
            qm31_from_u32s(4 * i + 82, 4 * i + 83, 4 * i + 84, 4 * i + 85),
        ));
    }
    for _ in 0..n_blakes {
        let output = blake2s_m31(&mut context, &inputs, n_bytes as usize);
        eval!(&mut context, (output.0) + (output.1));
    }

    context
}

pub fn build_triple_xor_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    // Inputs are u32 values packed as (low_16, high_16, 0, 0).
    // 42 ^ 17 ^ 55 = 12
    let a = U32Wrapper::from(42).guess(&mut context);
    let b = U32Wrapper::from(17).guess(&mut context);
    let c = U32Wrapper::from(55).guess(&mut context);
    let out = triple_xor(&mut context, a, b, c);
    expect![["
        U32((12 + 0i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out.get_value(&context));

    // 0x10000 ^ 0x20000 ^ 0x30001 = 1
    let a = U32Wrapper::from(0x10000).guess(&mut context);
    let b = U32Wrapper::from(0x20000).guess(&mut context);
    let c = U32Wrapper::from(0x30001).guess(&mut context);
    let out = triple_xor(&mut context, a, b, c);
    expect![["
        U32((1 + 0i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out.get_value(&context));

    // 0x30005 ^ 0x10007 ^ 0x4000b = 0x60009
    let a = U32Wrapper::from(0x30005).guess(&mut context);
    let b = U32Wrapper::from(0x10007).guess(&mut context);
    let c = U32Wrapper::from(0x4000b).guess(&mut context);
    let out = triple_xor(&mut context, a, b, c);
    expect![["
        U32((9 + 6i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out.get_value(&context));

    context
}

pub fn build_m31_to_u32_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    let a = guess(&mut context, QM31::from(42));
    let out_a = m31_to_u32(&mut context, a);
    expect![["
        U32((42 + 0i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_a.get_value(&context));

    let b = guess(&mut context, QM31::from(100_000));
    let out_b = m31_to_u32(&mut context, b);
    expect![["
        U32((34464 + 1i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_b.get_value(&context));

    let c = guess(&mut context, QM31::from(2_000_042));
    let out_c = m31_to_u32(&mut context, c);
    expect![["
        U32((33962 + 30i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_c.get_value(&context));

    context
}

pub fn build_blake_g_gate_context() -> Context<QM31> {
    let mut context = Context::<QM31>::default();

    // Inputs are u32 values packed as (low_16, high_16, 0, 0).
    // G(305419896, 4294967295, 2147483647, 123456789, 987654321, 468798)
    //   => (2827666065, 4146123195, 3407348176, 3638212488)
    let a = U32Wrapper::from(305419896).guess(&mut context);
    let b = U32Wrapper::from(4294967295).guess(&mut context);
    let c = U32Wrapper::from(2147483647).guess(&mut context);
    let d = U32Wrapper::from(123456789).guess(&mut context);
    let f0 = U32Wrapper::from(987654321).guess(&mut context);
    let f1 = U32Wrapper::from(468798).guess(&mut context);

    let [out_a, out_b, out_c, out_d] = blake_g_gate(&mut context, a, b, c, d, f0, f1);
    expect![["
        U32((49809 + 43146i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_a.get_value(&context));
    expect![["
        U32((53691 + 63264i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_b.get_value(&context));
    expect![["
        U32((464 + 51992i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_c.get_value(&context));
    expect![["
        U32((46984 + 55514i) + (0 + 0i)u)
    "]]
    .assert_debug_eq(&out_d.get_value(&context));

    context
}

/// Verifies a [`CircuitProof`] using the stwo verifier. Asserts that the proof is valid
/// and that the logup sum is zero.
fn stwo_verify(
    circuit_proof: CircuitProof<Blake2sMerkleHasher>,
    preprocessed_circuit: &PreprocessedCircuit,
) {
    let CircuitProof {
        claim,
        interaction_claim,
        pcs_config,
        stark_proof: proof,
        interaction_pow_nonce,
        channel_salt,
        circuit_hash: _,
    } = circuit_proof;

    let preprocessed_column_log_sizes = preprocessed_circuit.preprocessed_trace.log_sizes();
    let log_sizes = circuit_component_log_sizes(
        &all_circuit_components::<NoValue>(),
        &preprocessed_column_log_sizes,
    );

    let log_blowup_factor = pcs_config.fri_config.log_blowup_factor;
    let verifier_channel = &mut Blake2sM31Channel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    pcs_config.mix_into(verifier_channel);
    let commitment_scheme =
        &mut CommitmentSchemeVerifier::<Blake2sM31MerkleChannel>::new(pcs_config);

    let [trace_log_sizes, interaction_log_sizes] = column_log_sizes_per_tree(&log_sizes);

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_circuit.preprocessed_trace.log_sizes().values().copied().collect::<Vec<_>>(),
        verifier_channel,
    );
    let preprocessed_root = proof.proof.commitments[0];
    let circuit_hash = compute_circuit_hash(&log_sizes, log_blowup_factor, preprocessed_root);
    mix_circuit_hash(verifier_channel, &circuit_hash);
    claim.mix_into(verifier_channel);
    commitment_scheme.commit(proof.proof.commitments[1], &trace_log_sizes, verifier_channel);

    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);

    verifier_channel.mix_u64(interaction_pow_nonce);
    let interaction_elements = CircuitInteractionElements::draw(verifier_channel);

    interaction_claim.mix_into(verifier_channel);

    commitment_scheme.commit(proof.proof.commitments[2], &interaction_log_sizes, verifier_channel);

    // Build components for constraint verification.
    let components = CircuitComponents::new(
        &interaction_elements,
        &interaction_claim,
        &log_sizes,
        &preprocessed_circuit.preprocessed_trace.ids(),
    );
    stwo::core::verifier::verify_ex(
        &components.components(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
        true,
    )
    .unwrap();

    assert_eq!(lookup_sum(&claim, &interaction_claim, &interaction_elements,), QM31::zero());
}

#[test]
fn test_prove_and_stark_verify_blake_gate_context() {
    let mut blake_context = build_blake_context().finalize(false);
    blake_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut blake_context);
    let circuit_proof = prove_circuit_assignment(
        blake_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    stwo_verify(circuit_proof, &preprocessed_circuit);
}

#[test]
fn test_prove_and_stark_verify_permutation_context() {
    let mut permutation_context = build_permutation_context().finalize(false);
    permutation_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut permutation_context);
    let circuit_proof = prove_circuit_assignment(
        permutation_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    stwo_verify(circuit_proof, &preprocessed_circuit);
}

#[test]
fn test_prove_and_stark_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context().finalize(false);
    fibonacci_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut fibonacci_context);
    let circuit_proof = prove_circuit_assignment(
        fibonacci_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    stwo_verify(circuit_proof, &preprocessed_circuit);
}

#[test]
fn test_prove_and_stark_verify_triple_xor_context() {
    let mut triple_xor_context = build_triple_xor_context().finalize(false);
    triple_xor_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut triple_xor_context);
    let circuit_proof = prove_circuit_assignment(
        triple_xor_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    stwo_verify(circuit_proof, &preprocessed_circuit);
}

#[test]
fn test_prove_and_stark_verify_m31_to_u32_context() {
    let mut m31_to_u32_context = build_m31_to_u32_context().finalize(false);
    m31_to_u32_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut m31_to_u32_context);
    let circuit_proof = prove_circuit_assignment(
        m31_to_u32_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    stwo_verify(circuit_proof, &preprocessed_circuit);
}

#[test]
fn test_prove_and_stark_verify_blake_g_gate_context() {
    let mut blake_g_gate_context = build_blake_g_gate_context().finalize(false);
    blake_g_gate_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut blake_g_gate_context);
    let circuit_proof = prove_circuit_assignment(
        blake_g_gate_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    stwo_verify(circuit_proof, &preprocessed_circuit);
}

/// Verifies a [`CircuitProof`] using the circuit verifier. Requires the expected
/// `preprocessed_root` of the preprocessed trace.
fn circuit_verify(
    circuit_proof: CircuitProof<Blake2sMerkleHasher>,
    preprocessed_circuit: &PreprocessedCircuit,
    preprocessed_root: [u32; 8],
) {
    let circuit_config = CircuitConfig {
        config: circuit_proof.pcs_config,
        n_outputs: preprocessed_circuit.n_outputs,
        preprocessed_column_log_sizes: preprocessed_circuit.preprocessed_trace.log_sizes(),
    };
    let (proof, public_data) = prepare_circuit_proof_for_circuit_verifier(circuit_proof);
    verify_circuit(circuit_config, preprocessed_root.into(), proof, public_data).unwrap();
}

#[test]
fn test_prove_and_circuit_verify_triple_xor_context() {
    let mut triple_xor_context = build_triple_xor_context().finalize(false);
    triple_xor_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut triple_xor_context);
    let circuit_proof = prove_circuit_assignment(
        triple_xor_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    let preprocessed_root = preprocessed_root_from_proof(&circuit_proof);
    expect![
        "[975308199, 1477227831, 656207450, 1107639409, 4290412986, 3677648178, 516458056, \
         1265865711]"
    ]
    .assert_eq(&format!("{preprocessed_root:?}"));
    circuit_verify(circuit_proof, &preprocessed_circuit, preprocessed_root);
}

/// Extract the preprocessed-trace Merkle root (`commitments[0]`) from a `CircuitProof` as
/// `[u32; 8]`, matching the layout `ReducedHashValue<QM31>` consumes via `From<[u32; 8]>`.
fn preprocessed_root_from_proof(circuit_proof: &CircuitProof<Blake2sMerkleHasher>) -> [u32; 8] {
    let hash: Blake2sHash = circuit_proof.stark_proof.proof.commitments[0];
    le_u32s_from_bytes(hash.0)
}

#[test]
fn test_prove_and_circuit_verify_fibonacci_context() {
    let mut fibonacci_context = build_fibonacci_context().finalize(false);
    fibonacci_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut fibonacci_context);
    let circuit_proof = prove_circuit_assignment(
        fibonacci_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    let preprocessed_root = preprocessed_root_from_proof(&circuit_proof);
    expect![
        "[834735002, 594172773, 1583316646, 3249196940, 741016670, 2295728685, 4109491583, \
         2430221502]"
    ]
    .assert_eq(&format!("{preprocessed_root:?}"));
    circuit_verify(circuit_proof, &preprocessed_circuit, preprocessed_root);
}

#[test]
fn test_prove_and_circuit_verify_m31_to_u32_context() {
    let mut m31_to_u32_context = build_m31_to_u32_context().finalize(false);
    m31_to_u32_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut m31_to_u32_context);
    let circuit_proof = prove_circuit_assignment(
        m31_to_u32_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    let preprocessed_root = preprocessed_root_from_proof(&circuit_proof);
    expect![
        "[3771636404, 3055692813, 1894577333, 698197554, 2504506842, 900992605, 91068715, \
         318976758]"
    ]
    .assert_eq(&format!("{preprocessed_root:?}"));
    circuit_verify(circuit_proof, &preprocessed_circuit, preprocessed_root);
}

#[test]
fn test_prove_and_circuit_verify_blake_g_gate_context() {
    let mut blake_g_gate_context = build_blake_g_gate_context().finalize(false);
    blake_g_gate_context.validate_circuit();

    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut blake_g_gate_context);
    let circuit_proof = prove_circuit_assignment(
        blake_g_gate_context.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        default_circuit_pcs_config(preprocessed_circuit.trace_log_size),
    )
    .unwrap();
    let preprocessed_root = preprocessed_root_from_proof(&circuit_proof);
    expect![
        "[3530521409, 3130464395, 2146320570, 2669261259, 2154330194, 637627870, 3981384222, \
         4078960982]"
    ]
    .assert_eq(&format!("{preprocessed_root:?}"));
    circuit_verify(circuit_proof, &preprocessed_circuit, preprocessed_root);
}

#[test]
fn test_pad_context() {
    let mut context = build_fibonacci_context().finalize(false);
    pad_context(&mut context);

    let circuit = context.circuit();
    assert!(circuit.permutation.is_empty());
    let qm31_ops = circuit.n_qm31_ops_rows();
    assert!(qm31_ops.is_power_of_two());
    context.validate_circuit();
}
