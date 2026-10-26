use std::path::Path;

use eval_air_fn_constraints::SampleEvaluation;
use genco::lang::rust;
use genco::quote;
use indexmap::IndexMap;

use crate::cairo::utils::format_cairo_code;
use crate::supported_components::AutogenCodeType;
use crate::utils::*;

pub fn generate_sample_evaluations_file(
    dest_dir: &Path,
    sample_evaluations: &IndexMap<String, SampleEvaluation>,
) {
    let constants_to_write = sample_evaluations_consts(sample_evaluations);

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
        format_cairo_code(tokens.to_string().unwrap()),
        &AutogenCodeType::CAIRO,
    );
}
