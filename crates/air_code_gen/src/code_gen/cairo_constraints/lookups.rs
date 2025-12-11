use compiled_casm_air::compiled_structs::{CompiledAirFn, UseOrYield};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use super::utils::{n_logup_columns, QM31_N_TRACE_CELLTS};
use crate::code_gen::utils::relation_multiplicity_index;

pub const LOOKUP_RELATION_BATCH_SIZE: usize = 2;
pub const N_SAMPLES_FOR_PREFIX_SUM: usize = 2;

pub fn gen_lookup_constraints_fn(air_fn: &CompiledAirFn) -> rust::Tokens {
    let lookups = air_fn
        .constraint_lookups
        .iter()
        .enumerate()
        .map(|(i, (relation, _))| format!("{}_sum_{i}: QM31", relation.to_case(Case::Snake)))
        .collect::<Vec<_>>();
    let mults = air_fn
        .relation_names
        .iter()
        .map(|relation| format!("{}_multiplicity: QM31", relation.to_case(Case::Snake)))
        .collect::<Vec<_>>();

    // Revoke the AP tracking after defining the interaction trace vars, to avoid offset overflow.
    quote! {
        $("\n")$("\n")
        fn lookup_constraints(
            ref sum: QM31,
            domain_vanishing_eval_inv: QM31,
            random_coeff: QM31,
            claimed_sum: QM31,
            $(mults.join(",\n"))$((!mults.is_empty()).then(|| ",\n".to_string()).unwrap_or_default())
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
    let mut prev_trace = vec![];
    let n_chunks = n_logup_columns(air_fn) / QM31_N_TRACE_CELLTS;
    let last_sum_chunk_has_2_elements =
        air_fn.constraint_lookups.len() % LOOKUP_RELATION_BATCH_SIZE == 0;

    for (i, (trace_chunk, sum_chunk)) in (0..n_logup_columns(air_fn))
        .chunks(QM31_N_TRACE_CELLTS)
        .into_iter()
        .zip(air_fn.constraint_lookups.chunks(LOOKUP_RELATION_BATCH_SIZE))
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

        if i < n_chunks - 1 || (i == n_chunks - 1 && last_sum_chunk_has_2_elements) {
            let (rel2, rel2_sign) = get_sum_name_and_sign(sum_i + 1, &sum_chunk[1]);
            let rel1_t_rel2_mult = if relation_multiplicity_index(air_fn, &sum_chunk[1].0).is_some()
            {
                format!(
                    "({rel1} * {}_multiplicity)",
                    sum_chunk[1].0.to_case(Case::Snake)
                )
            } else {
                rel1.clone()
            };
            let rel2_t_rel1_mult = if relation_multiplicity_index(air_fn, &sum_chunk[0].0).is_some()
            {
                format!(
                    "({rel2} * {}_multiplicity)",
                    sum_chunk[0].0.to_case(Case::Snake)
                )
            } else {
                rel2.clone()
            };

            code.append(quote! {
                let constraint_quotient = (
                    (
                        (
                            $(prefix)
                        ) * $(rel1) * $(rel2)
                    ) $(rel2_sign) $(rel1_t_rel2_mult) $(rel1_sign) $(rel2_t_rel1_mult)
                ) * domain_vanishing_eval_inv;$("\n")
            });
        } else {
            let numerator = if !air_fn.relation_names.is_empty() {
                format!("{}_multiplicity", sum_chunk[0].0.to_case(Case::Snake))
            } else {
                "qm31_const::<1, 0, 0, 0>()".to_string()
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
    let name = format!("{}_sum_{}", sum.0.to_case(Case::Snake), i);
    let sign = if sum.1 == UseOrYield::Use { "-" } else { "+" };
    (name, sign.to_string())
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
