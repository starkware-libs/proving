use compiled_casm_air::compiled_structs::{CompiledAirFn, UseOrYield};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use super::utils::{has_enabler_or_mult_column, is_chain, n_logup_columns, QM31_N_TRACE_CELLTS};

pub const LOOKUP_RELATION_BATCH_SIZE: usize = 2;
pub const N_SAMPLES_FOR_PREFIX_SUM: usize = 2;

pub fn gen_lookup_constraints_fn(air_fn: &CompiledAirFn) -> rust::Tokens {
    let lookups = air_fn
        .constraint_lookups
        .iter()
        .enumerate()
        .map(|(i, (relation, _))| format!("{}_sum_{i}: QM31", relation.to_case(Case::Snake)))
        .collect::<Vec<_>>();

    // Revoke the AP tracking after defining the interaction trace vars, to avoid offset overflow.
    quote! {
        $("\n")$("\n")
        fn lookup_constraints(
            ref sum: QM31,
            domain_vanishing_eval_inv: QM31,
            random_coeff: QM31,
            claimed_sum: QM31,
            $(has_enabler_or_mult_column(air_fn).then(||
                    "enabler: QM31,".to_string()
            ).unwrap_or_default())
            column_size: M31,
            ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
            $(lookups.join(",\n"))
        ) {
            $(get_interaction_trace_vars(air_fn))$("\n")
            core::internal::revoke_ap_tracking();$("\n")
            $(gen_lookup_constraints(air_fn))
        }
    }
}

fn gen_lookup_constraints(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    let relations = &air_fn
        .constraint_lookups
        .iter()
        .map(|(name, use_or_yield)| (name.to_case(Case::Snake), *use_or_yield))
        .collect::<Vec<_>>();
    let mut prev_trace = vec![];
    let n_chunks = n_logup_columns(air_fn) / QM31_N_TRACE_CELLTS;
    let last_sum_chunk_has_2_elements =
        air_fn.constraint_lookups.len() % LOOKUP_RELATION_BATCH_SIZE == 0;

    for (i, (trace_chunk, sum_chunk)) in (0..n_logup_columns(air_fn))
        .chunks(QM31_N_TRACE_CELLTS)
        .into_iter()
        .zip(relations.chunks(LOOKUP_RELATION_BATCH_SIZE))
        .enumerate()
    {
        code.append(quote! { $("\n") });
        let trace = trace_chunk
            .into_iter()
            .map(|i| format!("trace_2_col{i}"))
            .collect::<Vec<_>>();
        let prefix = get_lookup_constraints_prefix(i, n_chunks, &trace, &prev_trace);
        let sum_i = i * LOOKUP_RELATION_BATCH_SIZE;
        let (rel1, rel1_sign) = get_sum_name_and_sign(sum_i, &sum_chunk[0]);

        // The following assumes:
        // 1. the last relation sum in the last chunk is a `yield`
        // 2. in case of a chain, the one before last is a `use` of this same component.

        if i < n_chunks - 1 || (i == n_chunks - 1 && last_sum_chunk_has_2_elements) {
            let (rel2, rel2_sign) = get_sum_name_and_sign(sum_i + 1, &sum_chunk[1]);
            let (rel1_mult, rel2_mult) = build_numerator(
                i,
                n_chunks,
                &rel1,
                &rel2,
                last_sum_chunk_has_2_elements,
                is_chain(air_fn),
                has_enabler_or_mult_column(air_fn),
            );

            code.append(quote! {
                let constraint_quotient = (
                    (
                        (
                            $(prefix)
                        ) * $(rel1) * $(rel2)
                    ) $(rel2_sign) $(rel1_mult) $(rel1_sign) $(rel2_mult)
                ) * domain_vanishing_eval_inv;$("\n")
            });
        } else {
            let numerator = if has_enabler_or_mult_column(air_fn) {
                "enabler"
            } else {
                "qm31_const::<1, 0, 0, 0>()"
            };

            code.append(quote! {
                let constraint_quotient = (
                    (
                        (
                            $(prefix)
                        ) * $(rel1)
                    ) $(rel1_sign) $(numerator)
                ) * domain_vanishing_eval_inv;$("\n")
            });
        }

        code.append(quote! {
            sum = sum * random_coeff + constraint_quotient;$("\n")
        });

        prev_trace = trace;
    }

    code
}

fn get_lookup_constraints_prefix(
    i: usize,
    n_chunks: usize,
    trace: &[String],
    prev_trace: &[String],
) -> String {
    let curr = format!("QM31Impl::from_partial_evals([{}])", trace.join(", "));
    let mut prefix = curr;

    if !prev_trace.is_empty() {
        let prev = format!("QM31Impl::from_partial_evals([{}])", prev_trace.join(", "));
        prefix = format!(
            "{prefix} 
                - {prev}"
        );
    }

    if i == n_chunks - 1 {
        let neg = format!(
            "QM31Impl::from_partial_evals([{}])",
            trace.iter().map(|s| format!("{s}_neg1")).join(", ")
        );
        let claimed_sum = "(claimed_sum * (column_size.inverse().into()))".to_string();
        prefix = format!(
            "{prefix} 
                - {neg}
                + {claimed_sum}"
        );
    }

    prefix
}

fn get_sum_name_and_sign(i: usize, sum: &(String, UseOrYield)) -> (String, String) {
    let name = format!("{}_sum_{}", sum.0, i);
    let sign = if sum.1 == UseOrYield::Use { "-" } else { "+" };
    (name, sign.to_string())
}

// Given two relations sums from the same chunk, this function decides whether to multiply each of
// them in the numerator of the constraint quotient with the enabler/multiplicity column.
fn build_numerator(
    i: usize,
    n_chunks: usize,
    rel1: &String,
    rel2: &String,
    last_sum_chunk_has_2_elements: bool,
    is_chain: bool,
    has_enabler_or_mult_column: bool,
) -> (String, String) {
    let mut mult_rel1 = false;
    let mut mult_rel2 = false;

    if has_enabler_or_mult_column {
        // The first relation sum in the chunk is multiplied by the enabler/multiplicity in the
        // numerator if:
        // 1. it is second to last sum (i.e., we are at the last chunk that has 2 elements).
        // 2. it is the third to last sum (i.e., we are at the second to last chunk and last chunk
        //    has one element), and it is a chain.
        mult_rel1 =
            i == n_chunks - 1 || (i == n_chunks - 2 && is_chain && !last_sum_chunk_has_2_elements);
        // The second relation sum in the chunk is multiplied by the enabler/multipliicity in the
        // numerator if it is the last relation sum (i.e., we are at the last chunk), and it is a
        // chain.
        mult_rel2 = i == n_chunks - 1 && is_chain;
    }

    match (mult_rel1, mult_rel2) {
        (true, true) => (format!("({rel1} * enabler)"), format!("({rel2} * enabler)")),
        (true, false) => (format!("({rel1} * enabler)"), rel2.clone()),
        (false, true) => (rel1.clone(), format!("({rel2} * enabler)")),
        (false, false) => (rel1.clone(), rel2.clone()),
    }
}

fn get_interaction_trace_vars(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    let names = (0..n_logup_columns(air_fn))
        .map(|i| format!("trace_2_col{}", i))
        .collect::<Vec<_>>();
    if names.is_empty() {
        return code;
    }
    code.append(quote! {
        let $(format!("[{}]: [Span<QM31>; {}]", names.join(", "), names.len()))
            = (*interaction_trace_mask_values.multi_pop_front().unwrap()).unbox();$("\n")
    });
    for name in names[0..names.len() - QM31_N_TRACE_CELLTS].iter() {
        code.append(quote! {
            let [$(name)]: [QM31; 1] = (*$(name).try_into().unwrap()).unbox();$("\n")
        });
    }
    for name in names[names.len() - QM31_N_TRACE_CELLTS..].iter() {
        code.append(quote! {
            let [$(name)_neg1, $(name)]: [QM31; $(N_SAMPLES_FOR_PREFIX_SUM)] = (*$(name).try_into().unwrap()).unbox();$("\n")
        });
    }
    code
}
