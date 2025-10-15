pub mod cairo_constraints;
pub mod constraints;
pub mod parse;
pub mod supported_components;
pub mod trace_gen;
pub mod utils;

#[cfg(test)]
mod tests {
    use std::path::Path;

    use compiled_casm_air::compiled_structs::CompiledAirFn;
    use eval_air_fn_constraints::SampleEvaluation;

    use super::utils::load_air_fns;
    use crate::code_gen::supported_components::{AutogenCodeFile, AutogenCodeType};
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    fn generate_component_code(
        air_fn: &CompiledAirFn,
        sample_evaluation: Option<&SampleEvaluation>,
        job: &AutogenCodeFile,
    ) {
        const CONSTRAINTS_DIR: &str = "../code_gen_regression/cairo_air/src/components";
        const CAIRO_CONSTRAINTS_DIR: &str = "../code_gen_regression/verifier/src/components";
        const WITNESS_DIR: &str = "../code_gen_regression/witness/src/components";
        let [constraints_folder_path, cairo_constraints_dir, witness_folder_path] =
            [CONSTRAINTS_DIR, CAIRO_CONSTRAINTS_DIR, WITNESS_DIR]
                .map(|dir| project_root().join(dir));
        let path = match job.code_type {
            AutogenCodeType::WITNESS => witness_folder_path,
            AutogenCodeType::AIR => constraints_folder_path,
            AutogenCodeType::CAIRO => cairo_constraints_dir,
        };
        compare_contents_or_fix_with_path(air_fn, sample_evaluation, job, &path);
    }

    #[test]
    fn add_ap_cairo_code_gen() {
        let codegen_jobs = [
            AutogenCodeFile {
                air_fn_name: "add_ap_opcode".to_string(),
                source_path: "../compiled_casm_air/src/compiled_jsons/opcodes/add_ap_opcode.json".into(),
                code_type: AutogenCodeType::CAIRO,
            },
            AutogenCodeFile {
                air_fn_name: "read_small".to_string(),
                source_path: "../compiled_casm_air/src/compiled_jsons/subroutines/read_small.json".into(),
                code_type: AutogenCodeType::CAIRO,
            },
            AutogenCodeFile {
                air_fn_name: "decode_instruction_d2a10".to_string(),
                source_path: "../compiled_casm_air/src/compiled_jsons/subroutines/decode_instruction_d2a10.json".into(),
                code_type: AutogenCodeType::CAIRO,
            },
        ];

        let (compiled_air_fns, sample_evaluations) =
            load_air_fns(Path::new("../compiled_casm_air/src/"), &codegen_jobs);

        for job in codegen_jobs {
            let air_fn = compiled_air_fns
                .get(&job.air_fn_name)
                .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
            let sample_evaluation = sample_evaluations.get(&job.air_fn_name);
            generate_component_code(air_fn, sample_evaluation, &job);
        }
    }

    #[test]
    fn code_gen_regression() {
        let components_to_check = [
            "../compiled_casm_air/src/compiled_jsons/builtins/mul_mod_builtin.json",
            "../compiled_casm_air/src/compiled_jsons/builtins/range_check_builtin_bits_128.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/partial_ec_mul.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/pedersen_points_table.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/range_check_19.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_b.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/range_check_7_2_5.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/triple_xor_32.json",
            "../compiled_casm_air/src/compiled_jsons/lookups/verify_bitwise_xor_8.json",
            // cannot generate - mults component with inputs
            // "../compiled_casm_air/src/compiled_jsons/lookups/verify_instruction.json"
            "../compiled_casm_air/src/compiled_jsons/opcodes/jnz_opcode_taken.json",
        ];

        let inline_air_fns_to_check = [
            "../compiled_casm_air/src/compiled_jsons/subroutines/decode_instruction_d2a10.json",
            "../compiled_casm_air/src/compiled_jsons/subroutines/double_karatsuba_n_7_limb_max_bound_511.json",
            "../compiled_casm_air/src/compiled_jsons/subroutines/mem_verify.json",
            "../compiled_casm_air/src/compiled_jsons/subroutines/verify_add_252.json",
        ];

        // Generate witness code only for component AirFns
        let witness_jobs = components_to_check
            .iter()
            .map(|path_str| {
                let path = Path::new(path_str);
                let air_fn_name = path
                    .file_stem()
                    .expect("Invalid filename")
                    .to_str()
                    .expect("Invalid filename");

                AutogenCodeFile {
                    air_fn_name: air_fn_name.to_string(),
                    source_path: path.into(),
                    code_type: AutogenCodeType::WITNESS,
                }
            })
            .collect::<Vec<_>>();

        // Generate constraint evaluation code for both component and inline AirFns
        let constraint_jobs = inline_air_fns_to_check
            .iter()
            .chain(components_to_check.iter())
            .map(|path_str| {
                let path = Path::new(path_str);
                let air_fn_name = path
                    .file_stem()
                    .expect("Invalid filename")
                    .to_str()
                    .expect("Invalid filename");
                AutogenCodeFile {
                    air_fn_name: air_fn_name.to_string(),
                    source_path: path.into(),
                    code_type: AutogenCodeType::AIR,
                }
            })
            .collect::<Vec<_>>();

        let jobs = [constraint_jobs, witness_jobs].concat();

        let (compiled_air_fns, sample_evaluations) =
            load_air_fns(Path::new("../compiled_casm_air/src/"), &jobs);

        for job in jobs {
            let air_fn = compiled_air_fns
                .get(&job.air_fn_name)
                .unwrap_or_else(|| panic!("Missing AirFn {}", job.air_fn_name));
            let sample_evaluation = sample_evaluations.get(&job.air_fn_name);
            generate_component_code(air_fn, sample_evaluation, &job);
        }
    }
}
