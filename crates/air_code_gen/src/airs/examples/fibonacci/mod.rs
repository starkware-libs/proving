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

    use air_infra::airs::examples::fibonacci::fib::Fib;
    use air_infra::core::air_fn_registry::AirFnRegistry;
    use air_infra::core::prover_types::Felt;
    use itertools::{all, Itertools};
    use num_traits::One;
    use stwo_prover::core::air::Component;
    use stwo_prover::core::backend::cpu::CpuCircleEvaluation;
    use stwo_prover::core::fields::m31::BaseField;
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
    use crate::code_gen::component_gen::generate_component;
    use crate::code_gen::cpu_prover_gen::generate_cpu_prover_component;
    use crate::code_gen::packed_types::{PackedFelt, N_LANES};
    use crate::code_gen::simd_prover_gen::generate_simd_prover_component;
    use crate::code_gen::simd_trace_gen::generate_simd_trace_writer_code;
    use crate::code_gen::test_utils_gen::generate_test_air_code;
    use crate::code_gen::trace_gen::generate_trace_writer_code;
    use crate::code_gen::utils::{project_root, reformat_rust_code};

    // TODO(ShaharS): autogenerate this function and move to a test_utils file.
    fn assert_fib_constraints_on_trace(
        component: &dyn Component,
        trace: &[CpuCircleEvaluation<BaseField, BitReversedOrder>],
    ) {
        for j in 0..trace[0].len() {
            assert_eq!(trace[0][j].square() + BaseField::one(), trace[1][j]);
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
    fn fib_code_gen() {
        let air_fn = Fib { claim_index: 100 };
        let resigtry = AirFnRegistry::new(&air_fn);

        let mut folder_path = project_root();
        folder_path.push("src/airs/examples/fibonacci");

        let lists = resigtry.get_compiled_air_fn(&air_fn);
        let air_entry = resigtry.get_air_fn_entry(&air_fn);
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
    fn test_write_trace() {
        let fib_component = Fib__100 { log_n_instances: 2 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();

        let trace = write_trace_cpu(&fib_component, &secrets).0;
        assert_fib_constraints_on_trace(&fib_component, &trace);
    }

    #[test]
    fn test_simd_write_trace() {
        let fib_component = Fib__100 { log_n_instances: 7 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();
        let simd_secrets = secrets
            .iter()
            .copied()
            .array_chunks::<N_LANES>()
            .map(PackedFelt::from)
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
            .map(Felt::from)
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

        let inputs = (0..1 << LOG_N_INSTANCES).map(Felt::from).collect_vec();
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
            .map(Felt::from)
            .array_chunks::<N_LANES>()
            .map(PackedFelt::from)
            .collect::<Vec<_>>();

        let trace_generator = registry.get_generator_mut::<Fib__100SimdTraceGenerator>("fibonacci");
        trace_generator.add_inputs(&inputs);
        let trace = Fib__100SimdTraceGenerator::write_trace("fibonacci", &mut registry);
        let trace_generator = registry.get_generator::<Fib__100SimdTraceGenerator>("fibonacci");
        let component = trace_generator.component();

        let air = Fib__100TestAIR { component };
        test_prove(&air, trace);
    }
}
