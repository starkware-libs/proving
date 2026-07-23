use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::ivalue::{IValue, NoValue};
use circuits::ops::Guess;
use circuits::wrappers::U32Wrapper;
use itertools::zip_eq;

use crate::merkle::{AuthPath, AuthPaths};

/// Represents the structure of a FRI proof.
#[derive(Debug, PartialEq, Clone)]
pub struct FriConfig {
    /// Log2 of the trace size.
    pub log_trace_size: usize,
    /// Log2 of the blowup factor.
    pub log_blowup_factor: usize,
    /// Number of queries.
    pub n_queries: usize,
    /// Log2 of the number of coefficients in the last layer of FRI.
    pub log_n_last_layer_coefs: usize,
    /// The step of the folds in FRI's layers.
    pub fold_step: usize,
}

impl FriConfig {
    pub fn log_evaluation_domain_size(&self) -> usize {
        self.log_trace_size + self.log_blowup_factor
    }
}

/// Represents the information for the FRI commitment phase of the proof.
#[derive(Clone, Debug, PartialEq)]
pub struct FriCommitProof<T> {
    pub layer_commitments: Vec<HashValue<T>>,
    pub last_layer_coefs: Vec<T>,
}

impl<T> FriCommitProof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig, all_fold_steps: &[usize]) {
        // The computation of `all_fold_step` guarantees also that
        // `config.log_trace_size = log_n_last_layer_coefs + ∑ fold_step_for_layer`, where
        // the sum runs over the FRI layers.
        assert_eq!(self.layer_commitments.len(), all_fold_steps.len());
        assert_eq!(self.last_layer_coefs.len(), 1 << config.log_n_last_layer_coefs);
    }
}

impl<Value: IValue> Guess<Value> for FriCommitProof<Value> {
    type Target = FriCommitProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            layer_commitments: self.layer_commitments.guess(context),
            last_layer_coefs: self.last_layer_coefs.guess(context),
        }
    }
}

/// Witness for the FRI decommitment phase.
///
/// For each FRI layer, for each query, the values of the layer polynomial on the FRI witness domain
/// (either a circle domain or a coset), containing that query.
#[derive(Clone, Debug, PartialEq)]
pub struct FriWitness<T>(pub Vec<Vec<Vec<T>>>);

impl<Value: IValue> Guess<Value> for FriWitness<Value> {
    type Target = FriWitness<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        FriWitness(self.0.guess(context))
    }
}

impl<T> FriWitness<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig, all_fold_steps: &[usize]) {
        assert_eq!(self.0.len(), all_fold_steps.len());
        for (witness_per_query, step) in zip_eq(&self.0, all_fold_steps) {
            assert_eq!(witness_per_query.len(), config.n_queries);
            assert!(witness_per_query.iter().all(|witness| witness.len() == 1 << step));
        }
    }
}

/// Represents the information required to verify a FRI proof.
#[derive(Clone, Debug, PartialEq)]
pub struct FriProof<T> {
    /// Information regarding the FRI commitment phase.
    pub commit: FriCommitProof<T>,
    /// Authentication paths for all the FRI trees.
    pub auth_paths: AuthPaths<T>,
    /// Witness for the FRI decommitment phase.
    pub witness: FriWitness<T>,
}

impl<T> FriProof<T> {
    /// Validates that the size of the members of the struct are consistent with the config.
    pub fn validate_structure(&self, config: &FriConfig) {
        let FriProof { commit, auth_paths, witness } = self;
        let all_fold_steps = compute_all_fold_steps(
            config.log_trace_size - config.log_n_last_layer_coefs,
            config.fold_step,
        );
        commit.validate_structure(config, &all_fold_steps);

        // Build `tree_heights` so that `tree_heights[i]` is the Merkle authentication-path
        // length for FRI layer `i`, to validate against the proof.
        // The fold inputs are merklized in cosets of size `2^fold_step` before authentication
        // starts, so a layer with `2^k` values has an auth path of length `k - fold_step`
        // (equivalently, the layer's log-size after folding).
        let mut tree_heights: Vec<usize> = Vec::with_capacity(all_fold_steps.len());
        let mut layer_size = config.log_evaluation_domain_size();
        for fold_step in &all_fold_steps {
            layer_size -= *fold_step;
            tree_heights.push(layer_size);
        }
        auth_paths.validate_structure(&tree_heights, config.n_queries);

        // Check the witness.
        witness.validate_structure(config, &all_fold_steps);
    }
}

impl<Value: IValue> Guess<Value> for FriProof<Value> {
    type Target = FriProof<Var>;

    fn guess(&self, context: &mut Context<Value>) -> Self::Target {
        Self::Target {
            commit: self.commit.guess(context),
            auth_paths: self.auth_paths.guess(context),
            witness: self.witness.guess(context),
        }
    }
}

pub fn empty_fri_proof(config: &FriConfig) -> FriProof<NoValue> {
    let all_fold_steps = compute_all_fold_steps(
        config.log_trace_size - config.log_n_last_layer_coefs,
        config.fold_step,
    );
    let mut log_layer_size = config.log_evaluation_domain_size();
    let mut auth_paths = vec![];

    for step in &all_fold_steps {
        auth_paths.push(vec![
            // The verifier computes the Merkle node at height `log_layer_size - step`
            // from the witness.
            AuthPath(vec![HashValue::no_value(); log_layer_size - step]);
            config.n_queries
        ]);
        log_layer_size -= step;
    }
    let auth_paths = AuthPaths { data: auth_paths };

    let witness_per_query_per_tree = all_fold_steps
        .iter()
        .map(|step| vec![vec![NoValue; 1 << step]; config.n_queries])
        .collect();
    FriProof {
        commit: FriCommitProof {
            layer_commitments: vec![
                HashValue([U32Wrapper::new_unsafe(NoValue); 8]);
                all_fold_steps.len()
            ],
            last_layer_coefs: vec![NoValue; 1 << config.log_n_last_layer_coefs],
        },
        auth_paths,
        witness: FriWitness(witness_per_query_per_tree),
    }
}

/// Computes all the FRI folding steps.
///
/// # Arguments
///
/// - `degree_log_ratio`: (log degree of committed polynomial) - (log degree of FRI's last layer).
/// - `fold_step`: the folding step of all the FRI folds except possibly the last.
pub fn compute_all_fold_steps(degree_log_ratio: usize, fold_step: usize) -> Vec<usize> {
    let n_folds = degree_log_ratio.div_ceil(fold_step);
    let rem = degree_log_ratio % fold_step;
    let mut all_fold_steps = vec![fold_step; n_folds];
    if rem != 0 {
        *all_fold_steps.last_mut().unwrap() = rem;
    }
    all_fold_steps
}
