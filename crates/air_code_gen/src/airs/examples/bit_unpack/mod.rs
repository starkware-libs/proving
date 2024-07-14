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
    use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
    use stwo_prover::trace_generation::TraceGenerator;

    use super::component::BitUnpack__12;
    use super::test_utils::BitUnpack__12TestAIR;
    use super::trace::write_trace_cpu;
    use crate::airs::examples::bit_unpack::simd_trace::BitUnpack__12SimdTraceGenerator;
    use crate::airs::examples::bit_unpack::trace::BitUnpack__12CpuTraceGenerator;
    use crate::airs::examples::test_utils::{assert_cpu_constraints, test_prove};
    use crate::code_gen::component_gen::generate_component;
    use crate::code_gen::cpu_prover_gen::generate_cpu_prover_component;
    use crate::code_gen::packed_types::{PackedUInt16, N_LANES};
    use crate::code_gen::simd_prover_gen::generate_simd_prover_component;
    use crate::code_gen::simd_trace_gen::generate_simd_trace_writer_code;
    use crate::code_gen::test_utils_gen::generate_test_air_code;
    use crate::code_gen::trace_gen::generate_trace_writer_code;
    use crate::code_gen::utils::{project_root, reformat_rust_code};

    #[test]
    fn bit_unpack_code_gen() {
        let air_fn = BitUnpack::<12> {};
        let resigtry = AirFnRegistry::new(&air_fn);

        let mut folder_path = project_root();
        folder_path.push("src/airs/examples/bit_unpack");

        let air_entry = resigtry.get_air_fn_entry(&air_fn);
        let lists = resigtry.get_compiled_air_fn(&air_fn);
        let trace_tokens =
            generate_trace_writer_code(&air_entry.name, &lists.input, &lists.deductions);
        let simd_trace_tokens = generate_simd_trace_writer_code(
            &air_entry.name,
            &lists.input.clone(),
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

        let trace = write_trace_cpu(&component, &inputs);

        assert_cpu_constraints(&component, trace);
    }

    #[test]
    fn test_bit_unpack_cpu_prove() {
        const LOG_N_INSTANCES: u32 = 7;
        let mut registry = ComponentGenerationRegistry::default();
        let fib_trace_gen = BitUnpack__12CpuTraceGenerator::default();
        registry.register("bit_unpack", fib_trace_gen);

        let inputs = (0..1 << LOG_N_INSTANCES).map(UInt16::from).collect_vec();

        let trace_generator =
            registry.get_generator_mut::<BitUnpack__12CpuTraceGenerator>("bit_unpack");
        trace_generator.add_inputs(&inputs);
        let trace = BitUnpack__12CpuTraceGenerator::write_trace("bit_unpack", &mut registry);
        let trace_generator =
            registry.get_generator::<BitUnpack__12CpuTraceGenerator>("bit_unpack");
        let component = trace_generator.component();
        let air = BitUnpack__12TestAIR { component };

        test_prove(&air, trace);
    }

    #[test]
    fn test_bit_unpack_simd_prove() {
        const LOG_N_INSTANCES: u32 = 7;
        let mut registry = ComponentGenerationRegistry::default();
        let fib_trace_gen = BitUnpack__12SimdTraceGenerator::default();
        registry.register("bit_unpack", fib_trace_gen);

        let inputs = (0..1 << LOG_N_INSTANCES)
            .map(UInt16::from)
            .array_chunks::<N_LANES>()
            .map(PackedUInt16::from_array)
            .collect_vec();

        let trace_generator =
            registry.get_generator_mut::<BitUnpack__12SimdTraceGenerator>("bit_unpack");
        trace_generator.add_inputs(&inputs);
        let trace = BitUnpack__12SimdTraceGenerator::write_trace("bit_unpack", &mut registry);
        let trace_generator =
            registry.get_generator::<BitUnpack__12SimdTraceGenerator>("bit_unpack");
        let component = trace_generator.component();
        let air = BitUnpack__12TestAIR { component };

        test_prove(&air, trace);
    }
}
