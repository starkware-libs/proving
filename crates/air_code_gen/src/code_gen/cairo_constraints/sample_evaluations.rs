use std::collections::HashMap;
use std::path::Path;

use convert_case::{Case, Casing};
use eval_air_fn_constraints::SampleEvaluation;
use genco::lang::rust;
use genco::quote;
use indexmap::IndexMap;

use super::component::SAMPLE_EVALUATION_RESULT_SUFFIX;
use crate::code_gen::cairo_constraints::utils::format_cairo_code;
use crate::code_gen::supported_components::AutogenCodeType;
use crate::code_gen::utils::add_file_to_module;

pub fn generate_sample_evaluations_file(
    dest_dir: &Path,
    source_repo_rev: &str,
    sample_evaluations: &IndexMap<String, SampleEvaluation>,
) {
    let source_rev_comment = format!("// AIR version {}\n", source_repo_rev);

    let mut constants_to_write = HashMap::new();

    for (fn_name, evaluation) in sample_evaluations {
        let constant_name = format!(
            "{}{}",
            fn_name.to_case(Case::UpperSnake),
            SAMPLE_EVALUATION_RESULT_SUFFIX
        );

        constants_to_write.insert(constant_name, evaluation.result);
    }

    let mut tokens: rust::Tokens = quote! { use stwo_verifier_core::fields::m31::M31; $("\n") };

    let mut constant_order = constants_to_write.keys().collect::<Vec<_>>();
    constant_order.sort();
    for name in constant_order {
        let value = constants_to_write.get(name).unwrap();
        let value_m31s = value.to_m31_array();
        tokens.extend(quote! { pub const $(name): [M31; 4] = [M31 { inner: $(value_m31s[0].0) }, M31 { inner: $(value_m31s[1].0) }, M31 { inner: $(value_m31s[2].0) }, M31 { inner: $(value_m31s[3].0) }]; $("\n") });
    }

    add_file_to_module(
        &dest_dir.join("sample_evaluations.cairo"),
        format_cairo_code(source_rev_comment + &tokens.to_string().unwrap()),
        AutogenCodeType::CAIRO,
    );
}
