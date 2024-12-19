pub mod framework_gen;
pub mod simd_prover_gen;
pub mod utils;

#[cfg(test)]
mod tests {
    use compiled_casm_air::compiled_structs::CompiledAirFn;
    use compiled_casm_air::utils::read_json;
    use serde_json::from_value;

    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    fn generate_component_code(air_fn: CompiledAirFn) {
        const COMPONENTS_DIR: &str = "../generated_components/src/components";
        let folder_path = project_root().join(COMPONENTS_DIR);
        compare_contents_or_fix_with_path(air_fn, &folder_path);
    }

    // TODO(Ohad): consider moving these next to the corresponding infra code, when/if they are in a
    // separate crate.
    #[test]
    fn narrow_fib_gen() {
        let serialized_air_fn =
            read_json("../air_infra/src/airs/examples/test_jsons/narrow_fib_num_steps_20.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn wide_fib_code_gen() {
        let serialized_air_fn = read_json(
            "../air_infra/src/airs/examples/test_jsons/wide_fib_num_narrow_8_narrow_size_20.json",
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn add_ap_code_gen() {
        let serialized_air_fn = read_json(
            "../compiled_casm_air/src/opcodes/add_ap_opcode_is_imm_t_op_1_base_fp_f.json",
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn verify_instruction_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/lookups/verify_instruction.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn range_check_code_gen() {
        let serialized_air_fn = read_json("../compiled_casm_air/src/lookups/range_check_6.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn jnz_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/opcodes/jnz_opcode_is_taken_t_dst_base_fp_t.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }
}
