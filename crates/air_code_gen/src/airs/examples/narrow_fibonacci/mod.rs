use simd_trace::NarrowFib__20SimdTraceGenerator;
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::fields::m31::M31;
use stwo_prover::core::fields::FieldExpOps;
use trace::NarrowFib__20CpuTraceGenerator;

pub mod component;
pub mod cpu_prover;
pub mod simd_prover;
pub mod simd_trace;
pub mod trace;

impl NarrowFib__20CpuTraceGenerator {
    pub fn deduce_output(input: [M31; 2]) -> [M31; 2] {
        let mut state = input;
        for _ in 0..20 {
            let next = [state[1], state[0].square() + state[1].square()];
            state = next;
        }
        state
    }
}

impl NarrowFib__20SimdTraceGenerator {
    pub fn deduce_output(input: [PackedM31; 2]) -> [PackedM31; 2] {
        let mut state = input;
        for _ in 0..20 {
            let next = [state[1], state[0].square() + state[1].square()];
            state = next;
        }
        state
    }
}
#[cfg(test)]
pub mod test_utils;

#[cfg(test)]
mod tests {
    use air_infra::airs::examples::fibonacci::narrow_fib::NarrowFib;
    use itertools::{all, Itertools};
    use stwo_prover::core::air::Component;
    use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
    use stwo_prover::core::backend::simd::m31::PackedM31;
    use stwo_prover::core::backend::simd::SimdBackend;
    use stwo_prover::core::fields::m31::{BaseField, M31};
    use stwo_prover::core::fields::FieldExpOps;
    use stwo_prover::core::poly::BitReversedOrder;
    use stwo_prover::trace_generation::registry::ComponentGenerationRegistry;
    use stwo_prover::trace_generation::TraceGenerator;

    use super::component::NarrowFib__20;
    use super::test_utils::NarrowFib__20TestAIR;
    use super::trace::{write_trace_cpu, NarrowFib__20CpuTraceGenerator};
    use crate::airs::examples::narrow_fibonacci::simd_trace::{
        write_trace_simd, NarrowFib__20SimdTraceGenerator,
    };
    use crate::airs::examples::test_utils::{assert_cpu_constraints, test_prove};
    use crate::code_gen::packed_types::N_LANES;
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    pub fn assert_fib_constraints_on_trace(
        component: &dyn Component,
        trace: &[CpuCircleEvaluation<BaseField, BitReversedOrder>],
    ) {
        for j in 0..trace[0].len() {
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
        let fib_component = NarrowFib__20 { log_n_instances: 2 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(|i| [M31::from(i + 1), M31::from(i + 4)])
            .collect::<Vec<_>>();

        let trace = write_trace_cpu(&fib_component, &secrets).0;
        assert_fib_constraints_on_trace(&fib_component, &trace);
    }

    #[test]
    fn test_simd_write_trace() {
        let narrow_fib_component = NarrowFib__20 { log_n_instances: 7 };
        let secrets = (0..1 << narrow_fib_component.log_n_instances)
            .map(|i| [M31::from(i + 1), M31::from(i + 4)])
            .collect::<Vec<_>>();
        let simd_secrets: Vec<[PackedM31; 2]> = secrets
            .iter()
            .copied()
            .array_chunks::<N_LANES>()
            .map(|c| {
                [
                    PackedM31::from_array(std::array::from_fn(|i| c[i][0])),
                    PackedM31::from_array(std::array::from_fn(|i| c[i][1])),
                ]
            })
            .collect();
        // Convert trace to raw values. Backends should produce the same values.
        let raw_cpu_trace = write_trace_cpu(&narrow_fib_component, &secrets)
            .0
            .into_iter()
            .map(|eval| eval.as_slice().to_vec())
            .collect_vec();

        let raw_simd_trace = write_trace_simd(&narrow_fib_component, &simd_secrets)
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
        ));
    }

    #[test]
    fn test_fib_constraints() {
        let fib_component = NarrowFib__20 { log_n_instances: 7 };
        let inputs = (0..1 << fib_component.log_n_instances)
            .map(|i| [M31::from(i + 1), M31::from(i + 4)])
            .collect_vec();
        let trace = write_trace_cpu(&fib_component, &inputs).0;

        assert_cpu_constraints(&fib_component, trace);
    }

    #[test]
    fn test_fib_cpu_prove() {
        const LOG_N_INSTANCES: u32 = 7;
        let mut registry = ComponentGenerationRegistry::default();
        let fib_trace_gen = NarrowFib__20CpuTraceGenerator::default();
        registry.register("narrow_fib", fib_trace_gen);
        let trace_generator =
            registry.get_generator_mut::<NarrowFib__20CpuTraceGenerator>("narrow_fib");
        let inputs = (0..1 << LOG_N_INSTANCES)
            .map(|i| [M31::from(i + 1), M31::from(i + 4)])
            .collect_vec();
        trace_generator.add_inputs(&inputs);
        let trace = NarrowFib__20CpuTraceGenerator::write_trace("narrow_fib", &mut registry);
        let component = registry
            .get_generator::<NarrowFib__20CpuTraceGenerator>("narrow_fib")
            .component();

        let air = NarrowFib__20TestAIR { component };

        test_prove(&air, trace);
    }

    #[test]
    fn test_fib_simd_prove() {
        const LOG_N_INSTANCES: u32 = 7;
        let mut registry = ComponentGenerationRegistry::default();
        let fib_trace_gen = NarrowFib__20SimdTraceGenerator::default();
        registry.register("narrow_fib", fib_trace_gen);

        let secrets = (0..1 << LOG_N_INSTANCES)
            .map(|i| [M31::from(i + 1), M31::from(i + 4)])
            .collect::<Vec<_>>();
        let simd_secrets: Vec<[PackedM31; 2]> = secrets
            .iter()
            .copied()
            .array_chunks::<N_LANES>()
            .map(|c| {
                [
                    PackedM31::from_array(std::array::from_fn(|i| c[i][0])),
                    PackedM31::from_array(std::array::from_fn(|i| c[i][1])),
                ]
            })
            .collect();
        let trace_generator =
            registry.get_generator_mut::<NarrowFib__20SimdTraceGenerator>("narrow_fib");
        trace_generator.add_inputs(&simd_secrets);
        let trace = NarrowFib__20SimdTraceGenerator::write_trace("narrow_fib", &mut registry);
        let component = registry
            .get_generator::<NarrowFib__20SimdTraceGenerator>("narrow_fib")
            .component();
        let air = NarrowFib__20TestAIR { component };

        test_prove::<SimdBackend>(&air, trace);
    }

    #[test]
    fn generated_code_is_the_same_test() {
        let air_fn = NarrowFib { num_steps: 20 };
        let folder_path = project_root().join("src/airs/examples/narrow_fibonacci");
        compare_contents_or_fix_with_path(&air_fn, &folder_path);
    }
}
