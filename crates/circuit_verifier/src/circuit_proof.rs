use stwo::core::pcs::PcsConfig;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::merkle_hasher::MerkleHasherLifted;

use crate::circuit_claim::{CircuitClaim, CircuitInteractionClaim};

pub struct CircuitProof<H: MerkleHasherLifted> {
    pub pcs_config: PcsConfig,
    pub claim: CircuitClaim,
    pub interaction_pow_nonce: u64,
    pub interaction_claim: CircuitInteractionClaim,
    pub stark_proof: ExtendedStarkProof<H>,
    pub channel_salt: u32,
}
