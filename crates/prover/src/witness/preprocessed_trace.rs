use std::sync::Arc;

use stwo::core::channel::MerkleChannel;
use stwo::core::fields::m31::BaseField;
use stwo::core::poly::circle::CanonicCoset;
use stwo::core::vcs_lifted::Hasher;
use stwo::prover::CommitmentTreeProver;
use stwo::prover::backend::BackendForChannel;
use stwo::prover::backend::simd::SimdBackend;
use stwo::prover::mempool::BaseColumnPool;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::{CircleEvaluation, PolyOps};
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::{
    PreProcessedTrace, PreProcessedTraceVariant,
};

use crate::prover::warm_pedersen_pp_trace;

/// Generates the root of the preprocessed trace commitment tree.
///
/// log_trace_size - The size to which we lift all columns (preprocessed and base trace)
/// before the blowup.
pub fn generate_preprocessed_commitment_root<MC: MerkleChannel>(
    log_blowup_factor: u32,
    preprocessed_trace: PreProcessedTraceVariant,
<<<<<<< HEAD
    lifting_log_size: u32,
) -> <<MC as MerkleChannel>::H as Hasher>::Hash
||||||| 167116f40
    lifting_log_size: u32,
) -> <<MC as MerkleChannel>::H as MerkleHasherLifted>::Hash
=======
    log_trace_size: u32,
) -> <<MC as MerkleChannel>::H as MerkleHasherLifted>::Hash
>>>>>>> origin/sharp-8.0
where
    SimdBackend: BackendForChannel<MC>,
{
    warm_pedersen_pp_trace(preprocessed_trace);
    let preprocessed_trace = Arc::new(preprocessed_trace.to_preprocessed_trace());

    // Precompute twiddles covering the largest evaluation domain in use: the biggest column's
    // extended domain log size, or the `lifting_log_size`.
    let max_column_log_size = preprocessed_trace.log_sizes().into_iter().max().unwrap();

    assert!(
        max_column_log_size <= log_trace_size,
        "log_trace_size={log_trace_size} should include the preprocessed trace, but preprocessed \
         trace has column of size {max_column_log_size}"
    );
    let lifting_log_size = log_trace_size + log_blowup_factor;

    let twiddles = SimdBackend::precompute_twiddles(
        CanonicCoset::new(max_column_log_size + log_blowup_factor).circle_domain().half_coset,
    );

    // Generate the commitment tree.
    let polys = SimdBackend::interpolate_columns(gen_trace(preprocessed_trace), &twiddles);
    let commitment_scheme = CommitmentTreeProver::<SimdBackend, MC>::new(
        polys,
        log_blowup_factor,
        &twiddles,
        false,
        lifting_log_size,
        &BaseColumnPool::new(),
    );

    commitment_scheme.commitment.root()
}

pub fn gen_trace(
    preprocessed_trace: Arc<PreProcessedTrace>,
) -> Vec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
    preprocessed_trace.columns.iter().map(|c| c.gen_column_simd()).collect()
}
