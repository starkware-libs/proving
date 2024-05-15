use air_infra::core::prover_types::Felt;

pub mod component;
pub mod cpu_prover;
pub mod trace;

// TODO(ShaharS): move this struct to another file and autogenerate it.
pub struct FibInput {
    pub a: Felt,
    pub b: Felt,
}

#[cfg(test)]
mod tests {
    use air_infra::core::prover_types::Felt;
    use itertools::Itertools;
    use num_traits::{One, Zero};
    use stwo_prover::core::air::Component;
    use stwo_prover::core::backend::cpu::CPUCircleEvaluation;
    use stwo_prover::core::channel::{Blake2sChannel, Channel};
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::fields::FieldExpOps;
    use stwo_prover::core::poly::circle::CanonicCoset;
    use stwo_prover::core::poly::BitReversedOrder;
    use stwo_prover::core::prover::{prove, verify};
    use stwo_prover::core::vcs::blake2_hash::Blake2sHash;

    use super::component::{Fib__1000, Fib__1000TestAIR};
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

    #[test]
    fn test_prove() {
        let fib_component = Fib__1000 { log_n_instances: 7 };
        let air = Fib__1000TestAIR {
            component: fib_component,
        };
        let inputs = (0..1 << air.component.log_n_instances)
            .map(Felt::from)
            .collect_vec();
        let trace = fill_trace(&air.component, &inputs);

        let trace_domain = CanonicCoset::new(air.component.log_n_instances).circle_domain();
        let trace = trace
            .into_iter()
            .map(|eval| CPUCircleEvaluation::<BaseField, BitReversedOrder>::new(trace_domain, eval))
            .collect_vec();

        // TODO(ShaharS): update channel digest with initial seed.
        let prover_channel = &mut Blake2sChannel::new(Blake2sHash::default());
        let proof = prove(&air, prover_channel, trace).unwrap();

        let verifier_channel = &mut Blake2sChannel::new(Blake2sHash::default());
        verify(proof, &air, verifier_channel).unwrap();
    }
}
