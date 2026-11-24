use std::array::from_fn;
use std::collections::HashSet;

use air_common::TraceType;
use air_compile::compiled_structs::{CompiledAirFn, CompiledAirVar, ConstraintEvalStep};
use air_infra::const_expr;
use air_infra::core::air_body::CompileFor;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::{FeltExpr, ValueInfo};
use air_infra::core::variables::{AirVar, AirVarImplInfo, AsProverType};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{M31, PRIME, QM31};

use super::qm31_mul::{
    qm31_mul_bilinear_products, qm31_mul_expr, qm31_mul_expr_in_air, qm31_mul_rank7_in_air,
    qm31_mul_reconstruct,
};
use crate::circuit::qm_31_ops::Qm31Ops;

fn schoolbook_qm31_mul_expr(op0: &[FeltExpr; 4], op1: &[FeltExpr; 4]) -> [FeltExpr; 4] {
    [
        op0[0].clone() * op1[0].clone() - op0[1].clone() * op1[1].clone()
            + const_expr!(2) * (op0[2].clone() * op1[2].clone() - op0[3].clone() * op1[3].clone())
            - op0[2].clone() * op1[3].clone()
            - op0[3].clone() * op1[2].clone(),
        op0[0].clone() * op1[1].clone()
            + op0[1].clone() * op1[0].clone()
            + const_expr!(2) * (op0[2].clone() * op1[3].clone() + op0[3].clone() * op1[2].clone())
            + op0[2].clone() * op1[2].clone()
            - op0[3].clone() * op1[3].clone(),
        op0[0].clone() * op1[2].clone() - op0[1].clone() * op1[3].clone()
            + op0[2].clone() * op1[0].clone()
            - op0[3].clone() * op1[1].clone(),
        op0[0].clone() * op1[3].clone()
            + op0[1].clone() * op1[2].clone()
            + op0[2].clone() * op1[1].clone()
            + op0[3].clone() * op1[0].clone(),
    ]
}

fn is_const_air_var(v: &CompiledAirVar) -> bool {
    match v {
        CompiledAirVar::Const(..) | CompiledAirVar::NamedConst { .. } => true,
        CompiledAirVar::UnaryOp(_, inner) => is_const_air_var(inner),
        CompiledAirVar::BinaryOp(lhs, _, rhs) => is_const_air_var(lhs) && is_const_air_var(rhs),
        _ => false,
    }
}

fn count_nonconst_muls_tree(v: &CompiledAirVar) -> usize {
    match v {
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            let self_mul =
                usize::from(op == "*" && !is_const_air_var(lhs) && !is_const_air_var(rhs));
            self_mul + count_nonconst_muls_tree(lhs) + count_nonconst_muls_tree(rhs)
        }
        CompiledAirVar::UnaryOp(_, inner) => count_nonconst_muls_tree(inner),
        CompiledAirVar::Tuple(xs) | CompiledAirVar::Array(xs) => {
            xs.iter().map(count_nonconst_muls_tree).sum()
        }
        CompiledAirVar::Struct { fields, .. } => {
            fields.iter().map(|(_, x)| count_nonconst_muls_tree(x)).sum()
        }
        CompiledAirVar::StaticCall(_, args) => args.iter().map(count_nonconst_muls_tree).sum(),
        CompiledAirVar::MethodCall(recv, _, args) => {
            count_nonconst_muls_tree(recv)
                + args.iter().map(count_nonconst_muls_tree).sum::<usize>()
        }
        _ => 0,
    }
}

fn count_all_muls_tree(v: &CompiledAirVar) -> usize {
    match v {
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            let self_mul = usize::from(op == "*");
            self_mul + count_all_muls_tree(lhs) + count_all_muls_tree(rhs)
        }
        CompiledAirVar::UnaryOp(_, inner) => count_all_muls_tree(inner),
        CompiledAirVar::Tuple(xs) | CompiledAirVar::Array(xs) => {
            xs.iter().map(count_all_muls_tree).sum()
        }
        CompiledAirVar::Struct { fields, .. } => {
            fields.iter().map(|(_, x)| count_all_muls_tree(x)).sum()
        }
        CompiledAirVar::StaticCall(_, args) => args.iter().map(count_all_muls_tree).sum(),
        CompiledAirVar::MethodCall(recv, _, args) => {
            count_all_muls_tree(recv) + args.iter().map(count_all_muls_tree).sum::<usize>()
        }
        _ => 0,
    }
}

fn count_add_subs_tree(v: &CompiledAirVar) -> usize {
    match v {
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            let self_add = usize::from(op == "+" || op == "-");
            self_add + count_add_subs_tree(lhs) + count_add_subs_tree(rhs)
        }
        CompiledAirVar::UnaryOp(_, inner) => count_add_subs_tree(inner),
        _ => 0,
    }
}

fn collect_unique_nonconst_muls<'a>(v: &'a CompiledAirVar, set: &mut HashSet<&'a CompiledAirVar>) {
    match v {
        CompiledAirVar::BinaryOp(lhs, op, rhs) => {
            if op == "*" && !is_const_air_var(lhs) && !is_const_air_var(rhs) {
                set.insert(v);
            }
            collect_unique_nonconst_muls(lhs, set);
            collect_unique_nonconst_muls(rhs, set);
        }
        CompiledAirVar::UnaryOp(_, inner) => collect_unique_nonconst_muls(inner, set),
        _ => {}
    }
}

fn count_compiled_air_fn_nonconst_muls(air_fn: &CompiledAirFn) -> usize {
    air_fn
        .constraints
        .iter()
        .map(|step| match step {
            ConstraintEvalStep::Constraint(expr, _) => count_nonconst_muls_tree(expr),
            ConstraintEvalStep::Intermediate(interm) => count_nonconst_muls_tree(&interm.var),
            ConstraintEvalStep::LookupTerm(_) => 0,
        })
        .sum()
}

fn count_compiled_air_fn_all_muls(air_fn: &CompiledAirFn) -> usize {
    air_fn
        .constraints
        .iter()
        .map(|step| match step {
            ConstraintEvalStep::Constraint(expr, _) => count_all_muls_tree(expr),
            ConstraintEvalStep::Intermediate(interm) => count_all_muls_tree(&interm.var),
            ConstraintEvalStep::LookupTerm(_) => 0,
        })
        .sum()
}

fn count_compiled_air_fn_add_subs(air_fn: &CompiledAirFn) -> usize {
    air_fn
        .constraints
        .iter()
        .map(|step| match step {
            ConstraintEvalStep::Constraint(expr, _) => count_add_subs_tree(expr),
            ConstraintEvalStep::Intermediate(interm) => count_add_subs_tree(&interm.var),
            ConstraintEvalStep::LookupTerm(_) => 0,
        })
        .sum()
}

fn make_state_vars(prefix: &str, start_col: usize) -> [FeltExpr; 4] {
    from_fn(|i| {
        let mut f = FeltExpr::new(format!("{prefix}_{i}"), Some(1));
        f.set_value(ValueInfo::StateIndex(start_col + i, Some(format!("{prefix}_{i}"))));
        f
    })
}

fn eval_qm31_rank7_expr(a: [u32; 4], b: [u32; 4]) -> [M31; 4] {
    let op0 = a.map(|v| const_expr!(v));
    let op1 = b.map(|v| const_expr!(v));
    let out = qm31_mul_expr(&op0, &op1);
    out.map(|e| M31::from(e.calc().parse::<u32>().unwrap()))
}

fn assert_in_air_gate_satisfied(
    reg: &AirFnRegistry,
    gate: &PureQm31MulGate,
    a: [u32; 4],
    b: [u32; 4],
    expected: [M31; 4],
) {
    let op0 = a.map(|v| const_expr!(v));
    let op1 = b.map(|v| const_expr!(v));
    let dst = expected.map(|v| const_expr!(v.0));
    let _ = reg.run_air(gate, (), [op0, op1, dst]);
}

#[derive(Clone, Copy, Debug, Serialize)]
enum MulMode {
    Schoolbook,
    Rank7,
    Karatsuba9,
}

#[derive(Clone, Debug, Serialize)]
struct PureQm31MulGate {
    mode: MulMode,
}

impl AirFn for PureQm31MulGate {
    type ExtIn = ();
    type In = [[FeltExpr; 4]; 3];
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), [op0, op1, dst]: Self::In) -> Self::Out {
        let prod = match self.mode {
            MulMode::Schoolbook => schoolbook_qm31_mul_expr(&op0, &op1),
            MulMode::Rank7 => qm31_mul_rank7_in_air(ab, &op0, &op1),
            MulMode::Karatsuba9 => qm31_mul_expr_in_air(ab, &op0, &op1),
        };
        for i in 0..4 {
            ab.constrain(dst[i].clone() - prod[i].clone(), "");
        }
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some((0..12).map(|i| Some(format!("v_{i}"))).collect())
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }
}

#[test]
fn test_qm31_mul_expr_basis_edges_and_1000_random() {
    let basis: [[u32; 4]; 4] = [[1, 0, 0, 0], [0, 1, 0, 0], [0, 0, 1, 0], [0, 0, 0, 1]];
    let gate_k9 = PureQm31MulGate { mode: MulMode::Karatsuba9 };
    let (reg_k9, _) = AirFnRegistry::new(&gate_k9);

    // Zero checks + all 16 basis pairs across both Rank-7 and Karatsuba-9 AIR expressions.
    assert_eq!(eval_qm31_rank7_expr([0; 4], [0; 4]), [M31::from(0); 4]);
    assert_in_air_gate_satisfied(&reg_k9, &gate_k9, [0; 4], [0; 4], [M31::from(0); 4]);
    for &b0 in &basis {
        assert_eq!(eval_qm31_rank7_expr([0; 4], b0), [M31::from(0); 4]);
        assert_eq!(eval_qm31_rank7_expr(b0, [0; 4]), [M31::from(0); 4]);
        assert_in_air_gate_satisfied(&reg_k9, &gate_k9, [0; 4], b0, [M31::from(0); 4]);
        assert_in_air_gate_satisfied(&reg_k9, &gate_k9, b0, [0; 4], [M31::from(0); 4]);
        for &b1 in &basis {
            let expected = (QM31::from_u32_unchecked(b0[0], b0[1], b0[2], b0[3])
                * QM31::from_u32_unchecked(b1[0], b1[1], b1[2], b1[3]))
            .to_m31_array();
            assert_eq!(eval_qm31_rank7_expr(b0, b1), expected);
            assert_in_air_gate_satisfied(&reg_k9, &gate_k9, b0, b1, expected);
        }
    }

    // Boundary combinations in {0, 1, PRIME - 1}.
    let edges = [0, 1, PRIME - 1];
    let mut edge_vecs = Vec::with_capacity(81);
    for &x0 in &edges {
        for &x1 in &edges {
            for &x2 in &edges {
                for &x3 in &edges {
                    edge_vecs.push([x0, x1, x2, x3]);
                }
            }
        }
    }
    for &u in &edge_vecs {
        let v = [edges[2], u[2], u[0], u[1]];
        let expected = (QM31::from_u32_unchecked(u[0], u[1], u[2], u[3])
            * QM31::from_u32_unchecked(v[0], v[1], v[2], v[3]))
        .to_m31_array();
        assert_eq!(eval_qm31_rank7_expr(u, v), expected);
        assert_in_air_gate_satisfied(&reg_k9, &gate_k9, u, v, expected);
    }

    // 1,000 deterministic pseudorandom QM31 pairs (SplitMix64).
    let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next_m31 = || -> u32 {
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^= z >> 31;
        (z as u32) % PRIME
    };

    for _ in 0..1_000 {
        let a = [next_m31(), next_m31(), next_m31(), next_m31()];
        let b = [next_m31(), next_m31(), next_m31(), next_m31()];
        let expected = (QM31::from_u32_unchecked(a[0], a[1], a[2], a[3])
            * QM31::from_u32_unchecked(b[0], b[1], b[2], b[3]))
        .to_m31_array();
        assert_eq!(eval_qm31_rank7_expr(a, b), expected);
        assert_in_air_gate_satisfied(&reg_k9, &gate_k9, a, b, expected);
    }
}

fn compiled_constraint_degrees(air_fn: &CompiledAirFn) -> Vec<usize> {
    use std::collections::HashMap;
    fn deg(v: &CompiledAirVar, interms: &HashMap<String, usize>) -> usize {
        match v {
            CompiledAirVar::Const(..) | CompiledAirVar::NamedConst { .. } => 0,
            CompiledAirVar::State(_)
            | CompiledAirVar::ExternalState(_)
            | CompiledAirVar::Enabler => 1,
            CompiledAirVar::Var(_, name) => *interms.get(name).expect("known intermediate"),
            CompiledAirVar::UnaryOp(_, x) => deg(x, interms),
            CompiledAirVar::BinaryOp(lhs, op, rhs) => {
                if op == "*" {
                    deg(lhs, interms) + deg(rhs, interms)
                } else {
                    deg(lhs, interms).max(deg(rhs, interms))
                }
            }
            _ => panic!("unexpected var in degree check: {v:?}"),
        }
    }
    let mut interms = HashMap::new();
    let mut out = Vec::new();
    for step in &air_fn.constraints {
        match step {
            ConstraintEvalStep::Intermediate(interm) => {
                let d = deg(&interm.var, &interms);
                for name in &interm.felt_names {
                    interms.insert(name.clone(), d);
                }
            }
            ConstraintEvalStep::Constraint(expr, _) => out.push(deg(expr, &interms)),
            ConstraintEvalStep::LookupTerm(_) => {}
        }
    }
    out
}

#[test]
fn test_qm31_mul_expr_exact_mul_node_counts_and_degree() {
    let op0 = make_state_vars("op0", 0);
    let op1 = make_state_vars("op1", 4);

    // Baseline schoolbook: 20 quadratic muls + 2 constant muls = 22 total muls.
    let schoolbook = schoolbook_qm31_mul_expr(&op0, &op1);
    let schoolbook_compiled = schoolbook.map(|e| e.compile(CompileFor::Constraints));
    let schoolbook_muls: usize = schoolbook_compiled.iter().map(count_nonconst_muls_tree).sum();
    let schoolbook_all_muls: usize = schoolbook_compiled.iter().map(count_all_muls_tree).sum();
    assert_eq!(schoolbook_muls, 20);
    assert_eq!(schoolbook_all_muls, 22);

    // Rank-7 decomposition stages: 7 quadratic muls in products, 0 in reconstruction.
    let products = qm31_mul_bilinear_products(&op0, &op1);
    for p in &products {
        assert_eq!(p.deg_in_state(), Some(2));
        assert_eq!(count_nonconst_muls_tree(&p.clone().compile(CompileFor::Constraints)), 1);
    }
    let product_vars = make_state_vars("v_lo", 8);
    let product_vars_hi = make_state_vars("v_hi", 12);
    let dummy_7: [FeltExpr; 7] = [
        product_vars[0].clone(),
        product_vars[1].clone(),
        product_vars[2].clone(),
        product_vars[3].clone(),
        product_vars_hi[0].clone(),
        product_vars_hi[1].clone(),
        product_vars_hi[2].clone(),
    ];
    let recon = qm31_mul_reconstruct(&dummy_7);
    let recon_muls: usize = recon
        .iter()
        .map(|e| count_nonconst_muls_tree(&e.clone().compile(CompileFor::Constraints)))
        .sum();
    assert_eq!(recon_muls, 0);

    // Standalone qm31_mul_expr: degree 2 on every coordinate, and exactly 7 unique quadratic muls.
    let rank7_exprs = qm31_mul_expr(&op0, &op1);
    for out in &rank7_exprs {
        assert_eq!(out.deg_in_state(), Some(2));
    }
    let rank7_compiled = rank7_exprs.map(|e| e.compile(CompileFor::Constraints));
    let mut unique_muls = HashSet::new();
    for c in &rank7_compiled {
        collect_unique_nonconst_muls(c, &mut unique_muls);
    }
    assert_eq!(unique_muls.len(), 7);

    // Compiled AirFn comparison across all 3 modes:
    let (reg_old, _) = AirFnRegistry::new(&PureQm31MulGate { mode: MulMode::Schoolbook });
    let compiled_old = reg_old.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_old[0]), 20);
    assert_eq!(count_compiled_air_fn_all_muls(&compiled_old[0]), 22);
    assert_eq!(count_compiled_air_fn_add_subs(&compiled_old[0]), 20);
    assert_eq!(compiled_constraint_degrees(&compiled_old[0]), vec![2; 4]);

    let (reg_r7, _) = AirFnRegistry::new(&PureQm31MulGate { mode: MulMode::Rank7 });
    let compiled_r7 = reg_r7.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_r7[0]), 7);
    assert_eq!(compiled_constraint_degrees(&compiled_r7[0]), vec![2; 4]);

    // Unimodular Karatsuba-9 (`qm31_mul_expr_in_air`): 9 quadratic muls and 0 constant muls.
    let (reg_k9, _) = AirFnRegistry::new(&PureQm31MulGate { mode: MulMode::Karatsuba9 });
    let compiled_k9 = reg_k9.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_k9[0]), 9);
    assert_eq!(count_compiled_air_fn_all_muls(&compiled_k9[0]), 9);
    // 27 mul add/subs + 4 `dst[i] - prod[i]` = 31 add/subs (40 total ops).
    assert_eq!(count_compiled_air_fn_add_subs(&compiled_k9[0]), 31);
    assert_eq!(compiled_constraint_degrees(&compiled_k9[0]), vec![2; 4]);

    // Qm31Ops full gate: 29 total muls (0 constant muls) and constraint degree 3 on all 4
    // constraints.
    let (reg_gate, _) = AirFnRegistry::new(&Qm31Ops {});
    let compiled_gate = reg_gate.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_gate["qm_31_ops"]), 29);
    assert_eq!(count_compiled_air_fn_all_muls(&compiled_gate["qm_31_ops"]), 29);
    assert_eq!(compiled_constraint_degrees(&compiled_gate["qm_31_ops"]), vec![3; 4]);
}
