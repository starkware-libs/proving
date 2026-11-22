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
    qm31_mul_bilinear_products, qm31_mul_expr, qm31_mul_expr_in_air, qm31_mul_reconstruct,
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

fn make_state_vars(prefix: &str, start_col: usize) -> [FeltExpr; 4] {
    from_fn(|i| {
        let mut f = FeltExpr::new(format!("{prefix}_{i}"), Some(1));
        f.set_value(ValueInfo::StateIndex(start_col + i, Some(format!("{prefix}_{i}"))));
        f
    })
}

fn eval_qm31_expr(a: [u32; 4], b: [u32; 4]) -> [M31; 4] {
    let op0 = a.map(|v| const_expr!(v));
    let op1 = b.map(|v| const_expr!(v));
    let out = qm31_mul_expr(&op0, &op1);
    out.map(|e| M31::from(e.calc().parse::<u32>().unwrap()))
}

#[derive(Clone, Debug, Serialize)]
struct PureQm31MulGate {
    use_rank7: bool,
}

impl AirFn for PureQm31MulGate {
    type ExtIn = ();
    type In = [[FeltExpr; 4]; 3];
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), [op0, op1, dst]: Self::In) -> Self::Out {
        let prod = if self.use_rank7 {
            qm31_mul_expr_in_air(ab, &op0, &op1)
        } else {
            schoolbook_qm31_mul_expr(&op0, &op1)
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

    // Zero checks + all 16 basis pairs.
    assert_eq!(eval_qm31_expr([0; 4], [0; 4]), [M31::from(0); 4]);
    for &b0 in &basis {
        assert_eq!(eval_qm31_expr([0; 4], b0), [M31::from(0); 4]);
        assert_eq!(eval_qm31_expr(b0, [0; 4]), [M31::from(0); 4]);
        for &b1 in &basis {
            let expected = (QM31::from_u32_unchecked(b0[0], b0[1], b0[2], b0[3])
                * QM31::from_u32_unchecked(b1[0], b1[1], b1[2], b1[3]))
            .to_m31_array();
            assert_eq!(eval_qm31_expr(b0, b1), expected);
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
        let v = [edges[3 - 1], u[2], u[0], u[1]];
        let expected = (QM31::from_u32_unchecked(u[0], u[1], u[2], u[3])
            * QM31::from_u32_unchecked(v[0], v[1], v[2], v[3]))
        .to_m31_array();
        assert_eq!(eval_qm31_expr(u, v), expected);
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
        assert_eq!(eval_qm31_expr(a, b), expected);
    }
}

#[test]
fn test_qm31_mul_expr_exact_mul_node_counts_and_degree() {
    let op0 = make_state_vars("op0", 0);
    let op1 = make_state_vars("op1", 4);

    // Baseline schoolbook: 20 quadratic FeltExpr * FeltExpr multiplications.
    let schoolbook = schoolbook_qm31_mul_expr(&op0, &op1);
    let schoolbook_compiled = schoolbook.map(|e| e.compile(CompileFor::Constraints));
    let schoolbook_muls: usize = schoolbook_compiled.iter().map(count_nonconst_muls_tree).sum();
    assert_eq!(schoolbook_muls, 20);

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

    // Compiled AirFn comparison: 20 -> 7 quadratic multiplications (-65.0%).
    let (reg_old, _) = AirFnRegistry::new(&PureQm31MulGate { use_rank7: false });
    let compiled_old = reg_old.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_old[0]), 20);

    let (reg_new, _) = AirFnRegistry::new(&PureQm31MulGate { use_rank7: true });
    let compiled_new = reg_new.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_new[0]), 7);

    // Qm31Ops full gate: 40 -> 27 non-const muls (7 Rank-7 muls + 4 * mul_flag + 4 * add_flag
    // + 4 * sub_flag + 4 pointwise muls + 4 * pointwise_mul_flag = 27, down from 40).
    let (reg_gate, _) = AirFnRegistry::new(&Qm31Ops {});
    let compiled_gate = reg_gate.compile();
    assert_eq!(count_compiled_air_fn_nonconst_muls(&compiled_gate["qm_31_ops"]), 27);
}
