//! Proof size breakdown computed from a [`ProofConfig`], for tests and size reports.

use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;

use crate::fri_proof::compute_all_fold_steps;
use crate::proof::{N_TRACES, ProofConfig};

const N_U8S_PER_U32: usize = 4;

/// Proof size breakdown, measured in `u8` elements.
///
/// Most of the proof consists of M31 values, but the serialized claim includes raw `u8` values.
///
/// Fields that scale with `n_queries` store the per-query cost; use `total_bytes()` for the full
/// size.
#[derive(Debug, Clone)]
pub struct ProofInfo {
    pub log_trace_size: usize,
    pub log_blowup_factor: usize,
    pub n_queries: usize,
    pub n_columns_per_trace: [usize; N_TRACES],
    // Fixed scalars: channel_salt (QM31) + 3 roots (HashValue, 32 bytes each) +
    // oods_pow_nonce (QM31) + pow_nonce (QM31) + interaction_pow_nonce (QM31).
    pub fixed: usize,
    // Claim (serialized in packed format): enable bits + log sizes + claimed sums.
    pub claim: usize,
    // OODS evaluations (QM31 per column, plus cumulative sum prev-point samples).
    pub oods: usize,
    // FRI commitments: one HashValue (32 bytes each) per layer.
    pub fri_commitments: usize,
    // FRI last layer coefs (QM31).
    pub fri_last_layer: usize,
    // Per-query fields (total = per_query * n_queries).
    // Eval domain samples per query: M31 values, kept as-is.
    pub eval_samples_per_query: usize,
    // Eval domain auth paths per query: N_TRACES trees (all lifted/split to eval domain size),
    // each path of depth log_eval_domain, each node is a full HashValue (8 words).
    pub eval_auth_per_query: usize,
    // FRI auth paths per query.
    pub fri_auth_per_query: usize,
    // FRI witness per query: each fold layer has 2^step QM31 coset values.
    pub fri_witness_per_query: usize,
}

impl ProofInfo {
    /// Returns the proof size breakdown in u8s, computed from config alone.
    pub fn from_config(config: &ProofConfig) -> Self {
        let n_queries = config.fri.n_queries;
        let log_eval_domain = config.log_evaluation_domain_size();

        let fixed = (1 + 3 * 2 + 1 + 1 + 1) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let claim = config.n_components() * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let n_columns_per_trace = config.n_columns_per_trace();
        let total_columns: usize = n_columns_per_trace.iter().sum();

        // The number of cumulative sum columns that have two mask values per columns.
        let n_cumsum = config.cumulative_sum_columns.iter().filter(|&&b| b).count();
        let oods = (total_columns + n_cumsum) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let eval_samples_per_query = total_columns * N_U8S_PER_U32;

        let hash_size = 2 * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let eval_auth_per_query = N_TRACES * log_eval_domain * hash_size;

        let degree_log_ratio =
            config.log_trace_size - config.fri.log_last_layer_degree_bound as usize;
        let all_fold_steps =
            compute_all_fold_steps(degree_log_ratio, config.fri.fold_step as usize);
        let n_fri_layers = all_fold_steps.len();

        let fri_commitments = n_fri_layers * hash_size;

        let fri_last_layer =
            (1 << config.fri.log_last_layer_degree_bound) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let mut log_layer_size = log_eval_domain;
        let fri_auth_per_query: usize = all_fold_steps
            .iter()
            .map(|step| {
                let depth = log_layer_size - step;
                let count = depth * hash_size;
                log_layer_size -= step;
                count
            })
            .sum();

        let fri_witness_per_query: usize = all_fold_steps
            .iter()
            .map(|step| (1 << step) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32)
            .sum();

        let log_trace_size = config.log_trace_size;
        let log_blowup_factor = config.fri.log_blowup_factor as usize;

        Self {
            log_trace_size,
            log_blowup_factor,
            n_queries,
            n_columns_per_trace,
            fixed,
            claim,
            oods,
            fri_commitments,
            fri_last_layer,
            eval_samples_per_query,
            eval_auth_per_query,
            fri_auth_per_query,
            fri_witness_per_query,
        }
    }

    /// Returns the total size in bytes.
    pub fn total_bytes(&self) -> usize {
        let Self {
            log_trace_size: _,
            log_blowup_factor: _,
            n_queries,
            n_columns_per_trace: _,
            fixed,
            claim,
            oods,
            fri_commitments,
            fri_last_layer,
            eval_samples_per_query,
            eval_auth_per_query,
            fri_auth_per_query,
            fri_witness_per_query,
        } = *self;
        fixed
            + claim
            + oods
            + fri_commitments
            + fri_last_layer
            + (eval_samples_per_query
                + eval_auth_per_query
                + fri_auth_per_query
                + fri_witness_per_query)
                * n_queries
    }
}

impl std::fmt::Display for ProofInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            log_trace_size,
            log_blowup_factor,
            n_queries,
            n_columns_per_trace,
            fixed,
            claim,
            oods,
            fri_commitments,
            fri_last_layer,
            eval_samples_per_query,
            eval_auth_per_query,
            fri_auth_per_query,
            fri_witness_per_query,
        } = *self;
        let [p, t, i, c] = n_columns_per_trace;
        writeln!(f, "Proof info:")?;
        writeln!(f, "  log_trace_size:       {log_trace_size:>10}")?;
        writeln!(f, "  log_blowup_factor:    {log_blowup_factor:>10}")?;
        writeln!(f, "  n_queries:            {n_queries:>10}")?;
        let total_columns = p + t + i + c;
        writeln!(f, "  n_columns_per_trace:  {total_columns:>10} = [{p}, {t}, {i}, {c}]")?;
        writeln!(f)?;
        writeln!(f, "Proof size breakdown (u32 elements)")?;
        writeln!(f, "  fixed:               {fixed:>10}")?;
        writeln!(f, "  claim:               {claim:>10}")?;
        writeln!(f, "  oods:                {oods:>10}")?;
        writeln!(
            f,
            "  eval_samples:        {:>10} = {eval_samples_per_query} * {n_queries}",
            eval_samples_per_query * n_queries,
        )?;
        writeln!(
            f,
            "  eval_auth_paths:     {:>10} = {eval_auth_per_query} * {n_queries}",
            eval_auth_per_query * n_queries,
        )?;
        writeln!(f, "  fri_commitments:     {fri_commitments:>10}")?;
        writeln!(f, "  fri_last_layer:      {fri_last_layer:>10}")?;
        writeln!(
            f,
            "  fri_auth:            {:>10} = {fri_auth_per_query} * {n_queries}",
            fri_auth_per_query * n_queries,
        )?;
        writeln!(
            f,
            "  fri_witness:         {:>10} = {fri_witness_per_query} * {n_queries}",
            fri_witness_per_query * n_queries,
        )?;
        writeln!(f, "  ─────────────────────────────")?;
        let total = self.total_bytes();
        writeln!(f, "  total (u32 elements): {:>10}", total / 4)?;
        writeln!(f, "  total (bytes):        {total:>10}")
    }
}
