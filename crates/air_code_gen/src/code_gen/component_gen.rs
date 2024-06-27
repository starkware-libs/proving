use air_infra::core::autogen_structs::{
    AutogenLists, ConstraintOrIntermediate, ProcessedAirVar, TraceGenerationStep,
};
use genco::lang::rust;
use genco::quote;

fn get_component_columns(deductions: &[TraceGenerationStep]) -> usize {
    deductions
        .iter()
        .filter(|deduction| matches!(deduction, TraceGenerationStep::Deduction(_)))
        .count()
}

pub fn get_component_constraints(constraints: &[ConstraintOrIntermediate]) -> usize {
    constraints
        .iter()
        .filter(|deduction| {
            matches!(deduction, ConstraintOrIntermediate::InInstanceConstraint(_))
                || matches!(
                    deduction,
                    ConstraintOrIntermediate::LookupConstraint {
                        fn_name: _,
                        input_felts: _,
                        output_felts: _
                    }
                )
        })
        .count()
}

fn generate_struct_code(name: &str) -> rust::Tokens {
    quote! {
        #[allow(non_camel_case_types)]
        pub struct $(name) {
            pub log_n_instances: u32,
        }
    }
}

/// Given a `ProcessedAirVar` expression, generates the Rust code to evaluate it at a single point
/// using the mask items.
fn parse_constraint_air_var(expr: &ProcessedAirVar) -> String {
    match expr {
        // TODO(ShaharS), consider to assert that the const values are in the correct range.
        ProcessedAirVar::Const(ty, val) => {
            if ty == "Felt" {
                return format!("BaseField::from_u32_unchecked({})", val);
            }
            format!("{}::from({})", ty, val)
        }
        ProcessedAirVar::State(index) => {
            format!("mask[{}][0]", index)
        }
        ProcessedAirVar::BinaryOp(lhs, op, rhs) => {
            format!(
                "({} {} {})",
                parse_constraint_air_var(lhs),
                op,
                parse_constraint_air_var(rhs)
            )
        }
        ProcessedAirVar::UnaryOp(op, val) => {
            format!("({}{})", op, parse_constraint_air_var(val))
        }
        ProcessedAirVar::Var(_, id) => id.to_string(),
        _ => unimplemented!(),
    }
}

/// Generates code to evaluate the constraints at a given point.
fn constraint_eval_at_point_code(constraints: &[ConstraintOrIntermediate]) -> rust::Tokens {
    let mut constraints_code = rust::Tokens::new();
    for constraint in constraints.iter() {
        match constraint {
            ConstraintOrIntermediate::Intermediate(var, expr) => {
                constraints_code.extend(quote! {
                    let $(var) = $(parse_constraint_air_var(expr));
                });
            }
            ConstraintOrIntermediate::InInstanceConstraint(expr) => {
                constraints_code.extend(quote! {
                    let numerator = $(parse_constraint_air_var(expr));
                    evaluation_accumulator.accumulate(numerator * denominator_inv);
                });
            }
            ConstraintOrIntermediate::LookupConstraint {
                fn_name: _,
                input_felts: _,
                output_felts: _,
            } => todo!(),
        }
    }
    constraints_code
}

fn generate_component_impl(
    name: &str,
    n_columns: usize,
    constraints: &[ConstraintOrIntermediate],
) -> rust::Tokens {
    let mut func1 = rust::Tokens::new();
    func1.extend(quote! {
        fn n_constraints(&self) -> usize {
            $(get_component_constraints(constraints))
        }
    });

    // Assumes that the maximal constraint degree is exactly 2.
    let mut func2 = rust::Tokens::new();
    func2.extend(quote! {
        fn max_constraint_log_degree_bound(&self) -> u32 {
            self.log_n_instances + 1
        }
    });

    // Assumes that the component trace is rectangular.
    // TODO(Ohad): add interaction log degree bounds.
    let mut func3 = rust::Tokens::new();
    func3.extend(quote! {
        fn trace_log_degree_bounds(&self) -> TreeVec<Vec<u32>> {
            TreeVec(vec![vec![self.log_n_instances; $(n_columns)],vec![]])
        }
    });

    // Assumes that each of the constraints applies on a single row.
    // TODO(Ohad): add interaction mask points.
    let mut func4 = rust::Tokens::new();
    func4.extend(quote! {
        fn mask_points(
            &self,
            point: CirclePoint<SecureField>,
        ) -> TreeVec<ColumnVec<Vec<CirclePoint<SecureField>>>> {
            TreeVec(vec![fixed_mask_points(&vec![vec![0_usize]; $(n_columns)], point), vec![]])
        }
    });

    let mut func5 = rust::Tokens::new();
    func5.extend(quote! {
        fn evaluate_constraint_quotients_at_point(
            &self,
            point: CirclePoint<SecureField>,
            mask: &ColumnVec<Vec<SecureField>>,
            evaluation_accumulator: &mut PointEvaluationAccumulator,
            _interaction_elements: &InteractionElements,
        ) {
        let constraint_zero_domain = CanonicCoset::new(self.log_n_instances).coset;
        let denominator_inv = coset_vanishing(constraint_zero_domain, point).inverse();
        $(constraint_eval_at_point_code(constraints))
        }
    });

    // TODO(Ohad): implement.
    let mut func6 = rust::Tokens::new();
    func6.extend(quote! {
        fn n_interaction_phases(&self) -> u32 {
            1
        }

        fn interaction_element_ids(&self) -> Vec<String> {
            vec![]
        }
    });

    let mut res_code = rust::Tokens::new();
    res_code.extend(quote! {
        impl Component for $(name) {
            $(func1)
            $['\n']
            $(func2)
            $['\n']
            $(func3)
            $['\n']
            $(func4)
            $['\n']
            #[allow(unused_parens)]
            $(func5)
            $['\n']
            $(func6)
        }
    });
    res_code
}

pub fn generate_component(component_name: &str, lists: AutogenLists) -> rust::Tokens {
    let imports = quote! {
        use stwo_prover::core::air::accumulation::PointEvaluationAccumulator;
        use stwo_prover::core::air::mask::fixed_mask_points;
        use stwo_prover::core::air::Component;
        use stwo_prover::core::circle::CirclePoint;
        use stwo_prover::core::constraints::coset_vanishing;
        use stwo_prover::core::fields::FieldExpOps;
        use stwo_prover::core::fields::m31::BaseField;
        use stwo_prover::core::fields::qm31::SecureField;
        use stwo_prover::core::pcs::TreeVec;
        use stwo_prover::core::poly::circle::CanonicCoset;
        use stwo_prover::core::{ColumnVec, InteractionElements};
    };
    let n_columns = get_component_columns(&lists.deductions);
    let struct_code = generate_struct_code(component_name);
    let component_impl_code =
        generate_component_impl(component_name, n_columns, &lists.constraints);

    quote! {
        $(imports)
        $['\n']
        $(struct_code)
        $['\n']
        $(component_impl_code)
    }
}
