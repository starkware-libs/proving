//! Top-level Cairo verifier input.
//!
//! Mirrors the Cairo verifier's `main(proof: CircuitProof) -> VerificationOutput`
//! signature. Field order MUST match the Cairo `CircuitProof` struct in
//! `stwo-cairo/stwo_cairo_verifier/crates/circuit_air/src/lib.cairo`.
//!
//! The serializer is asymmetric on `queried_values`: the prover stores them in tree-major
//! order, the Cairo verifier reads them sorted by column size and transposed (see
//! `cairo_air::utils::sort_and_transpose_queried_values`). The conversion in
//! [`prepare_circuit_proof_for_cairo_verifier`] applies the sort before the derive emits
//! felts.

use cairo_air::utils::sort_and_transpose_queried_values;
use circuit_verifier::circuit_claim::column_log_sizes_per_tree;
use circuit_verifier::circuit_proof::CircuitProof;
use circuit_verifier::components::prelude::OrderedHashMap;
use starknet_ff::FieldElement;
use stwo::core::ColumnVec;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::fri::FriProof;
use stwo::core::pcs::PcsConfig;
use stwo::core::pcs::quotients::CommitmentSchemeProof;
use stwo::core::proof::StarkProof;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;
use stwo::core::vcs_lifted::verifier::MerkleDecommitmentLifted;
use stwo_cairo_serialize::{CairoDeserialize, CairoSerialize};

use crate::claim::{CairoCircuitClaim, CairoCircuitInteractionClaim};

/// Owned mirror of the Cairo `CircuitProof` struct, with `queried_values` already sorted
/// and transposed into the layout the Cairo verifier expects.
///
/// Symmetric `CairoSerialize`/`CairoDeserialize` derive — both directions read fields in
/// declaration order, so this round-trips cleanly.
// Note: cannot derive `PartialEq`/`Eq` because `FriProof`/`MerkleDecommitmentLifted` do
// not implement them. Roundtrip tests compare serialized felts instead.
#[derive(Clone, Debug, CairoSerialize, CairoDeserialize)]
pub struct CairoCircuitProof<H: MerkleHasherLifted<Hash = Blake2sHash>> {
    pub claim: CairoCircuitClaim,
    pub interaction_pow: u64,
    pub interaction_claim: CairoCircuitInteractionClaim,
    pub stark_proof: CairoStarkProof<H>,
    pub channel_salt: u32,
}

/// Owned counterpart of `CommitmentSchemeProof` with `queried_values` already in the
/// 2D sorted-and-transposed layout (one `Vec<BaseField>` per tree, concatenated across
/// queries) that the Cairo verifier deserializes.
#[derive(Clone, Debug, CairoSerialize, CairoDeserialize)]
pub struct CairoStarkProof<H: MerkleHasherLifted<Hash = Blake2sHash>> {
    pub config: PcsConfig,
    pub commitments: Vec<Blake2sHash>,
    pub sampled_values: Vec<ColumnVec<Vec<QM31>>>,
    pub decommitments: Vec<MerkleDecommitmentLifted<H>>,
    /// Sorted+transposed queried values (per tree).
    pub queried_values: Vec<Vec<M31>>,
    pub proof_of_work: u64,
    pub fri_proof: FriProof<H>,
}

/// Builds the felt252 input stream for the Cairo circuit verifier from a live circuit proof.
/// Only the proof is serialized — verifier-config constants (output addresses,
/// preprocessed root, `min_lifting_log_size`, preprocessed column log sizes)
/// are hardcoded inside the Cairo verifier binary for a specific circuit topology.
pub fn prepare_circuit_proof_for_cairo_verifier<H: MerkleHasherLifted<Hash = Blake2sHash>>(
    circuit_proof: CircuitProof<H>,
    component_log_sizes: &OrderedHashMap<&'static str, u32>,
) -> Vec<FieldElement> {
    let [trace_log_sizes, interaction_log_sizes] = column_log_sizes_per_tree(component_log_sizes);
    let stark_proof = CairoStarkProof::<H>::from_stark_proof(
        circuit_proof.stark_proof.proof,
        &[trace_log_sizes.as_slice(), interaction_log_sizes.as_slice()],
    );
    let circuit_proof_for_cairo = CairoCircuitProof::<H> {
        claim: CairoCircuitClaim::new(&circuit_proof.claim),
        interaction_pow: circuit_proof.interaction_pow_nonce,
        interaction_claim: CairoCircuitInteractionClaim::from(&circuit_proof.interaction_claim),
        stark_proof,
        channel_salt: circuit_proof.channel_salt,
    };
    let mut felts = Vec::new();
    CairoSerialize::serialize(&circuit_proof_for_cairo, &mut felts);
    felts
}

impl<H: MerkleHasherLifted<Hash = Blake2sHash>> CairoStarkProof<H> {
    /// Builds the Cairo-ready stark proof from a Rust `StarkProof` plus per-tree column
    /// log sizes for the [trace, interaction] trees (used to sort `queried_values`).
    pub fn from_stark_proof(
        proof: StarkProof<H>,
        trace_and_interaction_trace_log_sizes: &[&[u32]; 2],
    ) -> Self {
        let CommitmentSchemeProof {
            config,
            commitments,
            sampled_values,
            decommitments,
            queried_values,
            proof_of_work,
            fri_proof,
        } = proof.0;

        let sorted = sort_and_transpose_queried_values(
            &queried_values,
            trace_and_interaction_trace_log_sizes.to_vec(),
        );

        Self {
            config,
            commitments: commitments.0,
            sampled_values: sampled_values.0,
            decommitments: decommitments.0,
            queried_values: sorted.0,
            proof_of_work,
            fri_proof,
        }
    }
}
