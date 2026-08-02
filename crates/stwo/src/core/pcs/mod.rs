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
use super::channel::Channel;
use super::fields::qm31::SecureField;
use super::fri::FriConfig;

#[derive(Copy, Debug, Clone, PartialEq, Eq)]
pub struct TreeSubspan {
    pub tree_index: usize,
    pub col_start: usize,
    pub col_end: usize,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
/// Configuration parameters for the commitment scheme prover.
pub struct PcsConfig {
    pub fri_config: FriConfig,
    /// The log size of the lifting domain (includes the `log_blowup_factor`). Each tree is
    /// committed with height `lifting_log_size`, and every column within a tree is lifted to
    /// this size regardless of its own domain. Must be at least the log size of the largest
    /// (extended) domain committed across the trees.
    pub lifting_log_size: u32,
}
impl PcsConfig {
    pub fn mix_into(&self, channel: &mut impl Channel) {
        let PcsConfig { fri_config, lifting_log_size } = self;
        let FriConfig {
            pow_bits,
            log_blowup_factor,
            n_queries,
            log_last_layer_degree_bound,
            fold_step,
        } = fri_config;

        channel.mix_felts(&[
            SecureField::from_u32_unchecked(
                *pow_bits,
                *log_blowup_factor,
                *n_queries as u32,
                *log_last_layer_degree_bound,
            ),
            SecureField::from_u32_unchecked(*fold_step, *lifting_log_size, 0, 0),
        ]);
    }
}
