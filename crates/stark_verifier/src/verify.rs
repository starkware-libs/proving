use std::collections::HashMap;

use circuits::context::{Context, Var};
use circuits::eval;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::IValue;
use circuits::ops::eq;
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;
use itertools::{Itertools, chain, zip_eq};
use stwo::core::fields::m31::{M31, P};
use stwo::core::verifier::COMPOSITION_LOG_SPLIT;
use stwo_constraint_framework::{INTERACTION_TRACE_IDX, ORIGINAL_TRACE_IDX};

use crate::channel::Channel;
use crate::constraint_eval::compute_composition_polynomial;
use crate::fri::{fri_commit, fri_decommit};
use crate::merkle::decommit_eval_domain_samples;
use crate::oods::{collect_oods_responses, compute_fri_input, extract_expected_composition_eval};
use crate::proof::{Proof, ProofConfig};
use crate::select_queries::{get_query_selection_input_from_channel, select_queries};
use crate::statement::{EvaluateArgs, OodsSamples, Statement};

/// Number of bits used to represent a component log-size in the packed claim format.
/// Must satisfy `2^LOG_SIZE_BITS >= max(log_trace_size) + 1`.
pub const LOG_SIZE_BITS: u32 = 5;

/// The bit-shift applied to component row counts when checking that no relation is used
/// more than `P` times (see `check_relation_uses`).
///
/// Dividing by `DIV = 2^RELATION_USES_NUM_ROWS_SHIFT` keeps intermediate products small
/// enough to fit in a `u64`: at most `n_uses_per_row * (2^(max_log_trace - SHIFT) + 1)` per
/// component, summed over all components per relation.
pub const RELATION_USES_NUM_ROWS_SHIFT: usize = 16;

pub fn validate_logup_sum(
    context: &mut Context<impl IValue>,
    public_logup_sum: Var,
    claimed_sums: &[Var],
) {
    let mut logup_sum = public_logup_sum;
    for claimed_sum in claimed_sums {
        logup_sum = eval!(context, (logup_sum) + (*claimed_sum));
    }
    context.debug_info.insert("logup_sum".into(), logup_sum);
    eq(context, logup_sum, context.zero());
}

pub fn verify<Value: IValue>(
    context: &mut Context<Value>,
    proof: &Proof<Var>,
    config: &ProofConfig,
    statement: &impl Statement<Value>,
) {
    proof.validate_structure(config);
    // The largest canonical coset and hence evaluation domain size is 2^30.
    assert!(config.log_evaluation_domain_size() <= 30);
    let mut channel = Channel::new(context);

    // Mix the channel salt.
    channel.mix_qm31s(context, [proof.channel_salt]);

    // Mix the pcs config.
    config.mix_pcs_config(context, &mut channel);

    // Mix the preprocessed root (known from the statement) into the channel.
    let preprocessed_root = statement.get_preprocessed_root(context);
    channel.mix_commitment(context, &preprocessed_root);

    let component_log_sizes = statement.get_component_log_sizes().clone();
    let component_sizes = validate_and_compute_component_sizes(
        context,
        &component_log_sizes,
        config.log_trace_size(),
    );

    // Check that the component sizes are at most 2^config.log_trace_size().
    // Note that we need k + 1 bits to represent 2^k.
    let component_sizes_bits =
        extract_bits(context, &component_sizes, config.log_trace_size() as u32 + 1);

    for claim_to_mix in statement.claims_to_mix(context) {
        channel.mix_u32s(context, claim_to_mix.into_iter());
    }

    // Mix the trace commitments into the channel.
    channel.mix_commitment(context, &proof.trace_root);

    channel.pow(context, config.n_interaction_pow_bits, proof.interaction_pow_nonce);
    // Pick the interaction elements.
    let [interaction_z, interaction_alpha] = channel.draw_two_qm31s(context);
    context.debug_info.insert("interaction_z".into(), interaction_z);
    context.debug_info.insert("interaction_alpha".into(), interaction_alpha);

    let public_logup_sum = statement.public_logup_sum(context, [interaction_z, interaction_alpha]);
    validate_logup_sum(context, public_logup_sum, &proof.claimed_sums);

    channel.mix_qm31s(context, proof.claimed_sums.iter().cloned());
    channel.mix_commitment(context, &proof.interaction_root);

    // Draw a random QM31 coefficient for the composition polynomial.
    let composition_polynomial_coeff = channel.draw_qm31(context);
    context.debug_info.insert("composition_polynomial_coeff".into(), composition_polynomial_coeff);

    channel.mix_commitment(context, &proof.composition_polynomial_root);

    // Draw a random point for the OODS.
    let oods_point = channel.draw_point(context);

    let shifted_relation_uses = check_relation_uses(context, statement, &component_sizes_bits);
    let unpacked_component_sizes = Simd::unpack(context, &component_sizes);
    statement.verify_claim(context, &unpacked_component_sizes, &shifted_relation_uses);

    // Mix the values at the OODS point (and its previous point where applicable) into the channel.
    let interaction_at_oods = proof
        .interaction_at_oods
        .iter()
        .flat_map(|interaction| {
            if let Some(interaction_at_prev) = interaction.at_prev {
                vec![interaction_at_prev, interaction.at_oods]
            } else {
                vec![interaction.at_oods]
            }
        })
        .collect_vec();
    channel.mix_qm31s(
        context,
        chain!(
            proof.preprocessed_columns_at_oods.iter().cloned(),
            proof.trace_at_oods.iter().cloned(),
            interaction_at_oods,
            proof.composition_eval_at_oods,
        ),
    );

    // Compute the composition evaluation at the OODS point from `proof.*_at_oods` and compare
    // to `proof.composition_eval_at_oods`.
    let composition_eval = compute_composition_polynomial(
        context,
        config,
        statement,
        EvaluateArgs {
            oods_samples: OodsSamples {
                preprocessed_columns: &proof.preprocessed_columns_at_oods,
                trace: &proof.trace_at_oods,
                interaction: &proof.interaction_at_oods,
            },
            pt: oods_point,
            log_domain_size: config.log_trace_size(),
            composition_polynomial_coeff,
            interaction_elements: [interaction_z, interaction_alpha],
            claimed_sums: &proof.claimed_sums,
            component_sizes: &unpacked_component_sizes,
            n_instances_bits: &component_sizes_bits,
        },
    );
    context.debug_info.insert("composition_eval".into(), composition_eval);
    let expected_composition_eval = extract_expected_composition_eval(
        context,
        &proof.composition_eval_at_oods,
        oods_point,
        config.log_trace_size() + COMPOSITION_LOG_SPLIT as usize,
    );
    eq(context, composition_eval, expected_composition_eval);

    // Draw a random challenge for the linear combination of the OODS quotients.
    let oods_quotient_coef = channel.draw_qm31(context);

    // Run the commit phase of FRI.
    let fri_alphas = fri_commit(context, &mut channel, &proof.fri.commit);

    // Proof of work before query selection.
    channel.pow(context, config.n_pow_bits, proof.pow_nonce);

    // Select queries.
    let query_selection_input =
        get_query_selection_input_from_channel(context, &mut channel, config.n_queries());
    let queries =
        select_queries(context, &query_selection_input, config.log_evaluation_domain_size());

    // Check decommitment of trace queries.
    let bits = queries.bits.iter().map(|simd| Simd::unpack(context, simd)).collect_vec();

    let opt_column_log_sizes_by_trace = get_opt_column_log_sizes_by_trace(
        context,
        config,
        component_log_sizes,
        statement.sorting_required(),
    );

    decommit_eval_domain_samples(
        context,
        config.n_queries(),
        &opt_column_log_sizes_by_trace,
        &proof.eval_domain_samples,
        &proof.eval_domain_auth_paths,
        &bits,
        &{
            let [trace, interaction, composition] = proof.merkle_roots();
            [preprocessed_root, trace, interaction, composition]
        },
    );

    // Compute FRI input.
    let oods_responses =
        collect_oods_responses(context, config, oods_point, &component_sizes_bits, proof);
    let fri_input = compute_fri_input(
        context,
        &oods_responses,
        &queries,
        &proof.eval_domain_samples,
        oods_quotient_coef,
    );

    fri_decommit(context, &proof.fri, &config.fri, fri_input, &bits, queries, &fri_alphas);
}

/// Verify that no relation is used more than P times.
/// For each relation, it verifies that sum(uses_per_row * num_rows) < P
/// where the sum is over all the components that use the relation.
///
/// To avoid overflows when computing the sum, we check
/// sum(uses_per_row * (floor(num_rows / DIV) + 1)) <= floor(P / DIV)
/// where DIV = 2 ** RELATION_USES_NUM_ROWS_SHIFT
fn check_relation_uses<Value: IValue>(
    context: &mut Context<impl IValue>,
    statement: &impl Statement<Value>,
    component_sizes_bits: &[Simd],
) -> HashMap<String, Var> {
    let components = statement.get_components();

    // Check that sum(uses_per_row * (floor(num_rows / DIV) + 1)) cannot overflow even for the
    // maximal num_rows (num_rows = P).
    // This is a sanity check that `RELATION_USES_NUM_ROWS_SHIFT` is large enough for the given
    // statement, it does not depend on the specific assignment.
    let mut max_shifted_uses_per_relation = HashMap::<&str, u64>::new();
    for component in components.values() {
        for relation_use in component.relation_uses_per_row() {
            let entry = max_shifted_uses_per_relation.entry(relation_use.relation_id).or_insert(0);
            *entry = entry
                .checked_add(relation_use.uses * (((P >> RELATION_USES_NUM_ROWS_SHIFT) + 1) as u64))
                .expect("Shifted num rows upper bound computation overflowed");
        }
    }
    assert!(max_shifted_uses_per_relation.values().all(|count| *count < (P as u64)));

    // Compute floor(num_rows / DIV) + 1 for all components
    let shifted_component_sizes_p1 = match component_sizes_bits.get(RELATION_USES_NUM_ROWS_SHIFT..)
    {
        Some(high_bits) => {
            let one = Simd::one(context, components.len());
            let shifted_component_sizes = Simd::combine_bits(context, high_bits);
            let res = Simd::add(context, &shifted_component_sizes, &one);
            // A variable in the Simd vector might be unused in the case where all the corresponding
            // components don't use any relations.
            Simd::mark_partly_used(context, &res);
            res
        }
        None => Simd::one(context, components.len()),
    };

    // Sum uses_per_row * (floor(num_rows / DIV) + 1) for all relations
    let mut shifted_relation_uses = HashMap::new();
    for (i, component) in components.values().enumerate() {
        let relation_uses = component.relation_uses_per_row();
        if relation_uses.is_empty() {
            continue;
        }
        let shifted_size_p1 = Simd::unpack_idx(context, &shifted_component_sizes_p1, i);
        for relation_use in relation_uses {
            let uses_per_row = context.constant(u32::try_from(relation_use.uses).unwrap().into());

            let shifted_uses_upper_bound = eval!(context, (shifted_size_p1) * (uses_per_row));

            shifted_relation_uses
                .entry(relation_use.relation_id.to_string())
                .and_modify(|entry| {
                    *entry = eval!(context, (*entry) + (shifted_uses_upper_bound));
                })
                .or_insert(shifted_uses_upper_bound);
        }
    }

    // Verify that the sum is at most `floor(P / DIV) = 2^(31 - RELATION_USES_NUM_ROWS_SHIFT) - 1`
    // by expressing it as a `31 - RELATION_USES_NUM_ROWS_SHIFT`-bit number.
    let shifted_use_counts = shifted_relation_uses
        .iter()
        .sorted_by_key(|(k, _v)| *k)
        .map(|(_k, v)| M31Wrapper::new_unsafe(*v))
        .collect_vec();
    let shifted_use_counts = Simd::pack(context, &shifted_use_counts);
    // The range check below ensures that:
    // `shifted_use_counts <= 2^(31 - RELATION_USES_NUM_ROWS_SHIFT) - 1`
    // therefore:
    // `use_counts < shifted_use_counts * 2^RELATION_USES_NUM_ROWS_SHIFT <=
    // <= 2^31 - 2^RELATION_USES_NUM_ROWS_SHIFT < P`
    // where `use_counts` is the total number of uses, per relation.
    extract_bits(context, &shifted_use_counts, 31 - RELATION_USES_NUM_ROWS_SHIFT as u32);
    shifted_relation_uses
}

// Returns a map from trace index to the column log sizes used to sort that trace's query columns
// into committed order. A trace index absent from the map needs no sorting (its columns are already
// in committed order), so only the trace and interaction columns ever appear. When
// `sorting_required` is false every trace is already in committed order and an empty map is
// returned, skipping sorting entirely.
fn get_opt_column_log_sizes_by_trace(
    context: &mut Context<impl IValue>,
    config: &ProofConfig,
    component_log_sizes: Simd,
    sorting_required: bool,
) -> HashMap<usize, Vec<Var>> {
    if !sorting_required {
        return HashMap::new();
    }

    let mut column_log_sizes = [
        Vec::with_capacity(config.n_trace_columns),
        Vec::with_capacity(config.n_interaction_columns),
    ];

    for (component_shape, log_size) in
        zip_eq(&config.component_shapes, Simd::unpack(context, &component_log_sizes))
    {
        column_log_sizes[0].extend(vec![log_size; component_shape.trace_columns]);
        column_log_sizes[1].extend(vec![log_size; component_shape.interaction_columns]);
    }
    let [trace, interaction] = column_log_sizes;
    HashMap::from([(ORIGINAL_TRACE_IDX, trace), (INTERACTION_TRACE_IDX, interaction)])
}

fn validate_and_compute_component_sizes(
    context: &mut Context<impl IValue>,
    component_log_sizes: &Simd,
    log_trace_size: usize,
) -> Simd {
    const _: () = assert!(
        LOG_SIZE_BITS == 5,
        "5 bits suffice to represent any log trace size up to the maximum of 30."
    );

    // Extract the bits of the component log sizes and verify that it is in the range [0,
    // 2**log_trace_size). Note that log_trace_size is at most 30, so 2**log_trace_size does not
    // overflow in M31.
    let component_log_size_bits = extract_bits(context, component_log_sizes, LOG_SIZE_BITS);
    let log_trace_size =
        Simd::repeat(context, M31::from(log_trace_size), component_log_sizes.len());
    let diff = Simd::sub(context, &log_trace_size, component_log_sizes);
    extract_bits(context, &diff, LOG_SIZE_BITS);

    Simd::pow2(context, &component_log_size_bits)
}
