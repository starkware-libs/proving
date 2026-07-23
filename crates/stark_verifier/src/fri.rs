use circuits::blake::HashValue;
use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::{eq, mul};
use circuits::simd::Simd;
use circuits::utils::select_by_index;
use itertools::{Itertools, zip_eq};
use stwo::core::circle::CirclePoint;
use stwo::core::vcs_lifted::verifier::{LOG_PACKED_LEAF_SIZE, PACKED_LEAF_SIZE};

use crate::channel::Channel;
use crate::circle::{
    add_points_simd, compute_half_coset_points, double_x_simd, minus_generator_point_simd,
    repeated_double_point_simd,
};
use crate::fri_proof::{FriCommitProof, FriConfig, FriProof, FriWitness};
use crate::merkle::{hash_leaf_qm31, hash_node, hash_packed_leaf_qm31s, verify_merkle_path};
use crate::select_queries::Queries;

#[cfg(test)]
#[path = "fri_test.rs"]
pub mod test;

/// Commits to the FRI layers and returns the random alphas.
pub fn fri_commit(
    context: &mut Context<impl IValue>,
    channel: &mut Channel,
    proof: &FriCommitProof<Var>,
) -> Vec<Var> {
    let mut alphas = Vec::new();
    for root in &proof.layer_commitments {
        channel.mix_commitment(context, root);
        alphas.push(channel.draw_qm31(context));
    }
    channel.mix_qm31s(context, proof.last_layer_coefs.iter().cloned());

    alphas
}

/// Validates that the values in `fri_input` are consistent with the FRI commitment.
pub fn fri_decommit<Value: IValue>(
    context: &mut Context<Value>,
    proof: &FriProof<Var>,
    config: &FriConfig,
    fri_input: Vec<Var>,
    mut bits: &[Vec<Var>],
    queries: Queries,
    alphas: &[Var],
) {
    let FriProof {
        commit: FriCommitProof { layer_commitments, last_layer_coefs },
        auth_paths,
        witness: FriWitness(witness_per_query_per_tree),
    } = proof;

    let mut layer_values = fri_input;
    let mut base_point = queries.points.clone();
    let mut packed_bits = queries.bits.as_slice();

    let mut log_degree_bound = config.log_trace_size;
    let mut step = config.fold_step;
    assert!(config.log_trace_size >= step);
    assert_eq!(config.log_n_last_layer_coefs, 0);

    // Translate base_point to the base of the current circle domain.
    let mut packed_lowest_bits = packed_bits.split_off(..step).unwrap();
    base_point = circle_translate_to_base_point(context, base_point, packed_lowest_bits);
    // Compute twiddles.
    let mut twiddles_per_fold = circle_compute_twiddles_from_base_point(context, &base_point, step);
    // Number of times to double the base point.
    // Since the first fold is circle-to-line, we double the base point step - 1 times.
    let mut n_doubles = step - 1;

    for (tree_idx, (root, witness_per_query)) in
        zip_eq(layer_commitments, witness_per_query_per_tree).enumerate()
    {
        let log_layer_size = bits.len();

        // Validate that the fri query is in the correct position inside the guessed
        // `witness_per_query`.
        validate_query_position_in_coset(
            context,
            witness_per_query,
            &layer_values,
            bits.split_off(..step).unwrap(),
        );

        // Check merkle decommitment.
        for (query_idx, witness) in witness_per_query.iter().enumerate() {
            let pack_leaves = log_layer_size >= LOG_PACKED_LEAF_SIZE as usize && step > 1;
            // Compute the leaves.
            let (mut leaves, n_folds): (Vec<HashValue<Var>>, usize) = if pack_leaves {
                (
                    witness
                        .chunks(PACKED_LEAF_SIZE)
                        .map(|chunk| hash_packed_leaf_qm31s(context, chunk.try_into().unwrap()))
                        .collect(),
                    step - LOG_PACKED_LEAF_SIZE as usize,
                )
            } else {
                (witness.iter().map(|val| hash_leaf_qm31(context, *val)).collect(), step)
            };

            // Compute the merkle root of the witness values. Each fold halves the active prefix
            // of `leaves`, leaving the root at index 0.
            let witness_root = {
                for fold in 0..n_folds {
                    for i in 0..1 << (n_folds - fold - 1) {
                        leaves[i] = hash_node(context, &leaves[2 * i], &leaves[2 * i + 1]);
                    }
                }
                leaves[0].clone()
            };
            // Verify the rest of the authentication path.
            let auth_path = auth_paths.at(tree_idx, query_idx);
            let bits_for_query = bits.iter().map(|b| b[query_idx]).collect_vec();
            verify_merkle_path(context, witness_root, &bits_for_query, root, auth_path);
        }

        // Compute alpha, alpha^2, ..., alpha^(2^(step - 1));
        let mut alpha_powers = Vec::with_capacity(step);
        let mut alpha_pow = alphas[tree_idx];
        alpha_powers.push(alpha_pow);
        for _ in 0..step - 1 {
            alpha_pow = mul(context, alpha_pow, alpha_pow);
            alpha_powers.push(alpha_pow);
        }

        // Unpack twiddles from [fold][twiddle_packed] to per-query [fold][twiddle].
        let twiddles_per_query: Vec<Vec<Vec<Var>>> = (0..config.n_queries)
            .map(|q| {
                twiddles_per_fold
                    .iter()
                    .map(|twiddles_at_fold| {
                        twiddles_at_fold
                            .iter()
                            .map(|simd| Simd::unpack_idx(context, simd, q))
                            .collect()
                    })
                    .collect()
            })
            .collect();

        // Compute the next layer.
        layer_values = zip_eq(witness_per_query, twiddles_per_query)
            .map(|(witness, twiddles_per_fold)| {
                fold_coset(context, witness, &twiddles_per_fold, &alpha_powers)
            })
            .collect();

        log_degree_bound = log_degree_bound.saturating_sub(step);
        if log_degree_bound == 0 {
            break;
        }

        // Double the base point to get the query domain point corresponding to `layer_values` for
        // the next step. This is done after the early exit check because the query domain
        // points are not needed after we exit the loop.
        let query_domain_point = repeated_double_point_simd(context, &base_point, n_doubles);

        // Update the number of times to double the base point for the next step.
        n_doubles = step;
        step = std::cmp::min(step, log_degree_bound);

        packed_lowest_bits = packed_bits.split_off(..step).unwrap();

        // Translate query_domain_point to the base of the current coset.
        base_point = translate_to_base_point(context, query_domain_point, packed_lowest_bits);

        // Compute twiddles for the next step.
        twiddles_per_fold = compute_twiddles_from_base_point(context, &base_point, step);
    }
    // The last fri layer log size is equal to the log blowup factor.
    assert_eq!(bits.len(), config.log_blowup_factor);
    assert_eq!(packed_bits.len(), config.log_blowup_factor);

    // The last base point's y-coords hasn't been used by `compute_twiddles_from_base_point` if the
    // last step was = 1.
    if step == 1 {
        Simd::mark_partly_used(context, &base_point.y);
    }

    // Check last layer.
    assert_eq!(config.log_n_last_layer_coefs, 0);
    let last_layer_val = last_layer_coefs[0];
    for value in layer_values {
        eq(context, value, last_layer_val);
    }
}

/// Folds a coset of log size n to a point using the folding coefficients `alphas`.
/// `twiddles_per_fold[i]` contains the twiddles needed at fold i, and has length 2^(n - 1 - i).
fn fold_coset<Value: IValue>(
    context: &mut Context<Value>,
    coset_values: &[Var],
    twiddles_per_fold: &[Vec<Var>],
    alphas: &[Var],
) -> Var {
    assert_eq!(coset_values.len(), 1 << twiddles_per_fold.len());
    let mut values = coset_values.to_vec();

    for (alpha, twiddles) in zip_eq(alphas, twiddles_per_fold) {
        for (j, t) in twiddles.iter().enumerate() {
            let (even, odd) = (values[2 * j], values[2 * j + 1]);
            let g = eval!(context, (even) + (odd));
            let h = eval!(context, ((even) - (odd)) * (*t));
            values[j] = eval!(context, (g) + ((*alpha) * (h)));
        }
    }
    values[0]
}

/// Verifies that the query value is in the correct position among the guessed coset values.
///
/// # Arguments
///
/// - `context`: the circuit's context.
/// - `witness_per_query`: for each query, the values of the layer's polynomial on the coset (in the
///   case of the circle-to-line fold, it's a circle domain) containing the query point. The domain
///   log size is equal to this layer's fri fold step.
/// - `layer_values`: the query values.
/// - `bits`: for each query, the coset log size-many lowest significant bits of the query position.
fn validate_query_position_in_coset<Value: IValue>(
    context: &mut Context<Value>,
    witness_per_query: &[Vec<Var>],
    layer_values: &[Var],
    bits: &[Vec<Var>],
) {
    for (query_idx, (query_value, coset)) in zip_eq(layer_values, witness_per_query).enumerate() {
        let bits: Vec<Var> = bits.iter().map(|b| b[query_idx]).collect();
        let expected_query_value = select_by_index(context, coset, &bits);
        eq(context, *query_value, expected_query_value);
    }
}

/// Computes the twiddles needed for a circle-to-line FRI fold step.
///
/// Prepends y-coordinate twiddles for the first circle-to-line fold, then delegates to
/// [compute_x_twiddles] for the remaining x-coordinate folds.
fn circle_compute_twiddles_from_base_point<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    fold_step: usize,
) -> Vec<Vec<Simd>> {
    assert!(fold_step > 0);
    let n_queries = base_point.x.len();

    if fold_step == 1 {
        return vec![vec![base_point.y.inv(context)]];
    }

    // For circle-to-line, the witness domain is a circle domain with half-coset of size
    // `fold_step - 1`.
    let coset_points = compute_half_coset_points(context, base_point, (fold_step - 1) as u32);
    if let Some(last_pt) = coset_points.iter().skip(1).last() {
        Simd::mark_partly_used(context, &last_pt.y);
    }

    // The first fold uses y-coordinate twiddles (one per pair of conjugate points).
    let zero = Simd::zero(context, n_queries);
    let y_coords: Vec<Simd> =
        coset_points.iter().flat_map(|p| [p.y.clone(), Simd::sub(context, &zero, &p.y)]).collect();
    let mut twiddles_per_fold: Vec<Vec<Simd>> =
        vec![y_coords.iter().map(|y| y.inv(context)).collect()];

    // The remaining folds use x-coordinate twiddles, reusing the same coset points.
    let x_coords: Vec<Simd> = coset_points.into_iter().map(|p| p.x).collect();
    twiddles_per_fold.extend(compute_x_twiddles(context, x_coords, fold_step - 1));
    twiddles_per_fold
}

/// Computes x-coordinate twiddles from pre-computed coset x-coordinates.
///
/// Performs `num_folds` rounds: each round inverts the current x-coordinates, then squashes them
/// via `double_x` for the next round.
fn compute_x_twiddles<Value: IValue>(
    context: &mut Context<Value>,
    mut x_coords: Vec<Simd>,
    num_folds: usize,
) -> Vec<Vec<Simd>> {
    let mut twiddles_per_fold: Vec<Vec<Simd>> = vec![];
    for fold_idx in 0..num_folds {
        twiddles_per_fold.push(x_coords.iter().map(|x| x.inv(context)).collect());
        // Don't add unused gates in the last iteration.
        if fold_idx != num_folds - 1 {
            x_coords = x_coords.iter().step_by(2).map(|x| double_x_simd(context, x)).collect();
        }
    }
    twiddles_per_fold
}

/// Computes the twiddles needed for a single line-to-line FRI fold step.
///
/// Returns a `Vec<Vec<Simd>>` indexed as `[fold_idx][twiddle_within_fold]`, where each twiddle is
/// SIMD-packed across queries. All twiddles are inverse x-coordinates.
///
/// # Arguments
///
/// - `context`: the circuit's context.
/// - `base_point`: for each query, the first point of the coset of log size `fold_step` that
///   contains the query. More precisely, if the query index has a little-endian bit decomposition
///   a₁a₂a₃a₄...aₙ then its base point is the circle point with index 0...0a_{step + 1}...aₙ. So,
///   for example, for a query with index 101110 and step = 2, its base point has index 001110.
/// - `fold_step`: the folding step for the current FRI fold.
fn compute_twiddles_from_base_point<Value: IValue>(
    context: &mut Context<Value>,
    base_point: &CirclePoint<Simd>,
    fold_step: usize,
) -> Vec<Vec<Simd>> {
    assert!(fold_step > 0);

    let coset_points = compute_half_coset_points(context, base_point, fold_step as u32);
    // The y-coordinate of the last point of half_coset is not necessarily used. In the special case
    // where half_coset consists only of the base point, we don't enter this code path (this special
    // case happens if and only if fold step = 1).
    if let Some(last_pt) = coset_points.iter().skip(1).last() {
        Simd::mark_partly_used(context, &last_pt.y);
    }

    let x_coords: Vec<Simd> = coset_points.into_iter().map(|p| p.x).collect();
    compute_x_twiddles(context, x_coords, fold_step)
}

/// Translates each packed query point to the base point of its local circle domain for the
/// circle-to-line fold.
///
/// Wraps [translate_to_base_point] by first handling the circle-specific bit (negating y).
fn circle_translate_to_base_point<Value: IValue>(
    context: &mut Context<Value>,
    mut base_point: CirclePoint<Simd>,
    packed_bits: &[Simd],
) -> CirclePoint<Simd> {
    let n_queries = base_point.x.len();
    let zero = Simd::zero(context, n_queries);
    let minus_y_coord = Simd::sub(context, &zero, &base_point.y);
    let minus_y_point = CirclePoint { x: base_point.x.clone(), y: minus_y_coord };
    // Select between `point` and `point - g_0` (implemented by negating `y`).
    base_point = CirclePoint {
        x: base_point.x.clone(),
        y: Simd::select(context, &packed_bits[0], &base_point.y, &minus_y_point.y),
    };
    translate_to_base_point(context, base_point, &packed_bits[1..])
}

/// Translates each packed query point to the base point of its local FRI coset.
///
/// For each SIMD lane `j`, when the corresponding bit is `1`, the function subtracts the circle
/// generator corresponding to this bit position; when it is `0`, it leaves the point unchanged.
///
/// Applying this for all provided bits clears the least significant `step`-many bits of each query.
///
/// # Arguments
///
/// - `context`: the circuit context.
/// - `base_point`: packed query points to translate.
/// - `packed_bits`: the least significant `step`-many bits of the current queries (where `step` is
///   the fold_step of the current FRI fold).
fn translate_to_base_point<Value: IValue>(
    context: &mut Context<Value>,
    mut base_point: CirclePoint<Simd>,
    packed_bits: &[Simd],
) -> CirclePoint<Simd> {
    let n_queries = base_point.x.len();

    for (i, bit) in packed_bits.iter().enumerate() {
        // The group inverse of the generator of the subgroup of size 2^(i+1).
        let minus_cur_gen_pt = minus_generator_point_simd(context, i + 1, n_queries);
        // Select between `point` and `point - cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &base_point, &minus_cur_gen_pt);
        base_point = CirclePoint {
            x: Simd::select(context, bit, &base_point.x, &point_if_bit.x),
            y: Simd::select(context, bit, &base_point.y, &point_if_bit.y),
        };
    }
    base_point
}
