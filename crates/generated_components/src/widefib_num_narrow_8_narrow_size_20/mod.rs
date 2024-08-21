pub mod component;
pub mod prover;

#[cfg(test)]
mod tests {
    use itertools::Itertools;
    use num_traits::One;
    use stwo_prover::core::backend::simd::column::BaseColumn;
    use stwo_prover::core::backend::simd::m31::PackedM31;
    use stwo_prover::core::fields::m31::M31;

    use super::prover::write_trace_simd;
    use crate::narrowfib_num_steps_20;

    #[test]
    fn wide_fib_test_write_trace() {
        let log_n_instances = 6;
        let inputs = BaseColumn::from_iter((0..1 << log_n_instances).map(M31::from));
        let inputs = inputs.data;

        let trace = write_trace_simd(inputs)
            .0
            .iter()
            .map(|x| x.data.to_vec())
            .collect_vec();

        for j in 0..trace[0].len() {
            for i in 0..((trace.len() / 2) - 1) {
                let input = if i == 0 {
                    [PackedM31::one(), trace[0][j]]
                } else {
                    [trace[2 * i - 1][j], trace[2 * i][j]]
                };
                let output = [trace[2 * i + 1][j], trace[2 * i + 2][j]];
                assert_eq!(
                    output[0].to_array(),
                    narrowfib_num_steps_20::deduce_output(input)[0].to_array()
                );
                assert_eq!(
                    output[1].to_array(),
                    narrowfib_num_steps_20::deduce_output(input)[1].to_array()
                );
            }
        }
    }
}
