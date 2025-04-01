use std::array::from_fn;
use std::cmp::{max, min};

use inst_def::InstDef;

use super::bounded_felt::*;
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

/// An AirFn implementation of the Karatsuba convolution algorithm.
/// Given two arrays of FeltExprs of length 4*N, this function computes their convolution
/// using the Karatsuba algorithm twice, meaning that the inner convolutions of length 2*N
/// are computed using SingleKaratsuba.
#[derive(Clone, Debug, InstDef)]
pub struct DoubleKaratsuba<const N: usize> {
    pub limb_max_bound: i32,
}
impl<const N: usize> AirFn for DoubleKaratsuba<N>
where
    [(); 4 * N]:,
    [(); 2 * { 2 * { 2 * N - 1 } + 1 } + 1]:,
{
    type ExtIn = ();
    type In = [[FeltExpr; 4 * N]; 2];
    type Out = [BoundedFeltExpr; 2 * { 2 * { 2 * N - 1 } + 1 } + 1];

    fn call(&self, air_builder: &mut AirBuilder, _: (), [x, y]: Self::In) -> Self::Out {
        // Split the input arrays x, y into halves x0, x1, y0, y1
        let x0 = from_fn(|i| x[i].clone());
        let x1 = from_fn(|i| x[i + 2 * N].clone());
        let y0 = from_fn(|i| y[i].clone());
        let y1 = from_fn(|i| y[i + 2 * N].clone());

        let single_karatsuba = SingleKaratsuba::<N> {};

        // Compute the convolutions z0 = x0 * y0 and z2 = x1 * y1
        let z0 = air_builder.call(&single_karatsuba, [x0.clone(), y0.clone()]);
        let z2 = air_builder.call(&single_karatsuba, [x1.clone(), y1.clone()]);

        // Compute the pointwise additions x0 + x1 and y0 + y1 and save them to intermediates
        let mut x_sum = from_fn(|i| x0[i].clone() + x1[i].clone());
        x_sum = air_builder.let_(x_sum, "x_sum");
        let mut y_sum = from_fn(|i| y0[i].clone() + y1[i].clone());
        y_sum = air_builder.let_(y_sum, "y_sum");

        // Compute the convolution z3 = (x0 + x1) * (y0 + y1)
        let z3 = air_builder.call(&single_karatsuba, [x_sum, y_sum]);

        let result_exprs = karatsuba_finish::<{ 2 * { 2 * N - 1 } + 1 }>(&z0, &z2, &z3);

        from_fn(|i| {
            let convolution_start = max(i, 4 * N - 1) - (4 * N - 1);
            let convolution_end = min(i, 4 * N - 1);
            let curr_max_bound = (convolution_end - convolution_start + 1) as i32
                * self.limb_max_bound
                * self.limb_max_bound;
            BoundedFeltExpr::new(result_exprs[i].clone(), curr_max_bound, 0)
        })
    }
}

/// An AirFn implementation of the Karatsuba convolution algorithm.
/// Given two arrays of FeltExprs of length 2*N, this function computes their convolution
/// by applying the Karatsuba algorithm once, meaning that the inner convolutions of length N
/// are computed using simple_convolution.
#[derive(Clone, Debug, InstDef)]
pub struct SingleKaratsuba<const N: usize> {}

impl<const N: usize> AirFn for SingleKaratsuba<N>
where
    [(); 2 * { 2 * N - 1 } + 1]:,
{
    type ExtIn = ();
    type In = [[FeltExpr; 2 * N]; 2];
    type Out = [FeltExpr; 2 * { 2 * N - 1 } + 1];

    fn call(&self, air_builder: &mut AirBuilder, _: (), [x, y]: Self::In) -> Self::Out {
        // Split the input arrays x, y into halves x0, x1, y0, y1
        let x0 = from_fn(|i| x[i].clone());
        let x1 = from_fn(|i| x[i + N].clone());
        let y0 = from_fn(|i| y[i].clone());
        let y1 = from_fn(|i| y[i + N].clone());

        // Compute the convolutions z0 = x0 * y0 and z2 = x1 * y1
        let z0 = air_builder.let_(simple_convolution(&x0, &y0), "z0");
        let z2 = air_builder.let_(simple_convolution(&x1, &y1), "z2");

        // Compute the pointwise additions x0 + x1 and y0 + y1 and save them to intermediates
        let mut x_sum = from_fn(|i| x0[i].clone() + x1[i].clone());
        x_sum = air_builder.let_(x_sum, "x_sum");
        let mut y_sum = from_fn(|i| y0[i].clone() + y1[i].clone());
        y_sum = air_builder.let_(y_sum, "y_sum");

        // Compute the convolution z3 = (x0 + x1) * (y0 + y1)
        let z3 = simple_convolution(&x_sum, &y_sum);

        karatsuba_finish::<{ 2 * N - 1 }>(&z0, &z2, &z3)
    }
}

/// Finishes the Karatsuba convolution by combining the results of the three convolutions.
/// Given x0, x1, y0, y1 FeltExpr array of the same length k, Karatsuba's algorithm computes
/// the convolution (x0, x1) * (y0, y1) by first computing z0 = x0 * y0, z2 = x1 * y1, and
/// z3 = (x0 + x1) * (y0 + y1).
/// This function finishes the algorithm by taking z0, z2, z3 (all of length 2k-1) then combining
/// z0, z1, z2 into a single array of length 4k-1 by computing z0 + (z1 <<< k) + (z2 <<< 2k) where
/// '<<<' is an array shift forward and z1 = z3 - z0 - z2.
fn karatsuba_finish<const N: usize>(
    z0: &[FeltExpr; N],
    z2: &[FeltExpr; N],
    z3: &[FeltExpr; N],
) -> [FeltExpr; 2 * N + 1] {
    let ceil_half_len = (N + 1) / 2;
    let mut res = vec![];

    // Add z0
    res.extend_from_slice(z0);
    res.push(const_expr!(0));

    // Add z2 shifted by N+1 = 2k
    res.extend_from_slice(z2);

    // Add z1 shifted by ceil_half_len = k
    for i in 0..N {
        res[i + ceil_half_len] =
            res[i + ceil_half_len].clone() + (z3[i].clone() - z0[i].clone() - z2[i].clone());
    }

    res.try_into().expect("res should have the correct length")
}

/// Computes the symbolic convolution of two FeltExpr arrays of length N.
/// The convolution is computed using the straightforward O(n^2) algorithm.
pub fn simple_convolution<const N: usize>(
    x: &[FeltExpr; N],
    y: &[FeltExpr; N],
) -> [FeltExpr; 2 * N - 1] {
    let result: [FeltExpr; 2 * N - 1] = from_fn(|i| {
        let convolution_start = max(i, N - 1) - (N - 1);
        let convolution_end = min(i, N - 1);
        (convolution_start..=convolution_end)
            .map(|j| x[j].clone() * y[i - j].clone())
            .reduce(|acc, val| acc + val)
            .expect("convolution shouldn't be empty")
    });

    result
}
