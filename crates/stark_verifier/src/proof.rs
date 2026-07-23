use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use circuits::wrappers::U32Wrapper;
use indexmap::IndexMap;
use itertools::{Itertools, zip_eq};
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::pcs::PcsConfig;

use crate::channel::Channel;
use crate::constraint_eval::CircuitEval;
use crate::fri_proof::{FriConfig, FriProof, compute_all_fold_steps, empty_fri_proof};
use crate::merkle::{AuthPath, AuthPaths};
use crate::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS, empty_eval_domain_samples};
use crate::proof_from_stark_proof::pack_into_qm31s;

pub const N_TRACES: usize = 4;
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
    // Fixed scalars: channel_salt (QM31) + 3 roots (HashValue, 32 bytes each) + pow_nonce (QM31) +
    // interaction_pow_nonce (QM31).
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
        let log_eval_domain = config.fri.log_evaluation_domain_size();

        let fixed = (1 + 3 * 2 + 1 + 1) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let claim = config.n_components() * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let n_columns_per_trace = config.n_columns_per_trace();
        let total_columns: usize = n_columns_per_trace.iter().sum();

        // The number of cumulative sum columns that have two mask values per columns.
        let n_cumsum = config.cumulative_sum_columns.iter().filter(|&&b| b).count();
        let oods = (total_columns + n_cumsum) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let eval_samples_per_query = total_columns * N_U8S_PER_U32;

        let hash_size = 2 * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

        let eval_auth_per_query = N_TRACES * log_eval_domain * hash_size;

        let degree_log_ratio = config.fri.log_trace_size - config.fri.log_n_last_layer_coefs;
        let all_fold_steps = compute_all_fold_steps(degree_log_ratio, config.fri.fold_step);
        let n_fri_layers = all_fold_steps.len();

        let fri_commitments = n_fri_layers * hash_size;

        let fri_last_layer =
            (1 << config.fri.log_n_last_layer_coefs) * SECURE_EXTENSION_DEGREE * N_U8S_PER_U32;

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

        let log_trace_size = config.fri.log_trace_size;
        let log_blowup_factor = config.fri.log_blowup_factor;

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

#[derive(Debug, PartialEq, Clone)]
pub struct ComponentShape {
    pub trace_columns: usize,
    pub interaction_columns: usize,
}

/// Represents the structure of a proof.
#[derive(Debug, PartialEq, Clone)]
pub struct ProofConfig {
    // TODO(lior): Add a check on the total security bits of the protocol given parameters
    //   such as `n_pow_bits`, `fri.n_queries`, etc.
    pub n_pow_bits: u32,
    pub n_interaction_pow_bits: u32,

    // AIR structure.
    pub n_preprocessed_columns: usize,
    pub n_trace_columns: usize,
    pub n_interaction_columns: usize,
    // One entry per component, in the order of the components map passed to `new`. This must match
    // the order in which the statement iterates its components in
    // `compute_composition_polynomial`; callers ensure this by building this config from the same
    // components map the statement uses. Note this is not necessarily the committed-column order:
    // when `Statement::sorting_required` is `true` the verifier sorts the query columns to the
    // committed order during decommitment instead.
    pub component_shapes: Vec<ComponentShape>,

    // Per column in the interaction trace, an indicator of whether it is a cumulative sum column.
    // This is used to determine whether to include a sample point at the previous point in the
    // OODS response.
    pub cumulative_sum_columns: Vec<bool>,

    pub fri: FriConfig,
}
impl ProofConfig {
    pub fn new<Value: IValue>(
        components: &IndexMap<&'static str, Box<dyn CircuitEval<Value>>>,
        n_preprocessed_columns: usize,
        pcs_config: &PcsConfig,
        n_interaction_pow_bits: u32,
    ) -> Self {
        let mut component_shapes = Vec::with_capacity(components.len());
        for component in components.values() {
            component_shapes.push(ComponentShape {
                trace_columns: component.trace_columns(),
                interaction_columns: component.interaction_columns(),
            });
        }

        let n_trace_columns: usize = component_shapes.iter().map(|info| info.trace_columns).sum();
        let n_interaction_columns: usize =
            component_shapes.iter().map(|info| info.interaction_columns).sum();
        let mut cumulative_sum_columns = Vec::with_capacity(n_interaction_columns);
        for info in &component_shapes {
            // The last SECURE_EXTENSION_DEGREE interaction columns of every component are
            // cumulative sum columns.
            assert!(
                info.interaction_columns >= SECURE_EXTENSION_DEGREE,
                "Expected at least {SECURE_EXTENSION_DEGREE} interaction columns per component"
            );
            cumulative_sum_columns
                .extend(vec![false; info.interaction_columns - SECURE_EXTENSION_DEGREE]);
            cumulative_sum_columns.extend(vec![true; SECURE_EXTENSION_DEGREE]);
        }

        let PcsConfig {
            pow_bits,
            fri_config:
                stwo::core::fri::FriConfig {
                    log_blowup_factor,
                    n_queries,
                    log_last_layer_degree_bound,
                    fold_step,
                },
            min_lifting_log_size,
        } = pcs_config;
        let log_trace_size = min_lifting_log_size.checked_sub(*log_blowup_factor).expect(
            "The circuit verifier expects min_lifting_log_size to be log_trace_size + \
             log_blowup_factor",
        ) as usize;

        Self {
            n_pow_bits: *pow_bits,
            n_interaction_pow_bits,
            n_preprocessed_columns,
            n_trace_columns,
            n_interaction_columns,
            component_shapes,
            cumulative_sum_columns,
            fri: FriConfig {
                log_trace_size,
                log_blowup_factor: *log_blowup_factor as usize,
                n_queries: *n_queries,
                log_n_last_layer_coefs: *log_last_layer_degree_bound as usize,
                fold_step: *fold_step as usize,
            },
        }
    }

    /// Returns the number of active components in the AIR.
    pub fn n_components(&self) -> usize {
        self.component_shapes.len()
    }

    /// Returns the log2 of the size of the trace.
    pub fn log_trace_size(&self) -> usize {
        self.fri.log_trace_size
    }

    /// Returns the log2 of the size of the evaluation domain.
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.fri.log_evaluation_domain_size()
    }

    /// Returns the number of queries.
    pub fn n_queries(&self) -> usize {
        self.fri.n_queries
    }

    /// Returns the number of columns for each of the traces.
    pub fn n_columns_per_trace(&self) -> [usize; N_TRACES] {
        [
            self.n_preprocessed_columns,
            self.n_trace_columns,
            self.n_interaction_columns,
            N_COMPOSITION_COLUMNS,
        ]
    }

    /// Mixes the pcs config into the channel.
    pub fn mix_pcs_config<Value: IValue>(
        &self,
        context: &mut Context<Value>,
        channel: &mut Channel,
    ) {
        let lifting_log_size = self.log_trace_size() + self.fri.log_blowup_factor;
        let pcs_config_values = vec![
            self.n_pow_bits,
            self.fri.log_blowup_factor as u32,
            self.fri.n_queries as u32,
            self.fri.log_n_last_layer_coefs as u32,
            self.fri.fold_step as u32,
            lifting_log_size as u32,
        ];

        let pcs_config_vars = pack_into_qm31s(pcs_config_values.into_iter())
            .into_iter()
            .map(|qm31| context.constant(qm31))
            .collect_vec();
        channel.mix_qm31s(context, pcs_config_vars);
    }
}

/// The values of an interaction column at the OODS point and the previous point.
#[derive(Clone, Debug, PartialEq)]
pub struct InteractionAtOods<T> {
    /// The value at the OODS point and optionally the value at the previous point
    /// (`oods_point - trace_generator`).
    pub at_oods: T,
    pub at_prev: Option<T>,
}

impl<Value: IValue> Guess<Value> for InteractionAtOods<Value> {
    type Target = InteractionAtOods<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        InteractionAtOods {
            at_oods: self.at_oods.guess(context),
            at_prev: self.at_prev.map(|at_prev| at_prev.guess(context)),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Proof<T> {
    pub channel_salt: T,

    // Merkle roots.
    pub trace_root: HashValue<T>,
    pub interaction_root: HashValue<T>,
    pub composition_polynomial_root: HashValue<T>,

    // Claimed sum for each component in the AIR.
    pub claimed_sums: Vec<T>,

    // Evaluations at the OODS point and the previous point.
    pub preprocessed_columns_at_oods: Vec<T>,
    pub trace_at_oods: Vec<T>,
    pub interaction_at_oods: Vec<InteractionAtOods<T>>,
    pub composition_eval_at_oods: [T; N_COMPOSITION_COLUMNS],

    // Evaluations at the evaluation domain.
    pub eval_domain_samples: EvalDomainSamples<T>,
    pub eval_domain_auth_paths: AuthPaths<T>,

    pub pow_nonce: T,
    pub interaction_pow_nonce: T,
    pub fri: FriProof<T>,
}
impl<T> Proof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &ProofConfig) {
        let Self {
            channel_salt: _,
            trace_root: _,
            interaction_root: _,
            composition_polynomial_root: _,
            claimed_sums,
            preprocessed_columns_at_oods,
            trace_at_oods,
            interaction_at_oods,
            composition_eval_at_oods,
            eval_domain_samples,
            eval_domain_auth_paths,
            pow_nonce: _,
            interaction_pow_nonce: _,
            fri,
        } = self;

        assert_eq!(claimed_sums.len(), config.n_components());

        // Validate preprocessed_columns_at_oods.
        assert_eq!(preprocessed_columns_at_oods.len(), config.n_preprocessed_columns);

        // Validate trace_at_oods.
        assert_eq!(trace_at_oods.len(), config.n_trace_columns);

        // Validate interaction_at_oods.
        assert_eq!(interaction_at_oods.len(), config.n_interaction_columns);
        for (interaction_at_oods, is_cumulative_sum) in
            zip_eq(interaction_at_oods, &config.cumulative_sum_columns)
        {
            assert_eq!(interaction_at_oods.at_prev.is_some(), *is_cumulative_sum);
        }

        // This check is trivially true because composition_eval_at_oods has a fixed length, but we
        // keep it for completeness.
        assert_eq!(composition_eval_at_oods.len(), N_COMPOSITION_COLUMNS);

        // Validate eval_domain_samples.
        eval_domain_samples.validate_structure(&config.n_columns_per_trace(), config.n_queries());

        let tree_heights = [config.log_evaluation_domain_size(); N_TRACES];
        eval_domain_auth_paths.validate_structure(&tree_heights, config.n_queries());

        // Validate FRI.
        fri.validate_structure(&config.fri);
    }

    /// Returns the 3 witness Merkle roots (trace, interaction, composition polynomial).
    /// The preprocessed root is excluded since it is not stored in the proof.
    pub fn merkle_roots(&self) -> [HashValue<T>; N_TRACES - 1]
    where
        T: Clone,
    {
        [
            self.trace_root.clone(),
            self.interaction_root.clone(),
            self.composition_polynomial_root.clone(),
        ]
    }
}

pub fn empty_proof(config: &ProofConfig) -> Proof<NoValue> {
    let auth_path = AuthPath(vec![HashValue::no_value(); config.log_evaluation_domain_size()]);

    let n_components = config.n_components();
    Proof {
        trace_root: HashValue([U32Wrapper::new_unsafe(NoValue); 8]),
        interaction_root: HashValue([U32Wrapper::new_unsafe(NoValue); 8]),
        composition_polynomial_root: HashValue([U32Wrapper::new_unsafe(NoValue); 8]),
        preprocessed_columns_at_oods: vec![NoValue; config.n_preprocessed_columns],
        trace_at_oods: vec![NoValue; config.n_trace_columns],
        interaction_at_oods: config
            .cumulative_sum_columns
            .iter()
            .map(|is_cumulative_sum| {
                if *is_cumulative_sum {
                    InteractionAtOods { at_oods: NoValue, at_prev: Some(NoValue) }
                } else {
                    InteractionAtOods { at_oods: NoValue, at_prev: None }
                }
            })
            .collect(),
        claimed_sums: vec![NoValue; n_components],
        composition_eval_at_oods: [NoValue; N_COMPOSITION_COLUMNS],
        eval_domain_samples: empty_eval_domain_samples(
            &config.n_columns_per_trace(),
            config.n_queries(),
        ),
        eval_domain_auth_paths: AuthPaths {
            data: vec![vec![auth_path; config.n_queries()]; N_TRACES],
        },
        pow_nonce: NoValue,
        interaction_pow_nonce: NoValue,
        fri: empty_fri_proof(&config.fri),
        channel_salt: NoValue,
    }
}

impl<Value: IValue> Guess<Value> for Proof<Value> {
    type Target = Proof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Proof {
            trace_root: self.trace_root.guess(context),
            interaction_root: self.interaction_root.guess(context),
            composition_polynomial_root: self.composition_polynomial_root.guess(context),
            claimed_sums: self.claimed_sums.guess(context),
            preprocessed_columns_at_oods: self.preprocessed_columns_at_oods.guess(context),
            trace_at_oods: self.trace_at_oods.guess(context),
            interaction_at_oods: self.interaction_at_oods.guess(context),
            composition_eval_at_oods: self.composition_eval_at_oods.guess(context),
            eval_domain_samples: self.eval_domain_samples.guess(context),
            eval_domain_auth_paths: self.eval_domain_auth_paths.guess(context),
            pow_nonce: self.pow_nonce.guess(context),
            interaction_pow_nonce: self.interaction_pow_nonce.guess(context),
            fri: self.fri.guess(context),
            channel_salt: self.channel_salt.guess(context),
        }
    }
}
