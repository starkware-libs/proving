pub mod component;
pub mod cpu_prover;
pub mod simd_prover;
pub mod simd_trace;
pub mod test_utils;
pub mod trace;

#[cfg(test)]
mod tests {
    use std::fs;

    use air_infra::airs::examples::fibonacci::wide_fib::WideFib;
    use air_infra::core::air_fn_registry::AirFnRegistry;
    use air_infra::core::prover_types::Felt;
    use itertools::{all, izip, Itertools};
    use stwo_prover::core::fields::m31::M31;

    use super::component::WideFib__8;
    use super::trace::write_trace_cpu;
    use crate::airs::examples::narrow_fibonacci::trace::NarrowFib__20CpuTraceGenerator;
    use crate::airs::examples::wide_fib::simd_trace::write_trace_simd;
    use crate::code_gen::packed_types::{PackedFelt, N_LANES};
    use crate::code_gen::simd_trace_gen::generate_simd_trace_writer_code;
    use crate::code_gen::trace_gen::generate_trace_writer_code;
    use crate::code_gen::utils::{project_root, reformat_rust_code};

    #[test]
    fn wide_fib_code_gen() {
        let air_fn = WideFib {
            num_narrow: 8,
            narrow_size: 20,
        };
        let resigtry = AirFnRegistry::new(&air_fn);

        let mut folder_path = project_root();
        folder_path.push("src/airs/examples/wide_fib");

        let air_entry = resigtry.get_air_fn_entry(&air_fn);
        let lists = resigtry.get_compiled_air_fn(&air_fn);
        let name = air_entry.name.to_string() + "__" + &air_fn.num_narrow.to_string();
        let trace_tokens = generate_trace_writer_code(&name, &lists.input, &lists.deductions);
        let simd_trace_tokens =
            generate_simd_trace_writer_code(&name, &lists.input, &lists.deductions);
        // Write the generated code to files.
        let text = reformat_rust_code(trace_tokens.to_string().unwrap());
        fs::write(folder_path.join("trace.rs"), text).unwrap();
        let text = reformat_rust_code(simd_trace_tokens.to_string().unwrap());
        fs::write(folder_path.join("simd_trace.rs"), text).unwrap();
    }

    #[test]
    fn wide_fib_test_write_trace() {
        let wide_fib_component = WideFib__8 { log_n_instances: 6 };

        let secrets = (0..1 << wide_fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();

        let trace = write_trace_cpu(&wide_fib_component, &secrets).0;

        for j in 0..trace[0].len() {
            for i in 0..((trace.len() / 2) - 1) {
                let input = if i == 0 {
                    [M31::from(1), trace[0][j]]
                } else {
                    [trace[2 * i - 1][j], trace[2 * i][j]]
                };
                let output = [trace[2 * i + 1][j], trace[2 * i + 2][j]];
                assert_eq!(output, NarrowFib__20CpuTraceGenerator::deduce_output(input))
            }
        }
    }

    #[test]
    fn wide_fib_test_write_trace_inputs() {
        let wide_fib_component = WideFib__8 { log_n_instances: 6 };

        let secrets = (0..1 << wide_fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();

        let (trace, inputs) = write_trace_cpu(&wide_fib_component, &secrets);

        let const_1_column = vec![M31::from(1); trace[0].len()];
        let trace_column_0 = trace[0].to_vec();
        let inputs_0 = inputs.0;
        izip!(const_1_column, trace_column_0, inputs_0)
            .for_each(|(one, trace_0, input)| assert_eq!([one, trace_0], input));

        let trace_column_13 = trace[13].to_vec();
        let trace_column_14 = trace[14].to_vec();
        let inputs_7 = inputs.7;
        izip!(trace_column_13, trace_column_14, inputs_7)
            .for_each(|(trace_13, trace_14, input)| assert_eq!([trace_13, trace_14], input));
    }

    #[test]
    fn wide_fib_simd_trace_test() {
        let wide_fib_component = WideFib__8 { log_n_instances: 8 };

        let secrets = (0..1 << wide_fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();
        let raw_cpu_trace = write_trace_cpu(&wide_fib_component, &secrets)
            .0
            .into_iter()
            .map(|eval| eval.as_slice().to_vec())
            .collect_vec();
        let simd_secrets = secrets
            .into_iter()
            .array_chunks::<N_LANES>()
            .map(PackedFelt::from_array)
            .collect_vec();

        let raw_simd_trace = write_trace_simd(&wide_fib_component, &simd_secrets)
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
}
