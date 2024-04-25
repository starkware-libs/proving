use air_infra::core::prover_types::Felt;

pub mod component;
pub mod trace;

pub struct FibInput {
    pub a: Felt,
    pub b: Felt,
}

#[cfg(test)]
mod tests {
    use air_infra::core::prover_types::Felt;
    use num_traits::{One, Zero};
    use stwo_prover::core::air::Component;
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::fields::FieldExpOps;

    use super::component::Fib__1000;
    use super::trace::write_trace_row;

    fn fill_trace(component: &dyn Component, secrets: &[Felt]) -> Vec<Vec<BaseField>> {
        let n_columns = component.trace_log_degree_bounds().len();
        let mut trace = vec![vec![BaseField::zero(); secrets.len()]; n_columns];
        for (i, secret) in secrets.iter().enumerate() {
            write_trace_row(&mut trace, *secret, i);
        }
        trace
    }

    fn assert_fib_constraints(component: &dyn Component, trace: &[Vec<BaseField>]) {
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
    fn test_write_trace() {
        let fib_component = Fib__1000 { log_n_instances: 2 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();

        let trace = fill_trace(&fib_component, &secrets);
        assert_fib_constraints(&fib_component, &trace);
    }
}
