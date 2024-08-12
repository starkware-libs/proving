pub mod component;
pub mod prover;

#[cfg(test)]
mod tests {

    use air_infra::airs::examples::fibonacci::wide_fib::WideFib;
    use itertools::Itertools;
    use num_traits::One;
    use stwo_prover::core::backend::simd::column::BaseColumn;
    use stwo_prover::core::backend::simd::m31::PackedM31;
    use stwo_prover::core::fields::m31::M31;

    use super::prover::write_trace_simd;
    use crate::airs::examples::NarrowFib_1ddf31c88316e62f;
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    #[test]
    fn wide_fib_code_gen() {
        let air_fn = WideFib {
            num_narrow: 8,
            narrow_size: 20,
        };
        let mut folder_path = project_root();
        folder_path.push("src/airs/examples/wide_fib");
        compare_contents_or_fix_with_path(&air_fn, &folder_path);
    }

    #[test]
    fn wide_fib_test_write_trace() {
        let log_n_instances = 6;
        let inputs = BaseColumn::from_iter((0..1 << log_n_instances).map(M31::from));

        let trace = write_trace_simd(inputs.data)
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
                    NarrowFib_1ddf31c88316e62f::deduce_output(input)[0].to_array()
                );
                assert_eq!(
                    output[1].to_array(),
                    NarrowFib_1ddf31c88316e62f::deduce_output(input)[1].to_array()
                );
            }
        }
    }
}
