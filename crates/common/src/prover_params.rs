//! The Cairo prover's parameters — everything that describes how to prove a Cairo run (but not
//! what to prove).

use serde::{Deserialize, Serialize};
use stwo::core::fri::FriConfig;

use crate::preprocessed_columns::preprocessed_trace::PreProcessedTraceVariant;

/// Concrete parameters of the proving system.
/// Used both for producing and verifying proofs.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProverParameters {
    /// Channel hash function.
    pub channel_hash: ChannelHash,
    /// Salt for the channel initialization.
    /// Note that the salt is only used to allow recomputation of the proof with other draws
    /// of the randomness, in case of failure due to unprovable draws (e.g. a zero in the
    /// denominator).
    pub channel_salt: u32,
    /// Parameters of the FRI proof.
    pub fri_config: FriConfig,
    /// Preprocessed trace.
    pub preprocessed_trace: PreProcessedTraceVariant,
    /// Whether or not to store the polynomials coefficients. Affects runtime-memory usage
    /// trade-off. Default is `false`.
    pub store_polynomials_coefficients: bool,
    /// Whether to include samples for every preprocessed column in the proof. Default is `false`.
    /// If `false`, the proof only includes samples for columns used by at least one component.
    pub include_all_preprocessed_columns: bool,

    /// Optional number of components for the memory id to big claim.
    /// If not provided, the number of components will be inferred from the input.
    pub opt_n_id_to_big_components: Option<usize>,

    /// Policy for choosing the `lifting_log_size` — the common height to which all committed
    /// trees are lifted. The right choice depends on the downstream verifier; see the
    /// [`LiftingSizePolicy`] variants.
    pub lifting_size_policy: LiftingSizePolicy,
}

/// The hash function used for commitments, for the prover-verifier channel,
/// and for PoW grinding.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChannelHash {
    /// Default variant, the fastest option.
    Blake2s,
    /// A variant for Blake2s where modulo M31 is applied to every 32bits in the output.
    Blake2sM31,
    /// A variant for recursive proof verification.
    /// Note that using `Poseidon252` results in a significant decrease in proving speed compared
    /// to `Blake2s` (because of the large field emulation)
    Poseidon252,
}

/// Policy for choosing the `lifting_log_size` used to lift all committed trees to a common
/// height.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiftingSizePolicy {
    /// Use the trace height ignoring preprocessed trace size.
    /// Intended for proofs targeted at the cairo verifier.
    Auto,
    /// Use the given lifting log size.
    /// Intended for proofs targeted at the circuit-2-to-1 verifier.
    Fixed(u32),
    /// Use the trace height including preprocessed trace size.
    /// Intended for proofs targeted at the circuit-cairo verifier.
    AtLeastPreprocessed,
}

impl LiftingSizePolicy {
    /// Resolves the policy to a concrete `lifting_log_size`.
    ///
    /// - `max_trace_log_size` — the trace-only max committed column log size.
    /// - `preprocessed_trace_log_size` — the preprocessed trace log size.
    ///
    /// Pass `None` for a parameter when it isn't known.
    pub fn resolve(
        &self,
        max_trace_log_size: Option<u32>,
        preprocessed_trace_log_size: Option<u32>,
    ) -> u32 {
        match self {
            Self::Auto => max_trace_log_size
                .expect("LiftingSizePolicy::Auto cannot be resolved without max_trace_log_size"),
            Self::Fixed(size) => *size,
            Self::AtLeastPreprocessed => {
                let preprocessed = preprocessed_trace_log_size.expect(
                    "LiftingSizePolicy::AtLeastPreprocessed requires preprocessed_trace_log_size",
                );
                let max_trace = max_trace_log_size
                    .expect("LiftingSizePolicy::AtLeastPreprocessed requires max_trace_log_size");
                std::cmp::max(preprocessed, max_trace)
            }
        }
    }
}
