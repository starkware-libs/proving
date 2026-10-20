use stwo::core::air::Component;
use stwo::core::channel::{Blake2sChannel, Channel};
use stwo::core::pcs::{CommitmentSchemeVerifier, TreeVec};
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleChannel;
use stwo::core::verifier::verify;

use crate::simple_air::{INTERACTION_POW_BITS, LOG_SIZE_LONG, LOG_SIZE_SHORT, create_proof};

#[test]
fn verify_simple_proof() {
    let (components, claimed_sums, config, proof, interaction_pow_nonce, channel_salt) =
        create_proof();

    // Verify.
    let verifier_channel = &mut Blake2sChannel::default();
    verifier_channel.mix_felts(&[channel_salt.into()]);
    config.fri_params.mix_into(verifier_channel);
    let commitment_scheme = &mut CommitmentSchemeVerifier::<Blake2sMerkleChannel>::new(config);

    // Retrieve the expected column sizes in each commitment interaction, from the AIR.
    let sizes = TreeVec::concat_cols(components.iter().map(|c| c.trace_log_degree_bounds()));
    let preprocessed_column_sizes = vec![LOG_SIZE_SHORT, LOG_SIZE_LONG];

    commitment_scheme.commit(
        proof.proof.commitments[0],
        &preprocessed_column_sizes,
        verifier_channel,
    );

    // Mix the (empty) public claim into the channel.
    verifier_channel.mix_felts(&[]);
    commitment_scheme.commit(proof.proof.commitments[1], &sizes[1], verifier_channel);
    verifier_channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce);
    verifier_channel.mix_u64(interaction_pow_nonce);
    verifier_channel.mix_felts(&claimed_sums);
    commitment_scheme.commit(proof.proof.commitments[2], &sizes[2], verifier_channel);
    verify(
        &components.iter().map(|c| c.as_ref()).collect::<Vec<&dyn Component>>(),
        verifier_channel,
        commitment_scheme,
        proof.proof,
    )
    .unwrap();
}
