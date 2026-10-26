use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use indexmap::IndexMap;
use itertools::zip_eq;
use stwo::core::fields::qm31::SECURE_EXTENSION_DEGREE;
use stwo::core::fri::FriConfig;
use stwo::core::pcs::PcsConfig;

use crate::constraint_eval::CircuitEval;
use crate::fri_proof::{FriProof, empty_fri_proof};
use crate::merkle::{AuthPath, AuthPaths};
use crate::oods::{EvalDomainSamples, N_COMPOSITION_COLUMNS, empty_eval_domain_samples};

pub const N_TRACES: usize = 4;

#[derive(Debug, PartialEq, Clone)]
pub struct ComponentShape {
    pub trace_columns: usize,
    pub interaction_columns: usize,
}

/// Represents the structure of a proof.
#[derive(Debug, PartialEq, Clone)]
pub struct ProofConfig {
    // TODO(lior): Add a check on the total security bits of the protocol given parameters
    //   such as `fri.pow_bits`, `fri.n_queries`, etc.
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

    /// Log2 of the trace size.
    pub log_trace_size: usize,

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

        // The circuit verifier verifies every tree against a single evaluation domain, so it
        // only accepts proofs whose trees — the preprocessed one included — were lifted to the
        // same height.
        let PcsConfig { fri_config, trace_lifting_log_size, preprocessed_lifting_log_size } =
            pcs_config;
        assert_eq!(
            trace_lifting_log_size, preprocessed_lifting_log_size,
            "The circuit verifier expects every tree, the preprocessed one included, to be lifted \
             to the same size"
        );
        let log_trace_size =
            trace_lifting_log_size.checked_sub(fri_config.log_blowup_factor).expect(
                "The circuit verifier expects trace_lifting_log_size to be log_trace_size + \
                 log_blowup_factor",
            ) as usize;

        Self {
            n_interaction_pow_bits,
            n_preprocessed_columns,
            n_trace_columns,
            n_interaction_columns,
            component_shapes,
            cumulative_sum_columns,
            log_trace_size,
            fri: *fri_config,
        }
    }

    /// Returns the number of active components in the AIR.
    pub fn n_components(&self) -> usize {
        self.component_shapes.len()
    }

    /// Returns the log2 of the size of the trace.
    pub fn log_trace_size(&self) -> usize {
        self.log_trace_size
    }

    /// Returns the log2 of the size of the evaluation domain.
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.log_trace_size + self.fri.log_blowup_factor as usize
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

    pub oods_pow_nonce: T,
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
            oods_pow_nonce: _,
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
        fri.validate_structure(config.log_trace_size, &config.fri);
    }

    /// Returns the 3 witness Merkle roots (trace, interaction, composition polynomial).
    /// The preprocessed root is excluded since it is not stored in the proof.
    pub fn merkle_roots(&self) -> [&HashValue<T>; N_TRACES - 1] {
        [&self.trace_root, &self.interaction_root, &self.composition_polynomial_root]
    }
}

pub fn empty_proof(config: &ProofConfig) -> Proof<NoValue> {
    let auth_path = AuthPath(vec![HashValue::no_value(); config.log_evaluation_domain_size()]);

    let n_components = config.n_components();
    Proof {
        trace_root: HashValue::no_value(),
        interaction_root: HashValue::no_value(),
        composition_polynomial_root: HashValue::no_value(),
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
        oods_pow_nonce: NoValue,
        pow_nonce: NoValue,
        interaction_pow_nonce: NoValue,
        fri: empty_fri_proof(config.log_trace_size, &config.fri),
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
            oods_pow_nonce: self.oods_pow_nonce.guess(context),
            pow_nonce: self.pow_nonce.guess(context),
            interaction_pow_nonce: self.interaction_pow_nonce.guess(context),
            fri: self.fri.guess(context),
            channel_salt: self.channel_salt.guess(context),
        }
    }
}
