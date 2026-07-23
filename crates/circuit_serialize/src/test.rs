use circuits_stark_verifier::proof::{ProofConfig, ProofInfo};
use circuits_stark_verifier::proof_from_stark_proof::proof_from_stark_proof;
use circuits_stark_verifier_examples::simple_air::create_proof;
use circuits_stark_verifier_examples::simple_statement::{
    PREPROCESSED_COLUMN_LOG_SIZES, simple_statement_components,
};
use stwo::core::fields::qm31::QM31;

use crate::deserialize::deserialize_proof_with_config;
use crate::serialize::CircuitSerialize;

#[test]
fn test_serialize_deserialize() {
    let (_components, claim, pcs_config, proof, interaction_pow_nonce, channel_salt) =
        create_proof();

    let components = simple_statement_components::<QM31>();
    let config = ProofConfig::new(&components, PREPROCESSED_COLUMN_LOG_SIZES.len(), &pcs_config, 8);
    let proof = proof_from_stark_proof(&proof, &config, claim, interaction_pow_nonce, channel_salt);

    let mut serialized = Vec::new();
    proof.serialize(&mut serialized);
    assert_eq!(serialized.len(), ProofInfo::from_config(&config).total_bytes());
    let deserialized = deserialize_proof_with_config(&mut serialized.as_slice(), &config).unwrap();
    assert_eq!(proof, deserialized);
}
