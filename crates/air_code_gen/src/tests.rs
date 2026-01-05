use expect_test::expect_file;

use crate::cairo_claim_generator::generate_cairo_claim_generator_file;
use crate::utils::reformat_rust_code;

#[test]
fn test_generate_cairo_claim_generator() {
    let generated_code = generate_cairo_claim_generator_file();
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = reformat_rust_code(code_string);
    expect_file!["../../code_gen_regression/witness/src/cairo_claim_generator.rs"]
        .assert_eq(&formatted_code);
}
