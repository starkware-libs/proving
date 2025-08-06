use compiled_casm_air::compiled_structs::CompiledAirFn;
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use indexmap::IndexSet;

use super::super::utils::get_variable_name;
use super::claims::{gen_claim_struct, gen_interaction_claim_struct};
use super::lookups::gen_lookup_constraints_fn;
use super::parse::parse_constraints;
use super::utils::{
    gen_consts, gen_imports, get_log_size, has_enabler_or_mult_column, make_preprocessed_column,
    n_logup_columns, QM31_N_TRACE_CELLTS,
};

pub fn generate_component_cairo_constraints_code(air_fn: &CompiledAirFn) -> rust::Tokens {
    let lookups = air_fn
        .lookup_names
        .iter()
        .enumerate()
        .map(|(i, (relation, _))| format!("{}_sum_{i}", relation.to_case(Case::Snake)))
        .collect::<Vec<_>>();

    quote! {
        $(gen_imports(air_fn))$("\n")
        $(gen_consts(air_fn))$("\n")
        $(gen_claim_struct(air_fn))$("\n")
        $(gen_interaction_claim_struct())$("\n")

        #[derive(Drop)]
        pub struct Component {
            pub claim: Claim,
            pub interaction_claim: InteractionClaim,
            $(air_fn.lookup_names.iter().map(|(r, _)| r).collect::<IndexSet<_>>().iter().map(|relation| {
                format!(
                    "pub {}_lookup_elements: crate::{relation}Elements,", relation.to_case(Case::Snake)
                )
            }).collect::<Vec<_>>().join("\n"))
        }

        pub impl ComponentImpl of CairoComponent<Component> {
            fn mask_points(
                self: @Component,
                ref preprocessed_column_set: PreprocessedColumnSet,
                ref trace_mask_points: ColumnArray<Array<CirclePoint<QM31>>>,
                ref interaction_trace_mask_points: ColumnArray<Array<CirclePoint<QM31>>>,
                point: CirclePoint<QM31>,
            ) {
                let log_size = $(get_log_size(air_fn, false));
                let trace_gen = CanonicCosetImpl::new(log_size).coset.step;
                let point_offset_neg_1 = point.add_circle_point_m31(-trace_gen.mul(1).to_point());
                $(gen_mask_points(air_fn))
            }

            fn max_constraint_log_degree_bound(self: @Component) -> u32 {
                $(get_log_size(air_fn, false)) + 1
            }

            fn evaluate_constraints_at_point(
                self: @Component,
                ref sum: QM31,
                ref preprocessed_mask_values: PreprocessedMaskValues,
                ref trace_mask_values: ColumnSpan<Span<QM31>>,
                ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
                random_coeff: QM31,
                point: CirclePoint<QM31>,
            ) {
                let log_size = $(get_log_size(air_fn, false));
                let trace_domain = CanonicCosetImpl::new(log_size);
                let domain_vanishing_eval_inv = trace_domain.eval_vanishing(point).inverse();
                let claimed_sum = *self.interaction_claim.claimed_sum;
                let column_size = m31(pow2(log_size));
                $(get_evaluate_locals(air_fn))$("\n")
                $(get_trace_vars(air_fn))$("\n")
                // Revoke the AP tracking to avoid offset overflow.
                core::internal::revoke_ap_tracking();$("\n")
                $(parse_constraints(air_fn))

                lookup_constraints(
                    ref sum,
                    domain_vanishing_eval_inv,
                    random_coeff,
                    claimed_sum,
                    $(has_enabler_or_mult_column(air_fn).then(||
                        "enabler,".to_string()
                    ).unwrap_or_default())
                    column_size,
                    ref interaction_trace_mask_values,
                    $(lookups.join(",\n"))
                );
            }
        }

        $(gen_lookup_constraints_fn(air_fn))
    }
}

fn get_evaluate_locals(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    // Public params
    for param in &air_fn.public_params {
        code.append(quote!{
            let $(param.name()): QM31 = (TryInto::<u32, M31>::try_into((*(self.claim.$(param.name())))).unwrap()).into();
        });
    }

    // Relation sums
    for (i, (relation, _)) in air_fn.lookup_names.iter().enumerate() {
        code.append(quote! {
            let mut $(relation.to_case(Case::Snake))_sum_$(i): QM31 = Zero::zero();
        });
    }

    // External states
    for external_state in &air_fn.external_states {
        let variable_name = if external_state.name == "Seq" {
            "seq"
        } else {
            &get_variable_name(
                external_state.name.to_lowercase().as_str(),
                external_state.args.join("_").as_str(),
            )
        };

        code.append(quote! {
            let $(variable_name)
                = preprocessed_mask_values.get($(make_preprocessed_column(air_fn, external_state)));
        });
    }

    code
}

fn gen_mask_points(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    // Generate preprocessed column set
    for external_state in &air_fn.external_states {
        code.append(quote! {
            preprocessed_column_set.insert($(make_preprocessed_column(air_fn, external_state)));
        });
    }

    // Generate trace mask
    for _name in &air_fn.state_names {
        code.append(quote! {
            trace_mask_points.append(array![point]);
        });
    }
    if has_enabler_or_mult_column(air_fn) {
        code.append(quote! {
            trace_mask_points.append(array![point]);
        });
    }

    // Generate interaction trace mask
    if n_logup_columns(air_fn) == 0 {
        return code;
    }

    // In a component with lookups, the last constraint is a prefix sum in QM31. Hence, the last 4
    // columns are sampled at a -1 offset.
    for _i in 0..(n_logup_columns(air_fn) - QM31_N_TRACE_CELLTS) {
        code.append(quote! {
            interaction_trace_mask_points.append(array![point]);
        });
    }
    // NOTE: The protocol is sensitive to the ordering of offsets.
    // This must agree with the ordering in the prover logup constraints.
    code.append(quote! {
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
        interaction_trace_mask_points.append(array![point_offset_neg_1, point]);
    });

    code
}

fn get_trace_vars(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    let has_state_vars = !air_fn.state_names.is_empty();
    let has_enabler = has_enabler_or_mult_column(air_fn);

    match (has_state_vars, has_enabler) {
        (true, true) => {
            code.append(quote! {
                let $(format!("[{}, enabler]: [Span<QM31>; {}]", air_fn.state_names.join(", "), air_fn.state_names.len() + 1)) 
                    = (*trace_mask_values.multi_pop_front().unwrap()).unbox();
            });
        }
        (true, false) => {
            code.append(quote! {
                let $(format!("[{}]: [Span<QM31>; {}]", air_fn.state_names.join(", "), air_fn.state_names.len())) 
                    = (*trace_mask_values.multi_pop_front().unwrap()).unbox();
            });
        }
        (false, true) => {
            code.append(quote! {
                let [enabler]: [Span<QM31>; 1] = (*trace_mask_values.multi_pop_front().unwrap()).unbox();
            });
        }
        (false, false) => {}
    }

    for name in &air_fn.state_names {
        code.append(quote! {
            let [$(name)]: [QM31; 1] = (*$(name).try_into().unwrap()).unbox();
        });
    }

    if has_enabler {
        code.append(quote! {
            let [enabler]: [QM31; 1] = (*enabler.try_into().unwrap()).unbox();
        });
    }

    code
}
