use std::array;

use circuits::ivalue::qm31_from_u32s;
use hashbrown::HashMap;
use itertools::{Itertools, chain, zip_eq};
use num_traits::Zero;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;
use stwo::core::proof::ExtendedStarkProof;
use stwo::core::vcs_lifted::blake2_merkle::Blake2sMerkleHasher;
use stwo::core::vcs_lifted::verifier::LOG_PACKED_LEAF_SIZE;

use crate::fri_proof::{FriCommitProof, FriProof, FriWitness, compute_all_fold_steps};
use crate::merkle::{AuthPath, AuthPaths};
use crate::oods::EvalDomainSamples;
use crate::proof::{InteractionAtOods, N_TRACES, Proof, ProofConfig};

/// Constructs [Proof] with the values from the given proof ([ExtendedStarkProof]).
pub fn proof_from_stark_proof(
    proof: &ExtendedStarkProof<Blake2sMerkleHasher>,
    config: &ProofConfig,
    claimed_sums: Vec<QM31>,
    interaction_pow_nonce: u64,
    channel_salt: u32,
) -> Proof<QM31> {
    let commitments = &proof.proof.commitments;
    let sampled_values = &proof.proof.sampled_values;
    let fri_proof = &proof.proof.fri_proof;

    let pow: u64 = proof.proof.proof_of_work;
    let pow_high = (pow >> 32) as u32;
    let pow_low = (pow & 0xFFFFFFFF) as u32;

    let interaction_pow_high = (interaction_pow_nonce >> 32) as u32;
    let interaction_pow_low = (interaction_pow_nonce & 0xFFFFFFFF) as u32;

    let all_fold_steps = compute_all_fold_steps(
        config.fri.log_trace_size - config.fri.log_n_last_layer_coefs,
        config.fri.fold_step,
    );

    Proof {
        trace_root: commitments[1].into(),
        interaction_root: commitments[2].into(),
        composition_polynomial_root: commitments[3].into(),
        preprocessed_columns_at_oods: as_single_row(&sampled_values[0]),
        trace_at_oods: as_single_row(&sampled_values[1]),
        interaction_at_oods: sampled_values[2]
            .iter()
            .map(|x| match x[..] {
                [at_prev, at_oods] => InteractionAtOods { at_oods, at_prev: Some(at_prev) },
                [at_oods] => InteractionAtOods { at_oods, at_prev: None },
                _ => panic!("Unexpected interaction at OODS values"),
            })
            .collect_vec(),
        claimed_sums,
        composition_eval_at_oods: as_single_row(&sampled_values[3]).try_into().unwrap(),
        eval_domain_samples: construct_eval_domain_samples(proof, config),
        eval_domain_auth_paths: construct_eval_domain_auth_paths(proof, config),
        fri: FriProof {
            commit: FriCommitProof {
                layer_commitments: chain!(
                    [fri_proof.first_layer.commitment.into()],
                    fri_proof.inner_layers.iter().map(|layer| layer.commitment.into()),
                )
                .collect(),
                last_layer_coefs: (*fri_proof.last_layer_poly).to_vec(),
            },
            auth_paths: construct_fri_auth_paths(proof, config, &all_fold_steps),
            witness: construct_fri_witness(proof, &all_fold_steps),
        },
        pow_nonce: qm31_from_u32s(pow_low, pow_high, 0, 0),
        interaction_pow_nonce: qm31_from_u32s(interaction_pow_low, interaction_pow_high, 0, 0),
        channel_salt: qm31_from_u32s(channel_salt, 0, 0, 0),
    }
}

/// Converts a 2D vector of singletons to a 1D vector.
fn as_single_row(values: &[Vec<QM31>]) -> Vec<QM31> {
    values
        .iter()
        .map(|x| {
            let [x] = x[..].try_into().unwrap();
            x
        })
        .collect_vec()
}

/// Constructs [EvalDomainSamples] with the values from the given proof ([ExtendedStarkProof]).
fn construct_eval_domain_samples(
    proof: &ExtendedStarkProof<Blake2sMerkleHasher>,
    config: &ProofConfig,
) -> EvalDomainSamples<QM31> {
    let unsorted_query_locations = &proof.aux.unsorted_query_locations;
    let queried_values = &proof.proof.queried_values;

    let n_queries = config.n_queries();

    // Map from query position to its one or more indices in
    // `extra_proof_data.unsorted_query_locations`.
    // For example, if `unsorted_query_locations = [10, 5, 10]` then
    //   `query_to_indices = { 10: [0, 2], 5: [1] }`.
    let mut query_to_indices = HashMap::<usize, Vec<usize>>::new();
    assert_eq!(unsorted_query_locations.len(), n_queries);
    for (i, query) in unsorted_query_locations.iter().enumerate() {
        query_to_indices.entry(*query).or_insert(Vec::new()).push(i);
    }

    let sorted_query_indices = query_to_indices.keys().sorted().collect_vec();

    // For each trace and column, add the sampled values from all the queries in unsorted order.
    let mut data: Vec<Vec<Vec<M31>>> = vec![vec![]; N_TRACES];
    for (trace_idx, n_columns_in_trace) in config.n_columns_per_trace().iter().enumerate() {
        for column_idx in 0..*n_columns_in_trace {
            data[trace_idx].push(vec![M31::zero(); n_queries]);
            for (query_idx, query) in sorted_query_indices.iter().enumerate() {
                for idx in &query_to_indices[*query] {
                    data[trace_idx][column_idx][*idx] =
                        queried_values[trace_idx][column_idx][query_idx];
                }
            }
        }
    }

    EvalDomainSamples::from_m31s(data)
}

/// Constructs [AuthPaths] for the evaluation domain queries (the in-domain queries) with the values
/// from the given proof ([ExtendedStarkProof]).
fn construct_eval_domain_auth_paths(
    proof: &ExtendedStarkProof<Blake2sMerkleHasher>,
    config: &ProofConfig,
) -> AuthPaths<QM31> {
    let unsorted_query_locations = &proof.aux.unsorted_query_locations;
    let res = proof
        .aux
        .trace_decommitment
        .iter()
        .map(|merkle_decommitment_aux| {
            unsorted_query_locations
                .iter()
                .map(|query_idx| {
                    let mut auth_path: AuthPath<QM31> = AuthPath(vec![]);
                    let mut pos = *query_idx;
                    for j in 0..config.log_evaluation_domain_size() {
                        let hash = merkle_decommitment_aux.all_node_values[j][&(pos ^ 1)];
                        auth_path.0.push(hash.into());
                        pos >>= 1;
                    }
                    auth_path
                })
                .collect()
        })
        .collect();

    AuthPaths { data: res }
}

/// Constructs [AuthPaths] for the FRI trees with the values from the given proof
/// ([ExtendedStarkProof]).
fn construct_fri_auth_paths(
    proof: &ExtendedStarkProof<Blake2sMerkleHasher>,
    config: &ProofConfig,
    all_fold_steps: &[usize],
) -> AuthPaths<QM31> {
    let unsorted_query_locations = &proof.aux.unsorted_query_locations;
    let layers = chain!([&proof.aux.fri.first_layer], &proof.aux.fri.inner_layers);
    let mut log_layer_size = config.log_evaluation_domain_size();
    let mut fold_sum = 0;
    let mut res = vec![];

    for (layer_proof, step) in zip_eq(layers, all_fold_steps) {
        res.push(
            unsorted_query_locations
                .iter()
                .map(|query| {
                    let mut pos = *query;
                    let pack_leaves = log_layer_size >= LOG_PACKED_LEAF_SIZE as usize && *step > 1;
                    let pack_shift = if pack_leaves { LOG_PACKED_LEAF_SIZE as usize } else { 0 };
                    pos >>= fold_sum + step;
                    let mut auth_path: AuthPath<QM31> = AuthPath(vec![]);
                    for j in *step..log_layer_size {
                        let hash =
                            layer_proof.decommitment.all_node_values[j - pack_shift][&(pos ^ 1)];
                        auth_path.0.push(hash.into());
                        pos >>= 1;
                    }
                    auth_path
                })
                .collect(),
        );
        log_layer_size -= step;
        fold_sum += step;
    }

    AuthPaths { data: res }
}

/// Constructs the witnesses for the FRI decommitment phase with the values from the given proof
/// ([ExtendedStarkProof]).
///
/// Returns a pair where:
/// - the first member contains the fri siblings of the first FRI layer. Currently the
///   circle-to-line fold is hardcoded to 1, so there is exactly one sibling per query.
/// - the second member contains, for each inner layer, for each query, the coset witness for that
///   query.
pub fn construct_fri_witness(
    proof: &ExtendedStarkProof<Blake2sMerkleHasher>,
    all_fold_steps: &[usize],
) -> FriWitness<QM31> {
    let all_layers: Vec<_> = chain![
        std::slice::from_ref(&proof.aux.fri.first_layer),
        proof.aux.fri.inner_layers.iter(),
    ]
    .collect();
    let mut witness_per_query_per_tree = vec![vec![]; all_fold_steps.len()];
    for query in &proof.aux.unsorted_query_locations {
        let mut pos = *query;
        for (tree_idx, (layer, step)) in zip_eq(&all_layers, all_fold_steps).enumerate() {
            let start = (pos >> step) << step;
            let witness: Vec<_> =
                (start..start + (1 << step)).map(|i| layer.all_values[0][&i]).collect();
            witness_per_query_per_tree[tree_idx].push(witness);
            pos >>= step;
        }
    }
    FriWitness(witness_per_query_per_tree)
}

/// Packs the component log sizes into QM31s.
/// Each QM31 holds up to 4 log sizes and the last one is padded with zeros.
pub fn pack_component_log_sizes(component_log_sizes: &[u32]) -> Vec<QM31> {
    pack_into_qm31s(component_log_sizes.iter().cloned())
}

/// Packs the public claim into QM31s.
/// Each QM31 holds up to 4 public claim values and the last one is padded with zeros.
pub fn pack_public_claim(public_claim: &[M31]) -> Vec<QM31> {
    pack_into_qm31s(public_claim.iter().cloned())
}

pub fn pack_into_qm31s<T: Into<M31>>(values: impl Iterator<Item = T>) -> Vec<QM31> {
    values
        .chunks(4)
        .into_iter()
        .map(|mut chunk| {
            QM31::from_m31_array(array::from_fn(|_| {
                chunk.next().map(|v| v.into()).unwrap_or(M31::zero())
            }))
        })
        .collect_vec()
}
