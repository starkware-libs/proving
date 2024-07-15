use air_infra::core::compiled_structs::{CompiledAirVar, ConstraintEvalStep};
use genco::lang::rust;
use genco::quote;

use super::component_gen::component_n_constraints;
use crate::code_gen::generate_prover_component;

pub fn generate_cpu_prover_component(
    component_name: &str,
    constraints: &[ConstraintEvalStep],
) -> rust::Tokens {
    generate_prover_component(
        component_name,
        "CpuBackend",
        imports_code(component_name),
        numerator_code(constraints),
        denominator_code(),
        accumulation_code(),
    )
}

fn imports_code(component_name: &str) -> rust::Tokens {
    quote! {
        #![allow(unused_imports)]
        use num_traits::identities::Zero;
        use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
        use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
        use stwo_prover::core::backend::{CpuBackend, Column};
        use stwo_prover::core::constraints::coset_vanishing;
        use stwo_prover::core::fields::m31::BaseField;
        use stwo_prover::core::fields::qm31::SecureField;
        use stwo_prover::core::fields::FieldExpOps;
        use stwo_prover::core::poly::circle::CanonicCoset;
        use stwo_prover::core::utils::bit_reverse;
        use stwo_prover::core::InteractionElements;

        use super::component::$(component_name);
    }
}

fn numerator_code(constraints: &[ConstraintEvalStep]) -> rust::Tokens {
    // TODO(ShaharS): accumulate each constraint according to its degree.
    let mut numerator_code = quote! {
        let trace_evals = &trace.evals[0];
        let mut numerators =
            vec![SecureField::zero(); 1 << (self.max_constraint_log_degree_bound())];
        let [mut accum] =
            evaluation_accumulator.columns(
                [(self.max_constraint_log_degree_bound(), self.n_constraints())],
            );
    };

    let mut constraints_code = quote! {};
    let n_constraints = component_n_constraints(constraints);
    let mut constraint_offset = 0;
    for constraint in constraints.iter() {
        match constraint {
            ConstraintEvalStep::InInstanceConstraint(expr) => {
                constraints_code.extend(quote! {
                    *numer +=
                        accum.random_coeff_powers[$(n_constraints - 1 - constraint_offset)] *
                        ($(parse_cpu_prover_constraint(expr)));
                });
                constraint_offset += 1;
            }
            ConstraintEvalStep::Intermediate(var, expr) => {
                constraints_code.extend(quote! {
                    let $(var) = $(parse_cpu_prover_constraint(expr));
                });
            }
            // TODO(Ohad): implement.
            ConstraintEvalStep::LookupConstraint {
                fn_name: _,
                input_felts: _,
                output_felts: _,
            } => (),
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

fn denominator_code() -> rust::Tokens {
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

fn parse_cpu_prover_constraint(expr: &CompiledAirVar) -> String {
    match expr {
        CompiledAirVar::Const(ty, val) => {
            if ty == "Felt" || ty == "M31" {
                return format!("BaseField::from_u32_unchecked({})", val);
            }
            format!("{ty}::from({val})")
        }
        CompiledAirVar::State(index) => {
            format!("trace_evals[{index}].values.at(i)")
        }
        CompiledAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_cpu_prover_constraint(arg));
            }
            format!("{}({})", id, arg_str)
        }
        CompiledAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_cpu_prover_constraint(arg));
            }
            format!("{}.{}({})", parse_cpu_prover_constraint(id), func, arg_str)
        }
        CompiledAirVar::Var(_, id) => id.to_string(),
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {op} {})",
                parse_cpu_prover_constraint(lhs),
                parse_cpu_prover_constraint(rhs)
            )
        }
        CompiledAirVar::UnaryOp(op, expr) => {
            format!("({op}{})", parse_cpu_prover_constraint(expr))
        }
        CompiledAirVar::Tuple(_) => unimplemented!(),
        CompiledAirVar::Array(_) => unimplemented!(),
    }
}
