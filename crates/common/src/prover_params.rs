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

    // TODO(Omri) - Replace lifting_log_size and raise_min_lifting_to_max_column with
    // 'LiftingSizePolicy' enum.
    /// The log size of the lifting domain (includes the `log_blowup_factor`).
    pub lifting_log_size: u32,
    /// If `true`, after writing the trace the prover raises `lifting_log_size` to
    /// the maximal committed column log size over all trees (including the preprocessed tree).
    /// The updated config thus lifts all trees to the same height. Otherwise, the given
    /// `lifting_log_size` is used as is. Default is `false`.
    #[serde(default)]
    pub raise_min_lifting_to_max_column: bool,
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
