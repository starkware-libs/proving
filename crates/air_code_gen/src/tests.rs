use air_infra::airs::casm::casm_registry::create_casm_registry_ordered_by_stwo_cairo;
use expect_test::expect_file;

use crate::claims_cairo::generate_claims_cairo_file;
use crate::claims_generator::generate_claim_generator_file;
use crate::claims_rust::generate_claims_rust_file;
use crate::components_code_gen::cairo_constraints::utils::format_cairo_code;
use crate::utils::reformat_rust_code;
#[test]
fn test_generate_claims_generator() {
    let compiled_regisry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_claim_generator_file(&compiled_regisry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../code_gen_regression/witness/src/claims_generator.rs"]
        .assert_eq(&formatted_code);
}

#[test]
fn test_generate_claims_rust() {
    let compiled_regisry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_claims_rust_file(&compiled_regisry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../code_gen_regression/cairo_air/src/claims.rs"].assert_eq(&formatted_code);
}

#[test]
fn test_generate_claims_cairo() {
    let compiled_regisry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_claims_cairo_file(&compiled_regisry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = format_cairo_code(code_string);
    expect_file!["../../code_gen_regression/cairo_air/src/claims.cairo"].assert_eq(&formatted_code);
}
