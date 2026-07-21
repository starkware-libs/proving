use std::path::Path;

use airs::casm::casm_registry::create_casm_registry_ordered_by_stwo_cairo;
use expect_test::expect_file;

use crate::cairo::claims::generate_claims_cairo_file;
use crate::cairo::utils::format_cairo_code;
use crate::supported_components::{AutogenCodeFile, AutogenCodeType};
use crate::test_utils::compare_contents_or_fix_with_path;
use crate::utils::load_air_fns;

#[test]
fn add_ap_cairo_code_gen() {
    let codegen_jobs = [
        AutogenCodeFile {
            air_fn_name: "add_ap_opcode".to_string(),
            source_path: "../../outputs/compiled_casm_air/compiled_jsons/opcodes/add_ap_opcode.\
                          json"
                .into(),
            dest_dir: "../../test_data/code_gen_regression/verifier/src/components".into(),
            code_type: AutogenCodeType::CAIRO,
        },
        AutogenCodeFile {
            air_fn_name: "read_small".to_string(),
            source_path: "../../outputs/compiled_casm_air/compiled_jsons/subroutines/read_small.\
                          json"
                .into(),
            dest_dir: "../../test_data/code_gen_regression/verifier/src/components".into(),
            code_type: AutogenCodeType::CAIRO,
        },
        AutogenCodeFile {
            air_fn_name: "decode_instruction_d2a10".to_string(),
            source_path: "../../outputs/compiled_casm_air/compiled_jsons/subroutines/\
                          decode_instruction_1af1f.json"
                .into(),
            dest_dir: "../../test_data/code_gen_regression/verifier/src/components".into(),
            code_type: AutogenCodeType::CAIRO,
        },
    ];

    let (compiled_air_fns, sample_evaluations) =
        load_air_fns(Path::new("../../outputs/compiled_casm_air/"), &codegen_jobs);

    for job in codegen_jobs {
        let air_fn = compiled_air_fns
            .get(&job.air_fn_name)
            .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
        let sample_evaluation = sample_evaluations.get(&job.air_fn_name);
        compare_contents_or_fix_with_path(air_fn, sample_evaluation, &job);
    }
}

#[test]
fn test_generate_claims_cairo() {
    let compiled_registry = create_casm_registry_ordered_by_stwo_cairo();
    let generated_code = generate_claims_cairo_file(&compiled_registry);
    let code_string = generated_code.to_string().unwrap();
    let formatted_code = format_cairo_code(code_string);
    expect_file!["../../../../test_data/code_gen_regression/cairo_air/src/claims.cairo"]
        .assert_eq(&formatted_code);
}
