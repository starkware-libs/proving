pub mod component;
pub mod cpu_prover;
pub mod simd_prover;
pub mod simd_trace;
#[cfg(test)]
pub mod test_utils;
pub mod trace;

#[cfg(test)]
mod tests {
    use air_infra::airs::examples::fibonacci::fib::Fib;
    use itertools::{all, Itertools};
    use num_traits::One;
    use stwo_prover::core::air::Component;
    use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
    use stwo_prover::core::backend::simd::m31::PackedM31;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::fields::FieldExpOps;
    use stwo_prover::core::poly::BitReversedOrder;
    use stwo_prover::trace_generation::{registry, TraceGenerator};

    use super::component::Fib__100;
    use super::simd_trace::write_trace_simd;
    use super::test_utils::Fib__100TestAIR;
    use super::trace::write_trace_cpu;
    use crate::airs::examples::fibonacci::simd_trace::Fib__100SimdTraceGenerator;
    use crate::airs::examples::fibonacci::trace::Fib__100CpuTraceGenerator;
    use crate::airs::examples::test_utils::{assert_cpu_constraints, test_prove};
    use crate::code_gen::packed_types::N_LANES;
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    // TODO(ShaharS): autogenerate this function and move to a test_utils file.
    fn assert_fib_constraints_on_trace(
        component: &dyn Component,
        trace: &[CpuCircleEvaluation<M31, BitReversedOrder>],
    ) {
        for j in 0..trace[0].len() {
            assert_eq!(trace[0][j].square() + M31::one(), trace[1][j]);
            for i in 2..component.n_constraints() {
                assert_eq!(
                    trace[i][j],
                    trace[i - 1][j].square() + trace[i - 2][j].square(),
                    "Fibonacci constraint failed at index {}",
                    i
                );
            }
        }
    }

    #[test]
    fn test_write_trace() {
        let fib_component = Fib__100 { log_n_instances: 2 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(M31::from)
            .collect::<Vec<_>>();

        let trace = write_trace_cpu(&fib_component, &secrets).0;
        assert_fib_constraints_on_trace(&fib_component, &trace);
    }

    #[test]
    fn test_simd_write_trace() {
        let fib_component = Fib__100 { log_n_instances: 7 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(M31::from)
            .collect::<Vec<_>>();
        let simd_secrets = secrets
            .iter()
            .copied()
            .array_chunks::<N_LANES>()
            .map(PackedM31::from)
            .collect::<Vec<_>>();

        // Convert trace to raw values. Backends should produce the same values.
        let raw_cpu_trace = write_trace_cpu(&fib_component, &secrets)
            .0
            .into_iter()
            .map(|eval| eval.as_slice().to_vec())
            .collect_vec();

        let raw_simd_trace = write_trace_simd(&fib_component, &simd_secrets)
            .0
            .into_iter()
            .map(|eval| eval.as_slice().to_vec())
            .collect_vec();

        assert!(all(
            raw_cpu_trace.iter().zip_eq(raw_simd_trace),
            |(cpu_col, simd_col)| {
                cpu_col
                    .iter()
                    .zip_eq(simd_col)
                    .all(|(&cpu, simd)| cpu == simd)
            }
        ))
    }

    #[test]
    fn test_fib_constraints() {
        let fib_component = Fib__100 { log_n_instances: 7 };
        let inputs = (0..1 << fib_component.log_n_instances)
            .map(M31::from)
            .collect_vec();
        let trace = write_trace_cpu(&fib_component, &inputs).0;

        assert_cpu_constraints(&fib_component, trace);
    }

    #[test]
    fn test_fib_cpu_prove() {
        const LOG_N_INSTANCES: u32 = 7;
        let mut registry = registry::ComponentGenerationRegistry::default();
        let fib_trace_gen = Fib__100CpuTraceGenerator::default();
        registry.register("fibonacci", fib_trace_gen);

        let inputs = (0..1 << LOG_N_INSTANCES).map(M31::from).collect_vec();
        let trace_generator = registry.get_generator_mut::<Fib__100CpuTraceGenerator>("fibonacci");
        trace_generator.add_inputs(&inputs);
        let trace = Fib__100CpuTraceGenerator::write_trace("fibonacci", &mut registry);
        let trace_generator = registry.get_generator::<Fib__100CpuTraceGenerator>("fibonacci");
        let component = trace_generator.component();

        let air = Fib__100TestAIR { component };
        test_prove(&air, trace);
    }

    #[test]
    fn test_fib_simd_prove() {
        const LOG_N_INSTANCES: u32 = 7;
        let mut registry = registry::ComponentGenerationRegistry::default();
        let fib_simd_trace_gen = Fib__100SimdTraceGenerator::default();
        registry.register("fibonacci", fib_simd_trace_gen);

        let inputs = (0..1 << LOG_N_INSTANCES)
            .map(M31::from)
            .array_chunks::<N_LANES>()
            .map(PackedM31::from)
            .collect::<Vec<_>>();

        let trace_generator = registry.get_generator_mut::<Fib__100SimdTraceGenerator>("fibonacci");
        trace_generator.add_inputs(&inputs);
        let trace = Fib__100SimdTraceGenerator::write_trace("fibonacci", &mut registry);
        let trace_generator = registry.get_generator::<Fib__100SimdTraceGenerator>("fibonacci");
        let component = trace_generator.component();

        let air = Fib__100TestAIR { component };
        test_prove(&air, trace);
    }

    #[test]
    fn generated_code_is_the_same_test() {
        let air_fn = Fib { claim_index: 100 };
        let folder_path = project_root().join("src/airs/examples/fibonacci");
        compare_contents_or_fix_with_path(&air_fn, &folder_path);
    }
}
