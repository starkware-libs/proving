pub mod component;
pub mod cpu_prover;
pub mod trace;

#[cfg(test)]
mod tests {
    use std::fs;
    use std::iter::zip;

    use air_infra::core::air_fn_registry::AirFnRegistry;
    use air_infra::core::prover_types::Felt;
    use air_infra::fibonacci::fib::Fib;
    use itertools::Itertools;
    use num_traits::{One, Zero};
    use stwo_prover::core::air::Component;
    use stwo_prover::core::backend::cpu::CPUCircleEvaluation;
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::fields::FieldExpOps;
    use stwo_prover::core::poly::circle::CanonicCoset;
    use stwo_prover::core::poly::BitReversedOrder;

    use super::component::{Fib__100, Fib__100TestAIR};
    use super::trace::write_trace_row;
    use crate::airs::examples::test_utils::{assert_cpu_constraints, test_prove};
    use crate::code_gen::component_gen::generate_component;
    use crate::code_gen::cpu_prover_gen::generate_cpu_prover_component;
    use crate::code_gen::trace_gen::gen_write_trace_code;
    use crate::code_gen::utils::{project_root, reformat_rust_code};

    fn fill_trace(
        component: &dyn Component,
        secrets: &[Felt],
    ) -> Vec<CPUCircleEvaluation<BaseField, BitReversedOrder>> {
        let n_columns = component.trace_log_degree_bounds().len();
        let mut trace_values = vec![vec![BaseField::zero(); secrets.len()]; n_columns];
        for (i, secret) in secrets.iter().enumerate() {
            write_trace_row(&mut trace_values, *secret, i);
        }
        let trace_domains = trace_values
            .iter()
            .map(|col| CanonicCoset::new(col.len().ilog2()).circle_domain())
            .collect_vec();
        zip(trace_values, trace_domains)
            .map(|(eval, trace_domain)| {
                CPUCircleEvaluation::<BaseField, BitReversedOrder>::new(trace_domain, eval)
            })
            .collect_vec()
    }

    // TODO(ShaharS): autogenerate this function and move to a test_utils file.
    fn assert_fib_constraints_on_trace(
        component: &dyn Component,
        trace: &[CPUCircleEvaluation<BaseField, BitReversedOrder>],
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
        let (_, air_entry, lists) = AirFnRegistry::new(&air_fn);

        let mut folder_path = project_root();
        folder_path.push("src/airs/examples/fibonacci");

        let trace_tokens = gen_write_trace_code(lists.input.clone(), &lists.deductions.clone());
        let cpu_prover_tokens =
            generate_cpu_prover_component(&air_entry.name, &lists.constraints.clone());
        let component_tokens = generate_component(&air_entry.name, lists);

        // Write the generated code to files.
        let text = reformat_rust_code(trace_tokens.to_string().unwrap());
        fs::write(folder_path.join("trace.rs"), text).unwrap();
        let text = reformat_rust_code(cpu_prover_tokens.to_string().unwrap());
        fs::write(folder_path.join("cpu_prover.rs"), text).unwrap();
        let text = reformat_rust_code(component_tokens.to_string().unwrap());
        fs::write(folder_path.join("component.rs"), text).unwrap();
    }

    #[test]
    fn test_write_trace() {
        let fib_component = Fib__100 { log_n_instances: 2 };
        let secrets = (0..1 << fib_component.log_n_instances)
            .map(Felt::from)
            .collect::<Vec<_>>();

        let trace = fill_trace(&fib_component, &secrets);
        assert_fib_constraints_on_trace(&fib_component, &trace);
    }

    #[test]
    fn test_fib_constraints() {
        let fib_component = Fib__100 { log_n_instances: 7 };
        let inputs = (0..1 << fib_component.log_n_instances)
            .map(Felt::from)
            .collect_vec();
        let trace = fill_trace(&fib_component, &inputs);

        assert_cpu_constraints(&fib_component, trace);
    }

    #[test]
    fn test_fib_prove() {
        let air = Fib__100TestAIR {
            component: Fib__100 { log_n_instances: 7 },
        };
        let inputs = (0..1 << air.component.log_n_instances)
            .map(Felt::from)
            .collect_vec();

        let trace = fill_trace(&air.component, &inputs);

        test_prove(&air, trace);
    }
}
