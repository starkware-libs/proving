pub mod prover;

pub use prover::{ClaimGenerator, InputType, OutputType};
use stwo_prover::core::backend::simd::m31::PackedM31;
use stwo_prover::core::fields::FieldExpOps;

// TODO(Ohad): auto gen and remove.
pub struct Claim {
    pub log_size: u32,
    pub n_calls: usize,
}

pub fn deduce_output(input: [PackedM31; 2]) -> [PackedM31; 2] {
    let mut state = input;
    for _ in 0..20 {
        let next = [state[1], state[0].square() + state[1].square()];
        state = next;
    }
    state
}

#[cfg(test)]
mod tests {

    use air_infra::airs::examples::fibonacci::narrow_fib::NarrowFib;
    use itertools::Itertools;
    use stwo_prover::core::backend::simd::m31::PackedM31;
    use stwo_prover::core::backend::Column;
    use stwo_prover::core::fields::m31::M31;
    use stwo_prover::core::fields::FieldExpOps;

    use super::prover::write_trace_simd;
    use crate::code_gen::utils::{compare_contents_or_fix_with_path, project_root};

    pub fn assert_fib_constraints_on_trace(trace: &[Vec<M31>]) {
        for j in 0..trace[0].len() {
            for i in 2..trace.len() {
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
        let log_n_instances = 6;
        let inputs = (0..1 << (log_n_instances))
            .map(|i| {
                [
                    PackedM31::broadcast(M31::from(i + 1)),
                    PackedM31::broadcast(M31::from(i + 4)),
                ]
            })
            .collect_vec();

        let (packed_trace, _) = write_trace_simd(inputs);
        let trace = packed_trace
            .into_iter()
            .map(|x| x.values.to_cpu())
            .collect_vec();
        assert_fib_constraints_on_trace(&trace);
    }

    #[test]
    fn generated_code_is_the_same_test() {
        let air_fn = NarrowFib { num_steps: 20 };
        let folder_path = project_root().join("src/airs/examples/narrow_fibonacci");
        compare_contents_or_fix_with_path(&air_fn, &folder_path);
    }
}
