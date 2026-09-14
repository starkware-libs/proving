//! Implements a FRI polynomial commitment scheme.
//!
//! This is a protocol where the prover can commit on a set of polynomials and then prove their
//! opening on a set of points.
//! Note: This implementation is not really a polynomial commitment scheme, because we are not in
//! the unique decoding regime. This is enough for a STARK proof though, where we only want to imply
//! the existence of such polynomials, and are ok with having a small decoding list.
//! Note: Opened points cannot come from the commitment domain.

pub mod quotients;
pub mod utils;
mod verifier;

use serde::{Deserialize, Serialize};

pub use self::utils::TreeVec;
pub use self::verifier::CommitmentSchemeVerifier;
use super::fri::FriConfig;
use super::verifier::PREPROCESSED_TRACE_IDX;

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct TreeSubspan {
    pub tree_index: usize,
    pub col_start: usize,
    pub col_end: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
/// Configuration parameters for the commitment scheme prover.
///
/// Deliberately not [`Default`]: the lifting log sizes depend on the trees being committed, so
/// there is no sane default for them. Build one with [`PcsConfig::from_fri_and_trace_size`],
/// [`PcsConfig::from_fri_and_lifting_size`], or a struct literal.
///
/// The lifting log sizes are the heights the trees are committed at: every column in a tree is
/// lifted to its tree's height, which includes `fri_config.log_blowup_factor` and must dominate
/// that tree's extended columns.
///
/// Only `fri_config` is mixed into the channel (see [`FriConfig::mix_into`]): the lifting log
/// sizes are intentionally left out, as no verifier reads them off the proof. The Cairo verifier
/// recomputes each tree's height from `fri_config.log_blowup_factor` and the committed columns'
/// log sizes, and the circuit verifier has them hardcoded for its topology.
pub struct PcsConfig {
    pub fri_config: FriConfig,
    /// The height of every committed tree but the preprocessed one.
    pub trace_lifting_log_size: u32,
    /// The height of the preprocessed tree, tree [`PREPROCESSED_TRACE_IDX`].
    pub preprocessed_lifting_log_size: u32,
}
impl PcsConfig {
    /// The config for proving a trace of `trace_log_size` under `fri_config`: every tree, the
    /// preprocessed one included, is lifted to the trace's extended domain.
    pub const fn from_fri_and_trace_size(fri_config: FriConfig, trace_log_size: u32) -> Self {
        Self::from_fri_and_lifting_size(fri_config, trace_log_size + fri_config.log_blowup_factor)
    }

    /// The config lifting every tree, the preprocessed one included, to `lifting_log_size`
    /// (which already includes the `log_blowup_factor`).
    pub const fn from_fri_and_lifting_size(fri_config: FriConfig, lifting_log_size: u32) -> Self {
        Self {
            fri_config,
            trace_lifting_log_size: lifting_log_size,
            preprocessed_lifting_log_size: lifting_log_size,
        }
    }

    /// The height the `tree_index`-th committed tree is lifted to: the preprocessed tree —
    /// tree [`PREPROCESSED_TRACE_IDX`] — to [`Self::preprocessed_lifting_log_size`], every other
    /// tree to [`Self::trace_lifting_log_size`].
    pub const fn lifting_log_size(&self, tree_index: usize) -> u32 {
        if tree_index == PREPROCESSED_TRACE_IDX {
            self.preprocessed_lifting_log_size
        } else {
            self.trace_lifting_log_size
        }
    }
}
