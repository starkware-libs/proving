use std::array::from_fn;
use std::cmp::{max, min};

use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bounded_felt::BoundedFeltExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

/// An AirFn implementation of the Karatsuba convolution algorithm.
/// Given two arrays of FeltExprs of length 4*N, this function computes their convolution
/// using the Karatsuba algorithm twice, meaning that the inner convolutions of length 2*N
/// are computed using SingleKaratsuba.
#[derive(Clone, Debug, Serialize)]
pub struct DoubleKaratsuba<const N: usize> {
    n: usize,
    limb_max_bound: i32,
    limb_min_bound: i32,
}

impl<const N: usize> DoubleKaratsuba<N> {
    pub fn new(limb_max_bound: i32, limb_min_bound: i32) -> Self {
        Self { n: N, limb_max_bound, limb_min_bound }
    }
}

macro_rules! impl_double_karatsuba {
    ($n:literal) => {
        impl AirFn for DoubleKaratsuba<$n> {
            type ExtIn = ();
            type In = [[FeltExpr; 4 * $n]; 2];
            type Out = [BoundedFeltExpr; 8 * $n - 1];

            fn call(&self, air_builder: &mut AirBuilder, _: (), [x, y]: Self::In) -> Self::Out {
                // Split the input arrays x, y into halves x0, x1, y0, y1
                let x0: [FeltExpr; 2 * $n] = from_fn(|i| x[i].clone());
                let x1: [FeltExpr; 2 * $n] = from_fn(|i| x[i + 2 * $n].clone());
                let y0: [FeltExpr; 2 * $n] = from_fn(|i| y[i].clone());
                let y1: [FeltExpr; 2 * $n] = from_fn(|i| y[i + 2 * $n].clone());

                let single_karatsuba = SingleKaratsuba::<$n>::new();

                // Compute the convolutions z0 = x0 * y0 and z2 = x1 * y1
                let z0 = air_builder.call(&single_karatsuba, [x0.clone(), y0.clone()]);
                let z2 = air_builder.call(&single_karatsuba, [x1.clone(), y1.clone()]);

                // Compute the pointwise additions x0 + x1 and y0 + y1 and save them to
                // intermediates
                let mut x_sum: [FeltExpr; 2 * $n] = from_fn(|i| x0[i].clone() + x1[i].clone());
                x_sum = air_builder.let_(x_sum, "x_sum");
                let mut y_sum: [FeltExpr; 2 * $n] = from_fn(|i| y0[i].clone() + y1[i].clone());
                y_sum = air_builder.let_(y_sum, "y_sum");

                // Compute the convolution z3 = (x0 + x1) * (y0 + y1)
                let z3 = air_builder.call(&single_karatsuba, [x_sum, y_sum]);

                let result_exprs: [FeltExpr; 8 * $n - 1] = karatsuba_finish(&z0, &z2, &z3);

                from_fn(|i| {
                    let convolution_start = max(i, 4 * $n - 1) - (4 * $n - 1);
                    let convolution_end = min(i, 4 * $n - 1);
                    let convolution_length = (convolution_end - convolution_start + 1) as i32;
                    let curr_max_bound = convolution_length * self.limb_max_bound;
                    let curr_min_bound = convolution_length * self.limb_min_bound;
                    BoundedFeltExpr::new(result_exprs[i].clone(), curr_max_bound, curr_min_bound)
                })
            }
        }
    };
}

impl_double_karatsuba!(7);
impl_double_karatsuba!(8);

/// An AirFn implementation of the Karatsuba convolution algorithm.
/// Given two arrays of FeltExprs of length 2*N, this function computes their convolution
/// by applying the Karatsuba algorithm once, meaning that the inner convolutions of length N
/// are computed using simple_convolution.
#[derive(Clone, Debug, Serialize)]
pub struct SingleKaratsuba<const N: usize> {
    n: usize,
}

impl<const N: usize> SingleKaratsuba<N> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self { n: N }
    }
}

macro_rules! impl_single_karatsuba {
    ($n:literal) => {
        impl AirFn for SingleKaratsuba<$n> {
            type ExtIn = ();
            type In = [[FeltExpr; 2 * $n]; 2];
            type Out = [FeltExpr; 4 * $n - 1];

            fn call(&self, air_builder: &mut AirBuilder, _: (), [x, y]: Self::In) -> Self::Out {
                // Split the input arrays x, y into halves x0, x1, y0, y1
                let x0: [FeltExpr; $n] = from_fn(|i| x[i].clone());
                let x1: [FeltExpr; $n] = from_fn(|i| x[i + $n].clone());
                let y0: [FeltExpr; $n] = from_fn(|i| y[i].clone());
                let y1: [FeltExpr; $n] = from_fn(|i| y[i + $n].clone());

                // Compute the convolutions z0 = x0 * y0 and z2 = x1 * y1
                let z0: [FeltExpr; 2 * $n - 1] =
                    air_builder.let_(simple_convolution(&x0, &y0), "z0");
                let z2: [FeltExpr; 2 * $n - 1] =
                    air_builder.let_(simple_convolution(&x1, &y1), "z2");

                // Compute the pointwise additions x0 + x1 and y0 + y1 and save them to
                // intermediates
                let mut x_sum: [FeltExpr; $n] = from_fn(|i| x0[i].clone() + x1[i].clone());
                x_sum = air_builder.let_(x_sum, "x_sum");
                let mut y_sum: [FeltExpr; $n] = from_fn(|i| y0[i].clone() + y1[i].clone());
                y_sum = air_builder.let_(y_sum, "y_sum");

                // Compute the convolution z3 = (x0 + x1) * (y0 + y1)
                let z3: [FeltExpr; 2 * $n - 1] = simple_convolution(&x_sum, &y_sum);

                karatsuba_finish(&z0, &z2, &z3)
            }
        }
    };
}

impl_single_karatsuba!(7);
impl_single_karatsuba!(8);

/// Finishes the Karatsuba convolution by combining the results of the three convolutions.
/// Given x0, x1, y0, y1 FeltExpr array of the same length k, Karatsuba's algorithm computes
/// the convolution (x0, x1) * (y0, y1) by first computing z0 = x0 * y0, z2 = x1 * y1, and
/// z3 = (x0 + x1) * (y0 + y1).
/// This function finishes the algorithm by taking z0, z2, z3 (all of length (M-1)/2=2k-1) then
/// combining z0, z1, z2 into a single array of length M=4k-1 by computing
/// z0 + (z1 <<< k) + (z2 <<< 2k) where '<<<' is an array shift forward and z1 = z3 - z0 - z2.
fn karatsuba_finish<const M: usize>(
    z0: &[FeltExpr],
    z2: &[FeltExpr],
    z3: &[FeltExpr],
) -> [FeltExpr; M] {
    let n = z0.len();

    assert_eq!(z2.len(), n, "z0, z2, z3 should have the same length");
    assert_eq!(z3.len(), n, "z0, z2, z3 should have the same length");
    assert_eq!(n % 2, 1, "length of z0, z2, z3 should be odd");
    assert_eq!(2 * n + 1, M, "length of z0, z2, z3 should be (M-1)/2");
    let ceil_half_len = n.div_ceil(2);

    let mut res = vec![];

    // Add z0
    res.extend_from_slice(z0);
    res.push(const_expr!(0));

    // Add z2 shifted by (M+1)/2 = 2k
    res.extend_from_slice(z2);

    // Add z1 shifted by ceil_half_len = k
    for i in 0..n {
        res[i + ceil_half_len] =
            res[i + ceil_half_len].clone() + (z3[i].clone() - z0[i].clone() - z2[i].clone());
    }

    res.try_into().expect("res should have the correct length")
}

/// Computes the symbolic convolution of two FeltExpr arrays of length n=(M+1)/2.
/// The convolution is computed using the straightforward O(n^2) algorithm.
pub fn simple_convolution<const M: usize>(x: &[FeltExpr], y: &[FeltExpr]) -> [FeltExpr; M] {
    let n = x.len();

    assert_eq!(y.len(), n, "x and y should have the same length");
    assert_eq!(2 * n - 1, M, "length of x, y should be (M+1)/2");

    from_fn(|i| {
        let convolution_start = max(i, n - 1) - (n - 1);
        let convolution_end = min(i, n - 1);
        (convolution_start..=convolution_end)
            .map(|j| x[j].clone() * y[i - j].clone())
            .reduce(|acc, val| acc + val)
            .expect("convolution shouldn't be empty")
    })
}
