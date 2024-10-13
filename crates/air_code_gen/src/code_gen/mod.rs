pub mod framework_gen;
pub mod packed_types;
pub mod simd_prover_gen;
pub mod trace_gen;
pub mod utils;

#[cfg(test)]
mod tests {
    // TODO(AnatG): move these to a different crate
    use air_infra::core::compiled_structs::CompiledAirFn;
    use air_infra::core::utils::read_json;
    use serde_json::from_value;

    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    fn generate_component_code(air_fn: CompiledAirFn) {
        const COMPONENTS_DIR: &str = "../generated_components/src/";
        let folder_path = project_root().join(COMPONENTS_DIR);
        compare_contents_or_fix_with_path(air_fn, &folder_path);
    }

    // TODO(Ohad): consider moving these next to the corresponding infra code, when/if they are in a
    // separate crate.
    #[test]
    fn narrow_fib_gen() {
        let serialized_air_fn =
            read_json("../air_infra/src/airs/examples/test_jsons/narrowfib_num_steps_20.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn wide_fib_code_gen() {
        let serialized_air_fn = read_json(
            "../air_infra/src/airs/examples/test_jsons/widefib_num_narrow_8_narrow_size_20.json",
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn add_ap_code_gen() {
        let serialized_air_fn = read_json(
            "../air_infra/src/airs/examples/test_jsons/addapopcode_is_imm_t_op1_base_fp_f.json",
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn verify_instruction_code_gen() {
        let serialized_air_fn =
            read_json("../air_infra/src/airs/examples/test_jsons/verifyinstruction.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }
}
