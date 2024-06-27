use air_infra::core::autogen_structs::{ConstraintOrIntermediate, ProcessedAirVar};
use genco::lang::rust;
use genco::quote;

use super::component_gen::get_component_constraints;
use super::generate_prover_component;

pub fn generate_simd_prover_component(
    component_name: &str,
    constraints: &[ConstraintOrIntermediate],
) -> rust::Tokens {
    generate_prover_component(
        component_name,
        "SimdBackend",
        imports_code(component_name),
        numerator_code(constraints),
        denominator_code(),
        accumulation_code(),
    )
}

fn imports_code(component_name: &str) -> rust::Tokens {
    quote! {
    use stwo_prover::core::air::accumulation::DomainEvaluationAccumulator;
    use stwo_prover::core::air::{Component, ComponentProver, ComponentTrace};
    use stwo_prover::core::backend::simd::column::{BaseFieldVec, SecureFieldVec};
    use stwo_prover::core::backend::simd::m31::PackedBaseField;
    use stwo_prover::core::backend::simd::qm31::PackedSecureField;
    use stwo_prover::core::backend::simd::SimdBackend;
    use stwo_prover::core::backend::{Column, ColumnOps};
    use stwo_prover::core::constraints::coset_vanishing;
    use stwo_prover::core::fields::m31::BaseField;
    use stwo_prover::core::fields::FieldOps;
    use stwo_prover::core::poly::circle::CanonicCoset;
    use stwo_prover::core::InteractionElements;

    use super::component::$(component_name);
    }
}

fn numerator_code(constraints: &[ConstraintOrIntermediate]) -> rust::Tokens {
    // TODO(ShaharS): accumulate each constraint according to its degree.
    let mut numerator_code = quote! {
        let trace_evals = &trace.evals[0];
        let mut numerators = SecureFieldVec::zeros(1 << (self.max_constraint_log_degree_bound()));
        let [accum] =
            evaluation_accumulator.columns(
                [(self.max_constraint_log_degree_bound(), self.n_constraints())],
            );
        let random_coeff_powers = &accum.random_coeff_powers;
    };

    let mut constraints_code = quote! {};
    let n_constraints = get_component_constraints(constraints);
    let mut constraint_offset = 0;
    for constraint in constraints.iter() {
        match constraint {
            ConstraintOrIntermediate::InInstanceConstraint(expr) => {
                constraints_code.extend(quote! {
                    let random_coeff = PackedSecureField::broadcast(random_coeff_powers[$(n_constraints - 1 - constraint_offset)]);
                    *numer += random_coeff * ($(parse_simd_prover_constraint(expr)));
                });
                constraint_offset += 1;
            }
            ConstraintOrIntermediate::Intermediate(var, expr) => {
                constraints_code.extend(quote! {
                    let $(var) = $(parse_simd_prover_constraint(expr));
                });
            }
            ConstraintOrIntermediate::LookupConstraint {
                fn_name: _,
                input_felts: _,
                output_felts: _,
            } => todo!(),
        }
    }
    numerator_code.extend(quote! {
        for (i, numer) in numerators.data.iter_mut().enumerate()
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
        let mut denoms =
            BaseFieldVec::from_iter(eval_domain.iter().map(|p| coset_vanishing(zero_domain, p)));
        <SimdBackend as ColumnOps<BaseField>>::bit_reverse_column(&mut denoms);
        let mut denom_inverses = BaseFieldVec::zeros(denoms.len());
        <SimdBackend as FieldOps<BaseField>>::batch_inverse(&denoms, &mut denom_inverses);
    }
}

fn accumulation_code() -> rust::Tokens {
    quote! {
        for (i, (num, denom)) in numerators
            .data
            .iter().
            zip(denom_inverses.data.iter())
            .enumerate()
        {
            unsafe{ accum.col.set_packed(i, *num * *denom) };
        }
    }
}

fn parse_simd_prover_constraint(expr: &ProcessedAirVar) -> String {
    match expr {
        ProcessedAirVar::Const(ty, val) => {
            if ty == "Felt" {
                return format!(
                    "PackedBaseField::broadcast(BaseField::from_u32_unchecked({}))",
                    val
                );
            }
            format!("{ty}::from({val})")
        }
        ProcessedAirVar::State(index) => {
            format!("trace_evals[{index}].data[i]")
        }
        ProcessedAirVar::StaticCall(id, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_simd_prover_constraint(arg));
            }
            format!("{}({})", id, arg_str)
        }
        ProcessedAirVar::MethodCall(id, func, args) => {
            let mut arg_str = String::new();
            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    arg_str.push_str(", ");
                }
                arg_str.push_str(&parse_simd_prover_constraint(arg));
            }
            format!("{}.{}({})", parse_simd_prover_constraint(id), func, arg_str)
        }
        ProcessedAirVar::Var(_, id) => id.to_string(),
        ProcessedAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {op} {})",
                parse_simd_prover_constraint(lhs),
                parse_simd_prover_constraint(rhs)
            )
        }
        ProcessedAirVar::UnaryOp(op, expr) => {
            format!("({op}{})", parse_simd_prover_constraint(expr))
        }
        ProcessedAirVar::Tuple(_) => unimplemented!(),
        ProcessedAirVar::Array(_) => unimplemented!(),
    }
}
