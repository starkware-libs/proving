//! `stwo_circuit_air`: AIR-specific verifier-side logic written in Cairo for the stwo-circuits
//! circuit.
use circuit_air::CircuitAirNewImpl;
use core::num::traits::Zero;
use multiverifier_consts::{N_OUTPUTS, circuit_fri_config};
use stwo_constraint_framework::LookupElementsImpl;
pub use stwo_constraint_framework::{RelationUse, RelationUsesDict, accumulate_relation_uses};
use stwo_verifier_core::Hash;
use stwo_verifier_core::channel::{Channel, ChannelTrait};
use stwo_verifier_core::fields::m31::{M31Trait, P_U32};
use stwo_verifier_core::fields::qm31::{QM31, QM31Serde, QM31Trait};
use stwo_verifier_core::fri::FriConfigTrait;
use stwo_verifier_core::pcs::verifier::CommitmentSchemeVerifierImpl;
use stwo_verifier_core::utils::SpanExTrait;
use stwo_verifier_core::verifier::{StarkProof, VerificationError, verify};
use stwo_verifier_utils::blake2s::hash_u32s;

pub mod circuit_air;

pub mod claims;
pub mod per_component;
use claims::{
    CircuitClaim, CircuitClaimImpl, CircuitInteractionClaim, CircuitInteractionClaimImpl,
    column_log_sizes_per_tree, logup_sum,
};
pub mod circuit_hash;
pub use circuit_hash::compute_circuit_hash;
pub mod components;
pub mod multiverifier_consts;
pub mod prelude;
pub mod preprocessed_columns;
pub mod relations;

// Security constants.
pub const INTERACTION_POW_BITS: u32 = 20;
const SECURITY_BITS: u32 = 96;

pub const P_U32_CONST: u32 = P_U32;

#[derive(Drop, Serde)]
pub struct CircuitProof {
    pub claim: CircuitClaim,
    pub interaction_pow: u64,
    pub interaction_claim: CircuitInteractionClaim,
    pub stark_proof: StarkProof,
    /// Salt used in the channel initialization.
    pub channel_salt: u32,
}

/// The output of a circuit verification: `blake2s(circuit_hash || output_words)` (see
/// `get_verification_output`).
#[derive(Drop, Serde)]
pub struct VerificationOutput {
    pub output_hash: Hash,
}

/// The u32 wire encoding's limb bound and shift: each limb is a u16.
const U16_SHIFT: u32 = 0x10000;

/// Returns the output of the verifier: `blake2s(circuit_hash || output_words)`, where each
/// output value is the circuit's wire encoding `(low_u16, high_u16, 0, 0)` of one u32 word and
/// contributes the recombined word.
pub fn get_verification_output(
    circuit_hash: Hash, output_values: Span<QM31>,
) -> VerificationOutput {
    let [h0, h1, h2, h3, h4, h5, h6, h7] = circuit_hash.hash.unbox();
    let mut words = array![h0, h1, h2, h3, h4, h5, h6, h7];
    for value in output_values {
        let [lo, hi, c2, c3] = (*value).to_fixed_array();
        let lo: u32 = lo.into();
        let hi: u32 = hi.into();
        let c2: u32 = c2.into();
        let c3: u32 = c3.into();
        assert!(
            lo < U16_SHIFT && hi < U16_SHIFT && c2 == 0 && c3 == 0,
            "circuit output value is not a packed u32",
        );
        words.append(lo + hi * U16_SHIFT);
    }
    VerificationOutput { output_hash: Hash { hash: hash_u32s(words.span()) } }
}

pub fn verify_circuit(proof: CircuitProof, circuit_hash: Hash) {
    let CircuitProof {
        claim, interaction_pow, interaction_claim, stark_proof, channel_salt,
    } = proof;

    // The circuit produces a fixed number of public outputs (its topology); the claim must
    // provide exactly that many.
    assert!(claim.output_values.len() == N_OUTPUTS);

    // Pin the proof's PCS config to the circuit's hardcoded canonical config. This rejects any
    // proof produced with weaker/mismatched FRI parameters.
    let fri_config = stark_proof.commitment_scheme_proof.config;
    assert!(fri_config == circuit_fri_config(), "unexpected proof fri config");

    let mut channel: Channel = Default::default();
    // Mix channel salt. Note that we first reduce it modulo `M31::P`, then cast it as QM31.
    let channel_salt_as_felt: QM31 = M31Trait::reduce_u32(channel_salt).into();
    channel.mix_felts([channel_salt_as_felt].span());

    fri_config.mix_into(ref channel);
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

    let log_blowup_factor = fri_config.log_blowup_factor;

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
        channel.verify_pow_nonce(INTERACTION_POW_BITS, interaction_pow),
        "{}",
        VerificationError::InteractionProofOfWork,
    );
    channel.mix_u64(interaction_pow);

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
    let trace_log_degree_bound = *preprocessed_column_log_sizes.max().unwrap();
    let circuit_air = CircuitAirNewImpl::new(@common_lookup_elements, @interaction_claim);

    verify(
        stark_proof,
        circuit_air,
        trace_log_degree_bound,
        composition_commitment,
        commitment_scheme,
        ref channel,
        SECURITY_BITS,
    );
}

