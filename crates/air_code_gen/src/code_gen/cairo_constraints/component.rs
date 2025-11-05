use std::collections::HashSet;

use compiled_casm_air::compiled_structs::CompiledAirFn;
use convert_case::{Case, Casing};
use eval_air_fn_constraints::assignment::Assignment;
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;
use stwo_cairo_common::prover_types::cpu::QM31;

use super::claims::{gen_claim_struct, gen_interaction_claim_struct};
use super::lookups::gen_lookup_constraints_fn;
use super::parse::parse_constraints;
use super::utils::{
    gen_consts, gen_imports, get_log_size, has_enabler_or_mult_column, make_preprocessed_column,
    n_logup_columns, QM31_N_TRACE_CELLTS,
};
use crate::code_gen::cairo_constraints::utils::lookup_elements_field;
use crate::code_gen::utils::{is_const_size_component, relations_used_or_yielded};

pub const SAMPLE_EVALUATION_RESULT_SUFFIX: &str = "_SAMPLE_EVAL_RESULT";

pub fn generate_component_cairo_constraints_code(
    air_fn: &CompiledAirFn,
    sample_assignment: &Assignment,
) -> rust::Tokens {
    let lookups = air_fn
        .constraint_lookups
        .iter()
        .enumerate()
        .map(|(i, (relation, _))| format!("{}_sum_{i}", relation.to_case(Case::Snake)))
        .collect::<Vec<_>>();

    let mut result = quote! {
        $(gen_imports(air_fn))$("\n")
        $(gen_consts(air_fn))$("\n")
        $(gen_claim_struct(air_fn))$("\n")
        $(gen_interaction_claim_struct())$("\n")

        #[derive(Drop)]
        pub struct Component {
            pub claim: Claim,
            pub interaction_claim: InteractionClaim,
            $(relations_used_or_yielded(air_fn).iter().map(|relation| {
                format!(
                    "pub {}: crate::{relation}Elements,", lookup_elements_field(relation)
                )
            }).collect::<Vec<_>>().join("\n"))
        }

        pub impl NewComponentImpl of NewComponent<Component> {
            type Claim = Claim;
            type InteractionClaim = InteractionClaim;

            fn new(
                claim: @Claim,
                interaction_claim: @InteractionClaim,
                interaction_elements: @CairoInteractionElements,
            ) -> Component {
                Component {
                    claim: *claim,
                    interaction_claim: *interaction_claim,
                    $(relations_used_or_yielded(air_fn).iter().map(|relation| {
                        format!(
                            "{}: interaction_elements.{}.clone(),", lookup_elements_field(relation), get_interaction_name(relation.to_case(Case::Snake))
                        )
                    }).collect::<Vec<_>>().join("\n"))
                }
            }
        }

        pub impl CairoComponentImpl of CairoComponent<Component> {
            fn mask_points(
                self: @Component,
                ref preprocessed_column_set: PreprocessedColumnSet,
                ref trace_mask_points: ColumnArray<Array<CirclePoint<QM31>>>,
                ref interaction_trace_mask_points: ColumnArray<Array<CirclePoint<QM31>>>,
                point: CirclePoint<QM31>,
            ) {
                let trace_gen = CanonicCosetImpl::new($(get_log_size(air_fn, false))).coset.step;
                let point_offset_neg_1 = point.add_circle_point_m31(-trace_gen.mul(1).to_point());
                $(gen_mask_points(air_fn))
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

    };

    // TODO(az-starkware): Implement the sample evaluation test for const-size components too
    if !is_const_size_component(air_fn) {
        result.extend(gen_tests_module(air_fn, sample_assignment));
    }

    result
}

fn gen_component_for_assignment(air_fn: &CompiledAirFn, assignment: &Assignment) -> rust::Tokens {
    let relation_names = air_fn
        .constraint_lookups
        .iter()
        .map(|x| x.0.clone())
        .collect::<HashSet<_>>()
        .into_iter()
        .sorted()
        .collect::<Vec<_>>();

    let mut lookup_elements_fields: Vec<rust::Tokens> = vec![];
    for relation_name in relation_names.iter() {
        let lookup_elements = assignment
            .lookup_elements
            .get(relation_name)
            .unwrap_or_else(|| panic!("Missing relation {relation_name} in assignment"));
        lookup_elements_fields
                    .push(quote! {
                        $(lookup_elements_field(relation_name)):
                            make_lookup_elements($(make_qm31(&lookup_elements.z)), $(make_qm31(&lookup_elements.alpha))), $("\n")
                    });
    }

    let mut claim_fields = quote! { log_size: $(assignment.log_height), $("\n") };

    for param in &air_fn.public_params {
        let param_value = assignment
            .environment
            .public_params
            .get(&param.name())
            .unwrap_or_else(|| panic!("Missing public param {param:?} in assignment"));
        claim_fields.append(quote! {
            $(param.name()): $(param_value.0), $("\n")
        });
    }

    quote! {
        Component {
            claim: Claim { $(claim_fields) },
            interaction_claim: InteractionClaim { claimed_sum: $(make_qm31(&assignment.claimed_sum)) },
            $(lookup_elements_fields)
        }
    }
}

fn gen_tests_module(air_fn: &CompiledAirFn, assignment: &Assignment) -> rust::Tokens {
    let mut preprocessed_values = quote! {};

    for external_state in air_fn.external_states.iter() {
        let external_column_value = assignment
            .environment
            .external_states
            .get(external_state)
            .unwrap_or_else(|| panic!("Missing external state {}", external_state));
        let preprocessed_column =
            make_preprocessed_column(external_state, &quote! { component.claim.log_size });
        preprocessed_values.append(quote! {
                    let mut preprocessed_trace = preprocessed_mask_add(preprocessed_trace, $(preprocessed_column), $(make_qm31(external_column_value))); $("\n")
                });
    }

    let trace_values: rust::Tokens = assignment
        .base_trace
        .iter()
        .chain(assignment.lookup_control_value.iter())
        .flat_map(|value| quote! { [$(make_qm31(value))].span(), $("\n") })
        .collect();

    let interaction_values: rust::Tokens = assignment
        .interaction_trace
        .iter()
        .flat_map(|value| quote! { $(make_qm31(value)), $("\n") })
        .collect();

    let expected_result_name = format!(
        "{}{}",
        air_fn.name.to_case(Case::UpperSnake),
        SAMPLE_EVALUATION_RESULT_SUFFIX
    );

    quote! {
        // Compiling for the "poseidon verifier", i.e. without the QM31 opcode, makes evaluate_constraints_at_point
        // too long to compile for some components (e.g. generic_opcode). Therefore we only test the evaluation
        // result when the opcode is available.
        #[cfg(and(test, feature: "qm31_opcode"))]
        mod tests {
            use super::{Component, Claim, InteractionClaim};
            use crate::utils::*;
            use crate::components::sample_evaluations::*;
            use crate::cairo_component::*;
            use core::array::ArrayImpl;
            use core::num::traits::Zero;
            #[allow(unused_imports)]
            use stwo_cairo_air::preprocessed_columns::{seq_column_idx, NUM_PREPROCESSED_COLUMNS};
            #[allow(unused_imports)]
            use crate::test_utils::{make_lookup_elements, make_interaction_trace, preprocessed_mask_add};
            #[allow(unused_imports)]
            use stwo_constraint_framework::{LookupElements, PreprocessedMaskValues};
            use stwo_verifier_core::circle::CirclePoint;
            use stwo_verifier_core::fields::qm31::{qm31_const, QM31, QM31Impl, QM31Trait};

            #[test]
            fn test_evaluation_result() {
                let component = $(gen_component_for_assignment(air_fn, assignment));
                let mut sum: QM31 = Zero::zero();
                let point = CirclePoint {
                    x: $(make_qm31(&assignment.point.0)),
                    y: $(make_qm31(&assignment.point.1))
                };

                let mut preprocessed_trace = PreprocessedMaskValues { values: [Default::default(); NUM_PREPROCESSED_COLUMNS].span().into() };
                $(preprocessed_values)

                let mut trace_columns = [ $(trace_values) ].span();
                let interaction_values = array![ $(interaction_values) ];
                let mut interaction_columns = make_interaction_trace(interaction_values, $(make_qm31(&assignment.last_row_sum)));
                component.evaluate_constraints_at_point(ref sum, ref preprocessed_trace, ref trace_columns, ref interaction_columns, $(make_qm31(&assignment.random_coeff)), point);
                assert_eq!(sum, QM31Trait::from_fixed_array($(expected_result_name)))
            }
        }
    }
}

fn make_qm31(value: &QM31) -> rust::Tokens {
    let value_components = value.to_m31_array();
    quote! {
        qm31_const::<$(value_components[0].0), $(value_components[1].0), $(value_components[2].0), $(value_components[3].0)>()
    }
}

fn get_interaction_name(relation: String) -> String {
    match relation.as_str() {
        "range_check_252_width_27" => relation,
        range_check if range_check.starts_with("range_check") => {
            relation.replace("range_check_", "range_checks.rc_")
        }
        "memory_id_to_big" => "memory_id_to_value".to_string(),
        _ => relation,
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
    for (i, (relation, _)) in air_fn.constraint_lookups.iter().enumerate() {
        code.append(quote! {
            let mut $(relation.to_case(Case::Snake))_sum_$(i): QM31 = Zero::zero();
        });
    }

    // External states
    for external_col_id in &air_fn.external_states {
        let variable_name = external_col_id.to_lowercase();

        code.append(quote! {
            let $(variable_name)
                = preprocessed_mask_values.get($(make_preprocessed_column(external_col_id, &get_log_size(air_fn, false))));
        });
    }

    code
}

fn gen_mask_points(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    // Generate preprocessed column set
    for external_state in &air_fn.external_states {
        code.append(quote! {
            preprocessed_column_set.insert($(make_preprocessed_column(external_state, &get_log_size(air_fn, false))));
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
