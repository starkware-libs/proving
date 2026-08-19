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

    /// Policy for choosing the heights the committed trees are lifted to. The right choice
    /// depends on the downstream verifier; see the [`LiftingSizePolicy`] variants.
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

/// Policy for choosing the heights the committed trees are lifted to.
///
/// Each variant fixes a pair: the height the trace trees are lifted to and the height the
/// preprocessed tree is lifted to — the proof's `PcsConfig::trace_lifting_log_size` and
/// `PcsConfig::preprocessed_lifting_log_size`, which the verifier commits with.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LiftingSizePolicy {
    /// Use the trace height ignoring preprocessed trace size.
    /// Intended for proofs targeted at the cairo verifier.
    ///
    /// The trace trees are lifted to the trace's own domain and the preprocessed tree to its
    /// own, so the two heights differ whenever the preprocessed trace is not the tallest.
    Auto,
    /// Use the given lifting log size.
    /// Intended for proofs targeted at the circuit-2-to-1 verifier.
    ///
    /// Every tree, the preprocessed one included, is lifted to the given size, so the two
    /// heights coincide.
    Fixed(u32),
    /// Use the trace height including preprocessed trace size.
    /// Intended for proofs targeted at the circuit-cairo verifier.
    ///
    /// Every tree, the preprocessed one included, is lifted to the taller of the two domains, so
    /// the two heights coincide as they do under [`Self::Fixed`] — the circuit verifies all
    /// trees against a single evaluation domain. Unlike `Fixed` the height follows the trace
    /// rather than being named up front, so it needs no precomputed preprocessed tree.
    AtLeastPreprocessed,
}
