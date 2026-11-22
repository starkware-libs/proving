use std::array::from_fn;

use air_infra::const_expr;
use air_infra::core::air_fn::AirBuilder;
use air_infra::core::expressions::felt_expr::FeltExpr;

/// Computes the 7 quadratic bilinear products of the certified minimum-rank (`Rank-7`)
/// decomposition of `QM31` multiplication over `M31`.
///
/// Using the unimodular basis isomorphism
/// `(a0 + a1*i) + (b0 + b1*i)*u <-> (a0 - 2*a1) + (b0 - 2*b1)*u + a1*u^2 + b1*u^3`
/// modulo `u^4 - 4*u^2 + 5 = 0` (`i = u^2 - 2`), we evaluate both degree-3 polynomials at
/// `{0, 1, -1, 2, -2, 3, inf}` and multiply pointwise (7 `FeltExpr * FeltExpr` multiplications).
pub fn qm31_mul_bilinear_products(op0: &[FeltExpr; 4], op1: &[FeltExpr; 4]) -> [FeltExpr; 7] {
    let eval_points = |x: &[FeltExpr; 4]| -> [FeltExpr; 7] {
        let sa = x[0].clone() - x[1].clone();
        let sb = x[2].clone() - x[3].clone();
        let l0 = sa.clone() - x[1].clone();
        let l1 = sa.clone() + sb.clone();
        let lm1 = sa - sb;

        let ta = x[0].clone() + const_expr!(2) * x[1].clone();
        let tb = const_expr!(2) * x[2].clone() + const_expr!(4) * x[3].clone();
        let l2 = ta.clone() + tb.clone();
        let lm2 = ta - tb;

        let l3 = x[0].clone()
            + const_expr!(7) * x[1].clone()
            + const_expr!(3) * x[2].clone()
            + const_expr!(21) * x[3].clone();
        let linf = x[3].clone();

        [l0, l1, lm1, l2, lm2, l3, linf]
    };

    let f = eval_points(op0);
    let g = eval_points(op1);
    from_fn(|k| f[k].clone() * g[k].clone())
}

/// Reconstructs the 4 `(1, i, u, i*u)` coordinates of `op0 * op1` over `M31` from the 7
/// evaluation products `[v0, v1, vm1, v2, vm2, v3, vinf]` using only linear combinations and
/// constant scalar divisions (`0` quadratic multiplications).
pub fn qm31_mul_reconstruct(v: &[FeltExpr; 7]) -> [FeltExpr; 4] {
    let [v0, v1, vm1, v2, vm2, v3, vinf] = v;
    let s1 = v1.clone() + vm1.clone();
    let s2 = v2.clone() + vm2.clone();
    let vinf5 = const_expr!(5) * vinf.clone();

    let out0 = (s2.clone() + const_expr!(20) * s1 - const_expr!(18) * v0.clone()) / const_expr!(24)
        - vinf5.clone();
    let out1 = (s2 - const_expr!(2) * v0.clone()) / const_expr!(8) - vinf5;
    let out2 = (const_expr!(30) * v0.clone() + const_expr!(10) * v1.clone()
        - const_expr!(55) * vm1.clone()
        + const_expr!(25) * v2.clone()
        - const_expr!(7) * vm2.clone()
        - const_expr!(3) * v3.clone())
        / const_expr!(120)
        + const_expr!(9) * vinf.clone();
    let out3 = (const_expr!(10) * v0.clone() - const_expr!(30) * v1.clone()
        + const_expr!(15) * (vm1.clone() + v2.clone())
        - const_expr!(9) * vm2.clone()
        - v3.clone())
        / const_expr!(120)
        + const_expr!(3) * vinf.clone();

    [out0, out1, out2, out3]
}

/// Symbolic `QM31` multiplication over `M31` using the 7-multiplication `Rank-7` decomposition.
///
/// Every output coordinate has degree 2 in `(op0, op1)` and is identically equal as a polynomial
/// in `F_{2^31 - 1}[op0, op1]` to the 20-multiplication schoolbook expansion.
pub fn qm31_mul_expr(op0: &[FeltExpr; 4], op1: &[FeltExpr; 4]) -> [FeltExpr; 4] {
    let products = qm31_mul_bilinear_products(op0, op1);
    qm31_mul_reconstruct(&products)
}

/// Symbolic `QM31` multiplication inside an [`AirBuilder`], binding the 7 quadratic bilinear
/// products via [`AirBuilder::let_for_constraint`] so each quadratic product is emitted once
/// across all 4 output constraints (`20 -> 7` quadratic multiplications, `-65.0%`).
pub fn qm31_mul_expr_in_air(
    ab: &mut AirBuilder,
    op0: &[FeltExpr; 4],
    op1: &[FeltExpr; 4],
) -> [FeltExpr; 4] {
    const NAMES: [&str; 7] = [
        "qm31_mul_v0",
        "qm31_mul_v1",
        "qm31_mul_vm1",
        "qm31_mul_v2",
        "qm31_mul_vm2",
        "qm31_mul_v3",
        "qm31_mul_vinf",
    ];
    let products = qm31_mul_bilinear_products(op0, op1);
    let shared = from_fn(|k| ab.let_for_constraint(products[k].clone(), NAMES[k]));
    qm31_mul_reconstruct(&shared)
}
