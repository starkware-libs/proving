use air_common::UseOrYield;
use air_compile::compiled_structs::CompiledAirFn;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;

use crate::cairo::utils::{QM31_N_TRACE_CELLTS, get_lookup_sums, get_numerators, n_logup_columns};

pub const LOOKUP_RELATION_BATCH_SIZE: usize = 2;
pub const N_SAMPLES_FOR_PREFIX_SUM: usize = 2;

pub fn gen_lookup_constraints_fn(air_fn: &CompiledAirFn) -> rust::Tokens {
    // Revoke the AP tracking after defining the interaction trace vars, to avoid offset overflow.
    quote! {
        $("\n")$("\n")
        fn lookup_constraints(
            ref sum: QM31,
            random_coeff: QM31,
            claimed_sum: QM31,
            $(get_numerators(air_fn).iter().map(|m| m.to_string() + ": QM31,\n").join(""))
            column_size: M31,
            ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
            $(get_lookup_sums(air_fn).iter().map(|m| m.to_string() + ": QM31,\n").join(""))
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

    for (i, (trace_chunk, sum_chunk)) in (0..n_logup_columns(air_fn))
        .chunks(QM31_N_TRACE_CELLTS)
        .into_iter()
        .zip(air_fn.constraint_lookups.chunks(LOOKUP_RELATION_BATCH_SIZE))
        .enumerate()
    {
        code.append(quote! { $("\n") });
        let trace = trace_chunk.into_iter().map(|i| format!("trace_2_col{i}")).collect::<Vec<_>>();
        let prefix = get_lookup_constraints_prefix(i, n_chunks, &trace, &prev_trace);
        let sum_i = i * LOOKUP_RELATION_BATCH_SIZE;
        let (rel1, rel1_sign) = get_sum_name_and_sign(sum_i, &sum_chunk[0]);

        if sum_chunk.len() == 2 {
            let (rel2, rel2_sign) = get_sum_name_and_sign(sum_i + 1, &sum_chunk[1]);
            let rel1_times_rel2_mult = format!("({rel1} * numerator_{})", sum_i + 1);
            let rel2_times_rel1_mult = format!("({rel2} * numerator_{sum_i})");

            code.append(quote! {
                let constraint_quotient = (
                    (
                        (
                            $(prefix)
                        ) * $(rel1) * $(rel2)
                    ) $(rel2_sign) $(rel1_times_rel2_mult) $(rel1_sign) $(rel2_times_rel1_mult)
                );$("\n")
            });
        } else {
            let numerator = format!("numerator_{sum_i}");

            code.append(quote! {
                let constraint_quotient = (
                    (
                        (
                            $(prefix)
                        ) * $(rel1)
                    ) $(rel1_sign) $(numerator)
                );$("\n")
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
    let names = (0..n_logup_columns(air_fn)).map(|i| format!("trace_2_col{i}")).collect::<Vec<_>>();
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
