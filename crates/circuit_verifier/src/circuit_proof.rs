use stwo::core::pcs::PcsConfig;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs::blake2_hash::Blake2sHash;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;

use crate::circuit_claim::{CircuitClaim, CircuitInteractionClaim};

/// Aggregates the prover's outputs: the STARK proof itself, and the values its consumers need
/// alongside it.
pub struct CircuitProof<H: MerkleHasherLifted> {
    pub pcs_config: PcsConfig,
    pub claim: CircuitClaim,
    pub interaction_pow_nonce: u64,
    pub interaction_claim: CircuitInteractionClaim,
    pub stark_proof: ExtendedStarkProof<H>,
    pub channel_salt: u32,
    /// `blake2s(log_blowup_factor || component_log_sizes || preprocessed_root)`: the identity of
    /// the proven circuit, as mixed into the channel by the prover.
    pub circuit_hash: Blake2sHash,
}
