use air_infra::core::autogen_structs::{ConstraintOrIntermediate, ProcessedAirVar};
use genco::lang::rust;
use genco::quote;

pub fn generate_cpu_prover_component(
    component_name: &str,
    constraints: &[ConstraintOrIntermediate],
) -> rust::Tokens {
    quote! {
        $(imports_code(component_name))
        $['\n']
        impl ComponentProver<CPUBackend> for $(component_name) {
            fn evaluate_constraint_quotients_on_domain(
                &self,
                trace: &ComponentTrace<'_, CPUBackend>,
                evaluation_accumulator: &mut DomainEvaluationAccumulator<CPUBackend>,
            ) {
                $("// Numerator computation.")
                $(numerator_code(constraints))
                $("\n// Denominator computation.")
                $(denomerator_code())
                $("\n// Accumulate constraints.")
                $(accumulation_code())
            }
        }
    }
}

fn imports_code(component_name: &str) -> rust::Tokens {
    quote! {
        use num_traits::identities::Zero;
        use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
        use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
        use stwo_prover::core::backend::{CPUBackend, Column};
        use stwo_prover::core::constraints::coset_vanishing;
        use stwo_prover::core::fields::m31::BaseField;
        use stwo_prover::core::fields::qm31::SecureField;
        use stwo_prover::core::fields::FieldExpOps;
        use stwo_prover::core::poly::circle::CanonicCoset;
        use stwo_prover::core::utils::bit_reverse;

        use super::component::$(component_name);
    }
}

fn numerator_code(constraints: &[ConstraintOrIntermediate]) -> rust::Tokens {
    // TODO(ShaharS): accumulate each constraint according to its degree.
    let mut numerator_code = quote! {
        let trace_evals = &trace.evals;
        let mut numerators =
            vec![SecureField::zero(); 1 << (self.max_constraint_log_degree_bound())];
        let [mut accum] =
            evaluation_accumulator.columns(
                [(self.max_constraint_log_degree_bound(), self.n_constraints())],
            );
    };

    let mut constraints_code = quote! {};
    let n_constraints = constraints.len();
    let mut constraint_offset = 0;
    for constraint in constraints.iter() {
        match constraint {
            ConstraintOrIntermediate::Constraint(expr) => {
                constraints_code.extend(quote! {
                    *numer +=
                        accum.random_coeff_powers[$(n_constraints - 1 - constraint_offset)] *
                        ($(parse_cpu_prover_constraint(expr)));
                });
                constraint_offset += 1;
            }
            ConstraintOrIntermediate::Intermediate(var, expr) => {
                constraints_code.extend(quote! {
                    let $(var) = $(parse_cpu_prover_constraint(expr));
                });
            }
        }
    }
    numerator_code.extend(quote! {
        for (i, numer) in numerators.iter_mut().enumerate()
        {
            $(constraints_code)
        }
    });

    numerator_code
}

fn denomerator_code() -> rust::Tokens {
    quote! {
        let zero_domain = CanonicCoset::new(self.log_n_instances).coset;
        let eval_domain = CanonicCoset::new(self.max_constraint_log_degree_bound()).circle_domain();
        let mut denoms = vec![];
        for point in eval_domain.iter() {
            denoms.push(coset_vanishing(zero_domain, point));
        }
        bit_reverse(&mut denoms);
        let mut denom_inverses =
            vec![BaseField::zero(); 1 << (self.max_constraint_log_degree_bound())];
        BaseField::batch_inverse(&denoms, &mut denom_inverses);
    }
}

fn accumulation_code() -> rust::Tokens {
    quote! {
        for (i, (num, denom)) in numerators.iter().zip(denom_inverses.iter()).enumerate() {
            accum.accumulate(i, *num * *denom);
        }
    }
}

fn parse_cpu_prover_constraint(expr: &ProcessedAirVar) -> String {
    match expr {
        ProcessedAirVar::Const(ty, val) => {
            if ty == "Felt" {
                return format!("BaseField::from_u32_unchecked({})", val);
            }
            format!("{ty}::from({val})")
        }
        ProcessedAirVar::State(index) => {
            format!("trace_evals[{index}].values.at(i)")
        }
        ProcessedAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_cpu_prover_constraint(arg));
            }
            format!("{}({})", id, arg_str)
        }
        ProcessedAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_cpu_prover_constraint(arg));
            }
            format!("{}.{}({})", parse_cpu_prover_constraint(id), func, arg_str)
        }
        ProcessedAirVar::Var(_, id) => id.to_string(),
        ProcessedAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {op} {})",
                parse_cpu_prover_constraint(lhs),
                parse_cpu_prover_constraint(rhs)
            )
        }
        ProcessedAirVar::UnaryOp(op, expr) => {
            format!("({op}{})", parse_cpu_prover_constraint(expr))
        }
        ProcessedAirVar::Tuple(_) => unimplemented!(),
        ProcessedAirVar::Array(_) => unimplemented!(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use air_infra::core::air_fn_registry::AirFnRegistry;
    use air_infra::fibonacci::fib::Fib;

    use crate::code_gen::cpu_prover_gen::generate_cpu_prover_component;
    use crate::code_gen::utils::{project_root, reformat_rust_code};

    #[test]
    fn test_generate_cpu_prover_component() {
        let air_fn = Fib { claim_index: 1000 };
        let (_, air_fn, lists) = AirFnRegistry::new(&air_fn);

        let tokens = generate_cpu_prover_component(&air_fn.name, &lists.constraints);
        let text = reformat_rust_code(tokens.to_string().expect("Could not format Rust code."));

        let mut path = project_root();
        path.push("src/airs/examples/fibonacci/cpu_prover.rs");
        fs::write(path, text).unwrap();
    }
}
