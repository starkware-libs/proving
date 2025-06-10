pub mod cairo_constraints;
pub mod constraints;
pub mod parse;
pub mod supported_components;
pub mod trace_gen;
pub mod utils;

#[cfg(test)]
mod tests {
    use compiled_casm_air::compiled_structs::CompiledAirFn;
    use compiled_casm_air::utils::read_json;
    use serde_json::from_value;

    use crate::code_gen::cairo_constraints::utils::dump_component_cairo_constraints_code;
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    fn generate_component_code(air_fn: CompiledAirFn) {
        for inline_fn in air_fn.inline_calls.keys() {
            let serialized_inline_air_fn = read_json(&format!(
                "../compiled_casm_air/src/compiled_jsons/subroutines/{}.json",
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

    #[ignore = "Cannot run scarb fmt nor build"]
    #[test]
    fn add_ap_cairo_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/opcodes/add_ap_opcode.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        dump_component_cairo_constraints_code(&air_fn);
    }

    #[test]
    fn add_ap_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/opcodes/add_ap_opcode.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn jnz_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/opcodes/jnz_opcode_taken.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn mul_mod_builtin_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/builtins/mul_mod_builtin.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn partial_ec_mul_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/partial_ec_mul.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn pedersen_points_table_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/pedersen_points_table.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn rc128_builtin_code_gen() {
        let serialized_air_fn = read_json(
            "../compiled_casm_air/src/compiled_jsons/builtins/range_check_builtin_bits_128.json",
        );
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn range_check_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_6.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_8.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_12.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_18.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_18_b.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_b.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_c.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_d.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_e.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_f.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_g.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_19_h.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_b.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_c.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_d.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_e.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_f.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_g.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_9_9_h.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_4_3.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_7_2_5.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);

        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/range_check_3_6_6_3.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn triple_xor_32_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/triple_xor_32.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[test]
    fn verify_bitwise_xor_7_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/verify_bitwise_xor_8.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }

    #[ignore = "mults component with inputs"]
    #[test]
    fn verify_instruction_code_gen() {
        let serialized_air_fn =
            read_json("../compiled_casm_air/src/compiled_jsons/lookups/verify_instruction.json");
        let air_fn: CompiledAirFn = from_value(serialized_air_fn).unwrap();
        generate_component_code(air_fn);
    }
}
