pub mod constraints;
pub mod parse;
pub mod trace_gen;
pub mod utils;

#[cfg(test)]
mod tests {
    use compiled_casm_air::compiled_structs::CompiledAirFn;
    use compiled_casm_air::utils::read_json;
    use serde_json::from_value;

    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    fn generate_component_code(air_fn: CompiledAirFn) {
        for inline_fn in air_fn.inline_calls.keys() {
            let serialized_inline_air_fn = read_json(&format!(
                "../compiled_casm_air/src/subroutines/{}.json",
                inline_fn
            ));
            let inline_air_fn: CompiledAirFn = from_value(serialized_inline_air_fn).unwrap();
            generate_component_code(inline_air_fn);
        }

        const CONSTRAINTS_DIR: &str = "../code_gen_regression/cairo_air/src/components";
        const WITNESS_DIR: &str = "../code_gen_regression/witness/src/components";
        let [constraints_folder_path, witness_folder_path] =
            [CONSTRAINTS_DIR, WITNESS_DIR].map(|dir| project_root().join(dir));
        compare_contents_or_fix_with_path(air_fn, &constraints_folder_path, &witness_folder_path);
    }

    // TODO(Gali): handle sub routines in example folder.
    #[ignore = "subroutines in example folder not handled yet"]
    #[test]
    fn narrow_fib_gen() {
        let serialized_air_fn =
            read_json("../air_infra/src/airs/examples/test_jsons/narrow_fib_num_steps_20.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[ignore = "missing deduce_output function"]
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
        let serialized_air_fn = read_json("../compiled_casm_air/src/opcodes/add_ap_opcode.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[ignore = "mults component with inputs"]
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

        let serialized_air_fn = read_json("../compiled_casm_air/src/lookups/range_check_12.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn = read_json("../compiled_casm_air/src/lookups/range_check_18.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn jnz_code_gen() {
        let serialized_air_fn = read_json("../compiled_casm_air/src/opcodes/jnz_opcode_taken.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn rc128_builtin_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/builtins/range_check_builtin_bits_128.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn mul_mod_builtin_code_gen() {
        let serialized_air_fn = read_json("../compiled_casm_air/src/builtins/mul_mod_builtin.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }
}
