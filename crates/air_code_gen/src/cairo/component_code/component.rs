use air_compile::compiled_structs::CompiledAirFn;
use convert_case::{Case, Casing};
use eval_air_fn_constraints::assignment::Assignment;
use genco::lang::rust;
use genco::quote;
use itertools::Itertools;
use stwo_cairo_common::prover_types::cpu::QM31;

use super::claims::gen_claim_struct;
use super::lookups::gen_lookup_constraints_fn;
use super::parse::parse_constraints;
use crate::cairo::utils::{
    gen_consts, gen_imports, get_log_size, get_logup_sums, get_numerators, make_preprocessed_column,
};
use crate::utils::SAMPLE_EVALUATION_RESULT_SUFFIX;

pub fn generate_component_cairo_constraints_code(
    air_fn: &CompiledAirFn,
    sample_assignment: &Assignment,
) -> rust::Tokens {
    let mut result = quote! {
        $(gen_imports(air_fn))$("\n")
        $(gen_consts(air_fn))$("\n")
        $(gen_claim_struct(air_fn))$("\n")

        #[derive(Drop)]
        pub struct Component {
            pub claim: Claim,
            pub claimed_sum: QM31,
            pub common_lookup_elements: CommonLookupElements
        }

        pub impl NewComponentImpl of NewComponent<Component> {
            type Claim = Claim;

            fn new(
                claim: @Claim,
                claimed_sum: QM31,
                common_lookup_elements: @CommonLookupElements,
            ) -> Component {
                Component {
                    claim: *claim,
                    claimed_sum,
                    common_lookup_elements: common_lookup_elements.clone()
                }
            }
        }

        pub impl AirComponentImpl of AirComponent<Component> {
            fn evaluate_constraints_at_point(
                self: @Component,
                ref sum: QM31,
                ref preprocessed_mask_values: PreprocessedMaskValues,
                ref trace_mask_values: ColumnSpan<Span<QM31>>,
                ref interaction_trace_mask_values: ColumnSpan<Span<QM31>>,
                random_coeff: QM31,
            ) {
                let log_size = $(get_log_size(air_fn, false));
                let claimed_sum = *self.claimed_sum;
                let column_size = m31(pow2(log_size));
                $(get_evaluate_locals(air_fn))$("\n")
                $(get_trace_vars(air_fn))$("\n")
                // Revoke the AP tracking to avoid offset overflow.
                core::internal::revoke_ap_tracking();$("\n")
                $(parse_constraints(air_fn))

                lookup_constraints(
                    ref sum,
                    random_coeff,
                    claimed_sum,
                    $(get_numerators(air_fn).iter().map(|m| m.to_string() + ",\n").join(""))
                    column_size,
                    ref interaction_trace_mask_values,
                    $(get_logup_sums(air_fn).join(",\n"))
                );
            }
        }

        $(gen_lookup_constraints_fn(air_fn))

    };

    result.extend(gen_tests_module(air_fn, sample_assignment));

    result
}

fn gen_component_for_assignment(air_fn: &CompiledAirFn, assignment: &Assignment) -> rust::Tokens {
    let common_lookup_elements = &assignment.common_lookup_elements;
    let lookup_elements_fields = quote! {
        common_lookup_elements:
            CommonLookupElements { z: $(make_qm31(&common_lookup_elements.z)), alpha: $(make_qm31(&common_lookup_elements.alpha)) }, $("\n")
    };

    let claim_fields = match air_fn.log_height {
        Some(_fixed_size) => quote! {},
        None => quote! { log_size: $(assignment.log_height), $("\n") },
    };

    quote! {
        Component {
            claim: Claim { $(claim_fields) },
            claimed_sum: $(make_qm31(&assignment.claimed_sum)),
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
            .unwrap_or_else(|| panic!("Missing external state {external_state}"));
        let preprocessed_column = make_preprocessed_column(external_state);
        preprocessed_values.append(quote! {
                ($(preprocessed_column), $(make_qm31(external_column_value))), $("\n")
        });
    }

    let trace_values: rust::Tokens = assignment
        .base_trace
        .iter()
        .flat_map(|value| quote! { [$(make_qm31(value))].span(), $("\n") })
        .collect();

    let interaction_values: rust::Tokens = assignment
        .interaction_trace
        .iter()
        .flat_map(|value| quote! { $(make_qm31(value)), $("\n") })
        .collect();

    let expected_result_name =
        format!("{}{}", air_fn.name.to_case(Case::UpperSnake), SAMPLE_EVALUATION_RESULT_SUFFIX);

    quote! {
        // Compiling for the "poseidon verifier", i.e. without the QM31 opcode, makes evaluate_constraints_at_point
        // too long to compile for some components (e.g. generic_opcode). Therefore we only test the evaluation
        // result when the opcode is available.
        #[cfg(and(test, feature: "qm31_opcode"))]
        mod tests {
            use super::{Component, Claim};
            use crate::components::sample_evaluations::*;
            use stwo_constraint_framework::AirComponent;
            use core::array::ArrayImpl;
            use core::num::traits::Zero;
            #[allow(unused_imports)]
            use crate::preprocessed_columns::*;
            #[allow(unused_imports)]
            use stwo_constraint_framework::test_utils::{make_interaction_trace, new_preprocessed_mask};
            #[allow(unused_imports)]
            use stwo_constraint_framework::{CommonLookupElements, PreprocessedMaskValuesTrait};
            use stwo_verifier_core::fields::qm31::{qm31_const, QM31, QM31Impl, QM31Trait};

            #[test]
            fn test_evaluation_result() {
                let component = $(gen_component_for_assignment(air_fn, assignment));
                let mut sum: QM31 = Zero::zero();

                let mut preprocessed_trace = new_preprocessed_mask(array![
                    $(preprocessed_values)
                ].span(), NUM_PREPROCESSED_COLUMNS);

                let mut trace_columns = [ $(trace_values) ].span();
                let interaction_values = array![ $(interaction_values) ];
                let mut interaction_columns = make_interaction_trace(interaction_values, $(make_qm31(&assignment.last_row_sum)));
                component.evaluate_constraints_at_point(ref sum, ref preprocessed_trace, ref trace_columns, ref interaction_columns, $(make_qm31(&assignment.random_coeff)));
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

fn get_evaluate_locals(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    // Relation sums and numerators
    for (i, (relation, _)) in air_fn.constraint_lookups.iter().enumerate() {
        code.append(quote! {
            let mut $(relation.to_case(Case::Snake))_sum_$(i): QM31 = Zero::zero();
            let mut numerator_$(i): QM31 = Zero::zero();
        });
    }

    // External states
    for external_col_id in &air_fn.external_states {
        let variable_name = external_col_id.to_lowercase();

        code.append(quote! {
            let $(variable_name)
                = preprocessed_mask_values.get($(make_preprocessed_column(external_col_id)));
        });
    }

    code
}

fn get_trace_vars(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();

    let trace_names = air_fn.state_names.clone();
    if !trace_names.is_empty() {
        code.append(quote! {
            let $(format!("[{}]: [Span<QM31>; {}]", trace_names.join(", "), trace_names.len()))
                = (*trace_mask_values.multi_pop_front().unwrap()).unbox();
        });
    }

    for name in &air_fn.state_names {
        code.append(quote! {
            let [$(name)]: [QM31; 1] = (*$(name).try_into().unwrap()).unbox();
        });
    }

    code
}
