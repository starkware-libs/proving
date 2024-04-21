use air_infra::core::prover_types::Felt;

pub mod trace;

pub struct FibInput {
    pub a: Felt,
    pub b: Felt,
}

pub struct WideFibComponent {
    pub log_n_instances: u32,
}

#[cfg(test)]
mod tests {
    use air_infra::core::prover_types::Felt;
    use num_traits::Zero;
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::fields::FieldExpOps;

    use super::trace::write_trace_row;
    use super::{FibInput, WideFibComponent};

    fn fill_trace(secrets: &[Felt]) -> Vec<Vec<BaseField>> {
        let mut trace = vec![vec![BaseField::zero(); secrets.len()]; 1000];
        for (i, secret) in secrets.iter().enumerate() {
            write_trace_row(
                &mut trace,
                FibInput {
                    a: Felt { value: 1 },
                    b: *secret,
                },
                i,
            );
        }
        trace
    }

    fn assert_fib_constraints(trace: &[Vec<BaseField>]) {
        for j in 0..trace[0].len() {
            for i in 2..1000 {
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
        let fib_component = WideFibComponent { log_n_instances: 2 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();

        let trace = fill_trace(&secrets);
        assert_fib_constraints(&trace);
    }
}
