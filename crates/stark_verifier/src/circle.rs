use circuits::context::{Context, Var};
use circuits::eval;
use circuits::ivalue::IValue;
use circuits::ops::inv;
use circuits::simd::Simd;
use circuits::wrappers::M31Wrapper;
use num_traits::CheckedSub;
use stwo::core::circle::{CirclePoint, M31_CIRCLE_GEN};
use stwo::core::fields::m31::M31;

#[cfg(test)]
#[path = "circle_test.rs"]
pub mod test;

/// Computes `pi(x) = 2 * x^2 - 1`, which is the x-coordinate of the point `(x, y) + (x, y)`.
pub fn double_x(context: &mut Context<impl IValue>, value: Var) -> Var {
    let value_sqr = eval!(context, (value) * (value));
    eval!(context, ((value_sqr) + (value_sqr)) - (1))
}

/// Same as [double_x], but for [Simd].
pub fn double_x_simd(context: &mut Context<impl IValue>, value: &Simd) -> Simd {
    let value_sqr = Simd::mul(context, value, value);
    let value_sqr_times2 = Simd::add(context, &value_sqr, &value_sqr);
    let one = Simd::one(context, value.len());
    Simd::sub(context, &value_sqr_times2, &one)
}

/// Computes `p + p`.
pub fn double_point(
    context: &mut Context<impl IValue>,
    p: &CirclePoint<M31Wrapper<Var>>,
) -> CirclePoint<M31Wrapper<Var>> {
    let xy = eval!(context, (*p.x.get()) * (*p.y.get()));
    let new_y = eval!(context, (xy) + (xy));
    CirclePoint {
        x: M31Wrapper::new_unsafe(double_x(context, *p.x.get())),
        y: M31Wrapper::new_unsafe(new_y),
    }
}

/// Same as [double_point], but for [Simd].
pub fn double_point_simd(
    context: &mut Context<impl IValue>,
    p: &CirclePoint<Simd>,
) -> CirclePoint<Simd> {
    let xy = Simd::mul(context, &p.x, &p.y);
    let new_y = Simd::add(context, &xy, &xy);
    CirclePoint { x: double_x_simd(context, &p.x), y: new_y }
}

/// Computes `2^n_doubles * p`.
pub fn repeated_double_point_simd(
    context: &mut Context<impl IValue>,
    p: &CirclePoint<Simd>,
    n_doubles: usize,
) -> CirclePoint<Simd> {
    let mut p = p.clone();
    for _ in 0..n_doubles {
        p = double_point_simd(context, &p);
    }
    p
}

/// Computes `point0 + point1` on the circle.
pub fn add_points(
    context: &mut Context<impl IValue>,
    point0: &CirclePoint<Var>,
    point1: &CirclePoint<Var>,
) -> CirclePoint<Var> {
    let x = eval!(context, ((point0.x) * (point1.x)) - ((point0.y) * (point1.y)));
    let y = eval!(context, ((point0.x) * (point1.y)) + ((point0.y) * (point1.x)));
    CirclePoint { x, y }
}

/// A version of [add_points] for [Simd].
pub fn add_points_simd(
    context: &mut Context<impl IValue>,
    point0: &CirclePoint<Simd>,
    point1: &CirclePoint<Simd>,
) -> CirclePoint<Simd> {
    let x0x1 = Simd::mul(context, &point0.x, &point1.x);
    let x0y1 = Simd::mul(context, &point0.x, &point1.y);
    let y0x1 = Simd::mul(context, &point0.y, &point1.x);
    let y0y1 = Simd::mul(context, &point0.y, &point1.y);
    let x = Simd::sub(context, &x0x1, &y0y1);
    let y = Simd::add(context, &x0y1, &y0x1);
    CirclePoint { x, y }
}

/// Computes the generator point of the subgroup of size `2^log_domain_size`.
pub fn generator_point(log_domain_size: usize) -> CirclePoint<M31> {
    M31_CIRCLE_GEN.repeated_double(31.checked_sub(&log_domain_size).unwrap() as u32)
}

/// Computes the generator point of the subgroup of size `2^log_domain_size`, repeated `size` times.
pub fn generator_point_simd(
    context: &mut Context<impl IValue>,
    log_domain_size: usize,
    size: usize,
) -> CirclePoint<Simd> {
    let pt = generator_point(log_domain_size);
    CirclePoint { x: Simd::repeat(context, pt.x, size), y: Simd::repeat(context, pt.y, size) }
}

/// Computes the group inverse of the generator point of the subgroup of size `2^log_domain_size`,
/// repeated `size` times.
pub fn minus_generator_point_simd(
    context: &mut Context<impl IValue>,
    log_domain_size: usize,
    size: usize,
) -> CirclePoint<Simd> {
    let pt = generator_point(log_domain_size);
    CirclePoint { x: Simd::repeat(context, pt.x, size), y: Simd::repeat(context, -pt.y, size) }
}

/// Computes the polynomial that vanishes on the canonical coset of size `2^log_trace_size`.
///
/// The polynomial is `pi^{log_trace_size - 1}(x) = pi(...(pi(x))...)`.
pub fn coset_vanishing_poly(
    context: &mut Context<impl IValue>,
    mut x: Var,
    log_trace_size: usize,
) -> Var {
    assert!(log_trace_size >= 1);

    for _ in 0..(log_trace_size - 1) {
        x = double_x(context, x);
    }
    x
}

/// Computes the inverse of the domain polynomial at `x`. See [coset_vanishing_poly].
///
/// The caller must ensure `x` is not on the trace coset.
pub fn denom_inverse(context: &mut Context<impl IValue>, x: Var, log_trace_size: usize) -> Var {
    let denom = coset_vanishing_poly(context, x, log_trace_size);
    inv(context, denom)
}

/// For each base point (i.e. for each SIMD lane in `base_points`), computes the points of the first
/// half of a coset of given log size starting from base point, in bit reversed order.
pub fn compute_half_coset_points<Value: IValue>(
    context: &mut Context<Value>,
    base_points: &CirclePoint<Simd>,
    log_size: u32,
) -> Vec<CirclePoint<Simd>> {
    assert!(log_size > 0);
    let gen_pt = generator_point_simd(context, log_size as usize, base_points.x.len());
    let mut curr_pt = base_points.clone();
    let mut half_coset = vec![curr_pt.clone()];
    let half_log_size = log_size - 1;
    for _ in 0..(1 << half_log_size) - 1 {
        curr_pt = add_points_simd(context, &curr_pt, &gen_pt);
        half_coset.push(curr_pt.clone());
    }
    // Bit reverse.
    stwo::core::utils::bit_reverse(&mut half_coset);
    half_coset
}
