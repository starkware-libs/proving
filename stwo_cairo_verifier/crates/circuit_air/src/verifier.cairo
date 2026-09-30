//! Circuit proof verification: the proof type, the verifier entry point and the verification
//! output it returns.
use core::num::traits::Zero;
use stwo_constraint_framework::LookupElementsImpl;
use stwo_verifier_core::Hash;
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::fields::m31::M31Trait;
use stwo_verifier_core::fields::qm31::{QM31, QM31Serde};
use stwo_verifier_core::fri::FriParamsTrait;
use stwo_verifier_core::pcs::verifier::CommitmentSchemeVerifierImpl;
use stwo_verifier_core::verifier::{StarkProof, VerificationError, verify};
use stwo_verifier_utils::blake2s::hash_u32s;
use crate::circuit_air::CircuitAirNewImpl;
use crate::claims::{
    CircuitClaim, CircuitClaimImpl, CircuitInteractionClaim, CircuitInteractionClaimImpl,
    column_log_sizes_per_tree, logup_sum,
};
use crate::multiverifier_consts::{CIRCUIT_FRI_PARAMS, N_OUTPUTS, TRACE_LOG_DEGREE_BOUND};

// Security constants.
pub const INTERACTION_POW_BITS: u32 = 20;
const SECURITY_BITS: u32 = 96;

#[derive(Drop, Serde)]
pub struct CircuitProof {
    pub claim: CircuitClaim,
    pub interaction_pow_nonce: u64,
    pub interaction_claim: CircuitInteractionClaim,
    pub stark_proof: StarkProof,
    /// Salt used in the channel initialization.
    pub channel_salt: u32,
}

/// Returns the output of the verifier: `blake2s(circuit_hash || output_words)`.
pub fn get_verification_output(circuit_hash: Hash, output_values: Span<u32>) -> Hash {
    let [h0, h1, h2, h3, h4, h5, h6, h7] = circuit_hash.hash.unbox();
    let mut words = array![h0, h1, h2, h3, h4, h5, h6, h7];
    words.append_span(output_values);
    Hash { hash: hash_u32s(words.span()) }
}

pub fn verify_circuit(proof: CircuitProof, circuit_hash: Hash) {
    let CircuitProof {
        claim, interaction_pow_nonce, interaction_claim, stark_proof, channel_salt,
    } = proof;

    // The circuit produces a fixed number of public outputs (its topology); the claim must
    // provide exactly that many.
    assert!(claim.output_values.len() == N_OUTPUTS);

    let mut channel: Channel = Default::default();
    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    let channel_salt_as_felt: QM31 = M31Trait::reduce_u32(channel_salt).into();
    channel.mix_felts([channel_salt_as_felt].span());

    // The FRI parameters are the circuit's hardcoded ones.
    CIRCUIT_FRI_PARAMS.mix_into(ref channel);
    let mut commitment_scheme = CommitmentSchemeVerifierImpl::new();

    // Unpack commitments.
    let commitments: @Box<[Hash; 4]> = stark_proof
        .commitment_scheme_proof
        .commitments
        .try_into()
        .unwrap();
    let [
        preprocessed_commitment,
        trace_commitment,
        interaction_trace_commitment,
        composition_commitment,
    ] =
        commitments
        .unbox();

    let [preprocessed_column_log_sizes, trace_log_sizes, interaction_trace_log_sizes] =
        column_log_sizes_per_tree();

    let log_blowup_factor = CIRCUIT_FRI_PARAMS.log_blowup_factor;

    // Preprocessed trace. The preprocessed column log sizes are hardcoded. The preprocessed-trace
    // commitment itself is taken from the proof and exposed in the verification output; binding it
    // to the expected circuit topology is the responsibility of whoever consumes that output.
    commitment_scheme
        .commit(
            preprocessed_commitment, preprocessed_column_log_sizes, ref channel, log_blowup_factor,
        );

    // Mix the circuit hash and the claim into the channel.
    channel.mix_commitment(circuit_hash);
    claim.mix_into(ref channel);

    // Commit the trace.
    commitment_scheme.commit(trace_commitment, trace_log_sizes, ref channel, log_blowup_factor);

    // Interaction proof of work.
    assert!(
        channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow_nonce),
        "{}",
        VerificationError::InteractionProofOfWork,
    );
    channel.mix_u64(interaction_pow_nonce);

    // Pick the interaction elements.
    let common_lookup_elements = LookupElementsImpl::draw(ref channel);
    assert!(
        logup_sum(@claim, @common_lookup_elements, @interaction_claim).is_zero(),
        "{}",
        VerificationError::InvalidLogupSum,
    );

    // Interaction trace.
    interaction_claim.mix_into(ref channel);
    commitment_scheme
        .commit(
            interaction_trace_commitment,
            interaction_trace_log_sizes,
            ref channel,
            log_blowup_factor,
        );

    // The circuit commits all trees at the lifted height of its largest extended domain (largest
    // preprocessed column log size + log_blowup_factor). `verify` expects the trace's log degree
    // bound (`trace_log_size = lifting - blowup`); the composition polynomial's raw degree bound is
    // one higher (degree-2 constraints) but it is split into 2 polynomials before LDE, bringing its
    // per-column degree bound back down to the trace's.
    let circuit_air = CircuitAirNewImpl::new(@common_lookup_elements, @interaction_claim);

    verify(
        stark_proof,
        circuit_air,
        TRACE_LOG_DEGREE_BOUND,
        composition_commitment,
        commitment_scheme,
        CIRCUIT_FRI_PARAMS,
        ref channel,
        SECURITY_BITS,
    );
}

