//! End-to-end Cairo serde round-trip — analogous to
//! `circuit_serialize::test::test_serialize_deserialize`. Builds a real circuit prover
//! output, converts it into the Cairo verifier's input format via this crate's
//! `CairoSerialize` impls, deserializes the felt252 stream back via `CairoDeserialize`,
//! and asserts the result equals the original.

use circuit_common::preprocessed::PreprocessedCircuit;
use circuit_prover::prover::{BaseColumnPool, SimdBackend, prove_circuit_assignment};
use circuit_verifier::statement::{all_circuit_components, circuit_component_log_sizes};
use circuits::context::Context;
use circuits::ivalue::{NoValue, qm31_from_u32s};
use circuits::ops::guess;
use num_traits::{One, Zero};
use stwo::core::fields::qm31::QM31;
use stwo::core::pcs::PcsConfig;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

use crate::claim::{CairoCircuitClaim, CairoCircuitInteractionClaim};
use crate::proof::{CairoCircuitProof, prepare_circuit_proof_for_cairo_verifier};

fn qm31(a: u32, b: u32, c: u32, d: u32) -> QM31 {
    qm31_from_u32s(a, b, c, d)
}

/// Builds a tiny but real circuit assignment so we can run the prover and get a
/// well-formed `CircuitProof` for serde round-trip test. Same shape as the Fibonacci context used
/// in `circuit_prover::test`, just smaller.
fn build_minimal_context() -> Context<QM31> {
    const N: usize = 16;
    let mut ctx = Context::<QM31>::new(1);
    let (mut a, mut b) = (guess(&mut ctx, QM31::zero()), guess(&mut ctx, QM31::one()));
    for _ in 2..N {
        (a, b) = (b, circuits::eval!(&mut ctx, (a) + (b)));
    }
    ctx.set_outputs(&[b]);
    ctx
}

#[test]
fn test_serialize_deserialize_cairo_proof() {
    let mut ctx = build_minimal_context().finalize(false);
    ctx.validate_circuit();
    let preprocessed_circuit = PreprocessedCircuit::preprocess_circuit(&mut ctx);
    let circuit_proof = prove_circuit_assignment(
        ctx.values(),
        &preprocessed_circuit,
        &BaseColumnPool::<SimdBackend>::new(),
        PcsConfig::default(),
    )
    .unwrap();

    // Round-trip via the same CairoSerialize / CairoDeserialize implementations the
    // Cairo verifier consumes. We can't compare `CairoCircuitProof` values directly
    // (FriProof / MerkleDecommitmentLifted don't implement `PartialEq`), so we compare
    // re-serialized felts — if the deserializer is the inverse of the serializer, the
    // two byte streams must match.
    let preprocessed_column_log_sizes = preprocessed_circuit.preprocessed_trace.log_sizes();
    let component_log_sizes = circuit_component_log_sizes(
        &all_circuit_components::<NoValue>(),
        &preprocessed_column_log_sizes,
    );
    let felts = prepare_circuit_proof_for_cairo_verifier(circuit_proof, &component_log_sizes);
    let mut iter = felts.iter();
    let deserialized: CairoCircuitProof<Blake2sMerkleHasher> =
        CairoCircuitProof::deserialize(&mut iter);
    assert!(iter.next().is_none(), "trailing data after proof");
    let mut felts_after = Vec::new();
    CairoSerialize::serialize(&deserialized, &mut felts_after);
    assert_eq!(felts, felts_after);
}

#[test]
fn test_serialize_deserialize_claim_and_interaction_claim() {
    // Distinct values in `ComponentList` order so any ordering bug shows up.
    let claim = CairoCircuitClaim { output_values: vec![qm31(1, 2, 3, 4), qm31(5, 6, 7, 8)] };
    let interaction = CairoCircuitInteractionClaim {
        claimed_sums: [
            qm31(1, 0, 0, 0),
            qm31(2, 0, 0, 0),
            qm31(3, 0, 0, 0),
            qm31(4, 0, 0, 0),
            qm31(5, 0, 0, 0),
            qm31(6, 0, 0, 0),
            qm31(7, 0, 0, 0),
            qm31(8, 0, 0, 0),
            qm31(9, 0, 0, 0),
            qm31(10, 0, 0, 0),
            qm31(11, 0, 0, 0),
        ],
    };

    // Roundtrip claim
    let mut felts = Vec::new();
    CairoSerialize::serialize(&claim, &mut felts);
    let mut iter = felts.iter();
    let claim_back = CairoCircuitClaim::deserialize(&mut iter);
    assert!(iter.next().is_none());
    assert_eq!(claim, claim_back);

    // Roundtrip interaction_claim
    let mut felts = Vec::new();
    CairoSerialize::serialize(&interaction, &mut felts);
    let mut iter = felts.iter();
    let interaction_back = CairoCircuitInteractionClaim::deserialize(&mut iter);
    assert!(iter.next().is_none());
    assert_eq!(interaction, interaction_back);
}
