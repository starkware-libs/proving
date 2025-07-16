use compiled_casm_air::compiled_structs::{CompiledAirFn, CompiledAirVar, ExternalState};
use convert_case::{Case, Casing};
use genco::lang::rust;
use genco::quote;
use indexmap::IndexSet;

use super::super::utils::get_variable_name;
use super::parse::{parse_constraints, parse_var};
use super::utils::{gen_consts, gen_imports};

// Subroutines that need extra calls to revoke_ap_tracking when compiled without qm31_opcode,
// otherwise you get an offset overflow error.
const LARGE_SUBROUTINES: [&str; 5] = [
    "double_karatsuba_n_7_limb_max_bound_511",
    "double_karatsuba_n_8_limb_max_bound_4095",
    "single_karatsuba_n_7",
    "single_karatsuba_n_8",
    "verify_add_252",
];

pub fn generate_inline_cairo_constraints_code(air_fn: &CompiledAirFn) -> rust::Tokens {
    let fn_name = air_fn.name.clone();
    let CompiledAirVar::Array(ref output_array) = air_fn.verifier_output.0 else {
        panic!("Verifier output is not array in {fn_name}")
    };

    let (input_name, input_type) = if air_fn.verifier_input_limbs.len() == 1 {
        (air_fn.verifier_input_limbs[0].clone(), "QM31".to_string())
    } else {
        (
            format!("[{}]", air_fn.verifier_input_limbs.join(", ")),
            format!("[QM31; {}]", air_fn.verifier_input_limbs.len()),
        )
    };

    let output_type = if output_array.len() == 1 {
        "QM31".to_string()
    } else {
        format!("[QM31; {}]", output_array.len())
    };
    let revoke_ap_tracking = LARGE_SUBROUTINES
        .contains(&air_fn.name.as_str())
        .then(|| "\ncore::internal::revoke_ap_tracking();\n".to_string())
        .unwrap_or_default();
    let mut code = rust::Tokens::new();

    code.append(quote! {
        $(gen_imports(air_fn))$("\n")
        $(gen_consts(air_fn))$("\n")
    });

    code.append(quote! {
        $("\n")
        pub fn $(fn_name)_evaluate(
            input: $(input_type),
            $(get_inline_args(air_fn))
            ref sum: QM31,
            domain_vanishing_eval_inv: QM31,
            random_coeff: QM31,
        ) -> $(output_type) {
            let $(input_name) = input;
            $(revoke_ap_tracking.clone())
            $(parse_constraints(air_fn))$("\n")
            $(revoke_ap_tracking)
            $(parse_var(air_fn, &air_fn.verifier_output.0, &mut 0))
        }
    });

    code
}

fn get_inline_args(air_fn: &CompiledAirFn) -> rust::Tokens {
    let mut code = rust::Tokens::new();
    for state_name in &air_fn.state_names {
        code.append(quote! {
            $(state_name): QM31,
        });
    }
    for relation in air_fn
        .lookup_names
        .iter()
        .map(|(r, _)| r)
        .collect::<IndexSet<_>>()
    {
        code.append(quote! {
            $(relation.to_case(Case::Snake))_lookup_elements: @crate::$(relation)Elements,
        });
    }
    for param in &air_fn.public_params {
        code.append(quote! {
            $(param.name()): QM31,
        });
    }
    for ExternalState { name, args, .. } in &air_fn.external_states {
        if name == "Seq" {
            code.append(quote! {
                seq: QM31,
            });
        } else {
            code.append(quote! {
                $(get_variable_name(name.to_lowercase().as_str(), args.join("_").as_str())): QM31,
            });
        }
    }
    for (i, (relation, _)) in air_fn.lookup_names.iter().enumerate() {
        code.append(quote! {
            ref $(relation.to_case(Case::Snake))_sum_$(i): QM31,
        });
    }
    code
}
