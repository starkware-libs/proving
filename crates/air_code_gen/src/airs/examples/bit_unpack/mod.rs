pub mod component;
pub mod cpu_prover;
pub mod simd_prover;
pub mod simd_trace;
#[cfg(test)]
pub mod test_utils;
pub mod trace;

#[cfg(test)]
mod tests {
    use std::fs;

    use air_infra::airs::examples::bit_unpacking::bit_unpack::BitUnpack;
    use air_infra::core::air_fn_registry::AirFnRegistry;
    use air_infra::core::prover_types::UInt16;
    use itertools::Itertools;

    use super::component::BitUnpack__12;
    use super::simd_trace::write_trace_simd;
    use super::test_utils::BitUnpack__12TestAIR;
    use super::trace::write_trace;
    use crate::airs::examples::test_utils::{assert_cpu_constraints, test_prove};
    use crate::code_gen::component_gen::generate_component;
    use crate::code_gen::cpu_prover_gen::generate_cpu_prover_component;
    use crate::code_gen::packed_types::{PackedUInt16, N_LANES};
    use crate::code_gen::simd_prover_gen::generate_simd_prover_component;
    use crate::code_gen::simd_trace_gen::generate_simd_write_trace_code;
    use crate::code_gen::test_utils_gen::generate_test_air_code;
    use crate::code_gen::trace_gen::generate_write_trace_code;
    use crate::code_gen::utils::{project_root, reformat_rust_code};

    #[test]
    fn bit_unpack_code_gen() {
        let air_fn = BitUnpack { n_bits: 12 };
        let resigtry = AirFnRegistry::new(&air_fn);

        let mut folder_path = project_root();
        folder_path.push("src/airs/examples/bit_unpack");

        let air_entry = resigtry.get_air_fn_entry(&air_fn);
        let lists = resigtry.get_codegen_air_fn(&air_fn);
        let trace_tokens = generate_write_trace_code(
            &air_entry.name,
            lists.input.clone(),
            &lists.deductions.clone(),
        );
        let simd_trace_tokens = generate_simd_write_trace_code(
            &air_entry.name,
            lists.input.clone(),
            &lists.deductions.clone(),
        );
        let cpu_prover_tokens =
            generate_cpu_prover_component(&air_entry.name, &lists.constraints.clone());
        let simd_prover_tokens =
            generate_simd_prover_component(&air_entry.name, &lists.constraints.clone());
        let component_tokens = generate_component(&air_entry.name, lists);
        let test_utils_tokens = generate_test_air_code(&air_entry.name);

        // Write the generated code to files.
        let text = reformat_rust_code(trace_tokens.to_string().unwrap());
        fs::write(folder_path.join("trace.rs"), text).unwrap();
        let text = reformat_rust_code(simd_trace_tokens.to_string().unwrap());
        fs::write(folder_path.join("simd_trace.rs"), text).unwrap();
        let text = reformat_rust_code(cpu_prover_tokens.to_string().unwrap());
        fs::write(folder_path.join("cpu_prover.rs"), text).unwrap();
        let text = reformat_rust_code(simd_prover_tokens.to_string().unwrap());
        fs::write(folder_path.join("simd_prover.rs"), text).unwrap();
        let text = reformat_rust_code(component_tokens.to_string().unwrap());
        fs::write(folder_path.join("component.rs"), text).unwrap();
        let text = reformat_rust_code(test_utils_tokens.to_string().unwrap());
        fs::write(folder_path.join("test_utils.rs"), text).unwrap();
    }

    #[test]
    fn test_bit_unpack_constraints() {
        let component = BitUnpack__12 { log_n_instances: 7 };
        let inputs = (0..1 << component.log_n_instances)
            .map(UInt16::from)
            .collect_vec();

        let trace = write_trace(&component, &inputs);

        assert_cpu_constraints(&component, trace);
    }

    #[test]
    fn test_bit_unpack_prove() {
        let air = BitUnpack__12TestAIR {
            component: BitUnpack__12 { log_n_instances: 7 },
        };
        let inputs = (0..1 << air.component.log_n_instances)
            .map(UInt16::from)
            .collect_vec();

        let trace = write_trace(&air.component, &inputs);

        test_prove(&air, trace);
    }

    #[test]
    fn test_bit_unpack_simd_prove() {
        let air = BitUnpack__12TestAIR {
            component: BitUnpack__12 { log_n_instances: 7 },
        };
        let inputs = (0..1 << air.component.log_n_instances)
            .map(UInt16::from)
            .array_chunks::<N_LANES>()
            .map(PackedUInt16::from_array)
            .collect_vec();

        let trace = write_trace_simd(&air.component, &inputs);

        test_prove(&air, trace);
    }
}
