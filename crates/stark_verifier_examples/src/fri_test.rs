use circuits::context::TraceContext;
use circuits::ops::Guess;
use circuits::simd::Simd;
use circuits::test_utils::simd_from_u32s;
use circuits_stark_verifier::fri::fri_decommit;
use circuits_stark_verifier::fri_proof::{
    FriCommitProof, FriProof, FriWitness, compute_all_fold_steps,
};
use circuits_stark_verifier::merkle::{AuthPath, AuthPaths};
use circuits_stark_verifier::proof::ProofConfig;
use circuits_stark_verifier::select_queries::select_queries;
use itertools::{chain, zip_eq};
use num_traits::One;
use rstest::rstest;
use stwo::core::channel::{Blake2sM31Channel, Channel, MerkleChannel};
use stwo::core::circle::Coset;
use stwo::core::fields::m31::BaseField;
use stwo::core::fields::qm31::{QM31, SecureField};
use stwo::core::fri::{ExtendedFriProof, FriConfig};
use stwo::core::poly::circle::CircleDomain;
use stwo::core::queries::Queries;
use stwo::core::vcs_lifted::blake2_merkle::{Blake2sM31MerkleChannel, Blake2sMerkleHasher};
use stwo::core::vcs_lifted::verifier::LOG_PACKED_LEAF_SIZE;
use stwo::prover::backend::CpuBackend;
use stwo::prover::backend::cpu::CpuCirclePoly;
use stwo::prover::fri::FriProver;
use stwo::prover::poly::BitReversedOrder;
use stwo::prover::poly::circle::{PolyOps, SecureEvaluation};

#[rstest]
#[case::fold_step_1(8, 2, 1, 7)]
#[case::fold_step_2(8, 2, 2, 7)]
#[case::fold_step_3(8, 2, 3, 7)]
#[case::fold_step_4(8, 2, 4, 7)]
#[case::fold_step_4_blowup_3(8, 3, 4, 7)]
fn test_fri_decommit_with_jumps(
    #[case] log_trace_size: u32,
    #[case] log_blowup_factor: u32,
    #[case] fold_step: usize,
    #[case] n_queries: usize,
) {
    let log_evaluation_domain_size = log_trace_size + log_blowup_factor;
    let query_indices = generate_query_indices(n_queries, log_evaluation_domain_size);
    let fri_proof =
        create_fri_proof(log_trace_size, log_blowup_factor, fold_step, n_queries, &query_indices);

    let mut context = TraceContext::default();
    // Make a dummy config.
    let config = ProofConfig {
        n_pow_bits: 0,
        n_interaction_pow_bits: 0,
        n_preprocessed_columns: 0,
        n_trace_columns: 0,
        n_interaction_columns: 0,
        component_shapes: vec![],
        cumulative_sum_columns: vec![],
        fri: circuits_stark_verifier::fri_proof::FriConfig {
            log_trace_size: log_trace_size as usize,
            log_blowup_factor: log_blowup_factor as usize,
            n_queries,
            log_n_last_layer_coefs: 0,
            fold_step,
        },
    };

    // Compute FRI input.
    let input = simd_from_u32s(&mut context, query_indices.iter().map(|x| *x as u32).collect());
    let queries = select_queries(&mut context, &input, log_evaluation_domain_size as usize);
    let fri_input: Vec<_> = query_indices
        .iter()
        .map(|query| context.constant(fri_proof.aux.first_layer.all_values[0][query]))
        .collect();
    let bits: Vec<Vec<_>> =
        queries.bits.iter().map(|simd| Simd::unpack(&mut context, simd)).collect();

    // Compute the circuit FriProof.
    let all_fold_steps = compute_all_fold_steps(config.log_trace_size(), fold_step);
    let witness = test_construct_fri_witness(&fri_proof, &all_fold_steps, &query_indices);
    let auth_paths =
        test_construct_fri_auth_paths(&fri_proof, &config, &query_indices, &all_fold_steps);
    let last_layer_coefficients = fri_proof.proof.last_layer_poly.into_ordered_coefficients();
    let circuit_fri_proof = FriProof {
        commit: FriCommitProof {
            layer_commitments: chain!(
                [fri_proof.proof.first_layer.commitment.into()],
                fri_proof.proof.inner_layers.iter().map(|layer| layer.commitment.into()),
            )
            .collect(),
            last_layer_coefs: last_layer_coefficients.clone(),
        },
        auth_paths,
        witness,
    };
    let circuit_fri_proof = circuit_fri_proof.guess(&mut context);

    // Compute the folding alphas.
    let mut channel = Blake2sM31Channel::default();
    let proof_layer_commitments = chain!(
        [fri_proof.proof.first_layer.commitment],
        fri_proof.proof.inner_layers.iter().map(|layer| layer.commitment),
    );
    let alpha_values: Vec<_> = proof_layer_commitments
        .map(|commitment| {
            Blake2sM31MerkleChannel::mix_root(&mut channel, commitment);
            channel.draw_secure_felt()
        })
        .collect();
    channel.mix_felts(&last_layer_coefficients);
    let alphas: Vec<_> = alpha_values.iter().map(|x| context.constant(*x)).collect();

    fri_decommit(&mut context, &circuit_fri_proof, &config.fri, fri_input, &bits, queries, &alphas);
    context.validate_circuit();
    println!("Stats: {:?}", context.stats);
}
/// Returns an evaluation of a random polynomial with degree `2^log_degree`.
///
/// The evaluation domain size is `2^(log_degree + log_blowup_factor)`.
/// Copied from [`stwo::core::fri::tests::polynomial_evaluation`].
fn polynomial_evaluation(
    log_degree: u32,
    log_blowup_factor: u32,
) -> SecureEvaluation<CpuBackend, BitReversedOrder> {
    let poly = CpuCirclePoly::new(vec![BaseField::one(); 1 << log_degree]);
    let coset = Coset::half_odds(log_degree + log_blowup_factor - 1);
    let domain = CircleDomain::new(coset);
    let values = poly.evaluate(domain);
    SecureEvaluation::new(domain, values.into_iter().map(SecureField::from).collect())
}

fn create_fri_proof(
    log_trace_size: u32,
    log_blowup_factor: u32,
    fold_step: usize,
    n_queries: usize,
    query_indices: &[usize],
) -> ExtendedFriProof<Blake2sMerkleHasher> {
    let config = FriConfig::new(0, log_blowup_factor, n_queries, fold_step as u32);
    let column = polynomial_evaluation(log_trace_size, log_blowup_factor);
    let twiddles = CpuBackend::precompute_twiddles(column.domain.half_coset);
    let prover = FriProver::<CpuBackend, Blake2sM31MerkleChannel>::commit(
        &mut Blake2sM31Channel::default(),
        config,
        &column,
        &twiddles,
    );
    let queries = Queries::new(query_indices, log_trace_size + log_blowup_factor);
    prover.decommit_on_queries(&queries)
}

fn generate_query_indices(n_queries: usize, log_evaluation_domain_size: u32) -> Vec<usize> {
    let domain_size = 1 << log_evaluation_domain_size;
    let step = domain_size / (n_queries + 1);
    (1..=n_queries).map(|i| i * step).collect()
}

/// Constructs the witnesses for the FRI decommitment phase with the values from the given proof
/// ([ExtendedFriProof]).
fn test_construct_fri_witness(
    proof: &ExtendedFriProof<Blake2sMerkleHasher>,
    all_fold_steps: &[usize],
    query_locations: &[usize],
) -> FriWitness<QM31> {
    let all_layers = [
        &[&proof.aux.first_layer][..],
        proof.aux.inner_layers.iter().collect::<Vec<_>>().as_slice(),
    ]
    .concat();
    let mut witness_per_query_per_tree = vec![vec![]; all_fold_steps.len()];
    for query in query_locations {
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

/// Constructs [AuthPaths] for the FRI trees with the values from the given proof
/// ([ExtendedFriProof]).
fn test_construct_fri_auth_paths(
    proof: &ExtendedFriProof<Blake2sMerkleHasher>,
    config: &ProofConfig,
    query_locations: &[usize],
    all_fold_steps: &[usize],
) -> AuthPaths<QM31> {
    let unsorted_query_locations = &query_locations;
    let layers = chain!([&proof.aux.first_layer], &proof.aux.inner_layers);
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
