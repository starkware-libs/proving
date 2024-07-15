pub mod component;
pub mod cpu_prover;
pub mod simd_prover;
pub mod simd_trace;
#[cfg(test)]
pub mod test_utils;
pub mod trace;

#[cfg(test)]
mod tests {
    use air_infra::airs::examples::bit_unpacking::bit_unpack::BitUnpack;
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
    use crate::code_gen::packed_types::{PackedUInt16, N_LANES};
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    #[test]
    fn test_bit_unpack_constraints() {
        let component = BitUnpack__12 { log_n_instances: 7 };
        let inputs = (0..1 << component.log_n_instances)
            .map(UInt16::from)
            .collect_vec();

        let trace = write_trace_cpu(&component, &inputs).0;

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

    #[test]
    fn fixed_code_test() {
        let air_fn = BitUnpack::<12> {};
        let folder_path = project_root().join("src/airs/examples/bit_unpack");
        compare_contents_or_fix_with_path(&air_fn, &folder_path);
    }
}
