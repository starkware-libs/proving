use circuits::EXTENSION_DEGREE;
use circuits::context::Context;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::IValue;
use circuits::simd::Simd;
use stwo::core::circle::CirclePoint;

use crate::channel::Channel;
use crate::circle::{add_points_simd, generator_point_simd};

#[cfg(test)]
#[path = "select_queries_test.rs"]
pub mod test;

/// Fetches the input for query selection from the channel, by drawing `ceil(n_queries / 8)`
/// pairs of `QM31` values. Each pair consists of 8 `M31` values. One `M31` for each query.
pub fn get_query_selection_input_from_channel(
    context: &mut Context<impl IValue>,
    channel: &mut Channel,
    n_queries: usize,
) -> Simd {
    let mut v = Vec::new();
    for _ in 0..n_queries.div_ceil(EXTENSION_DEGREE * 2) {
        v.extend_from_slice(&channel.draw_two_qm31s(context));
    }
    let n_qm31s = n_queries.div_ceil(EXTENSION_DEGREE);
    if n_qm31s % 2 == 1 {
        context.mark_as_unused(v[n_qm31s]);
    }

    Simd::from_packed(v[0..n_qm31s].to_vec(), n_queries)
}

/// Information regarding the chosen queries in the evaluation domain.
pub struct Queries {
    /// A vector with the bit representation of all the indices of the chosen queries in the
    /// evaluation domain.
    /// `bits[i]` is a [Simd] with the `i`-th bit of each of the queries (`i = 0` is the LSB).
    pub bits: Vec<Simd>,
    /// The circle points from the evaluation domain that correspond to the chosen queries.
    pub points: CirclePoint<Simd>,
}

/// Takes the data obtained from [get_query_selection_input_from_channel], and returns [Queries].
pub fn select_queries(
    context: &mut Context<impl IValue>,
    input: &Simd,
    log_domain_size: usize,
) -> Queries {
    let n_bits = 31;
    let bits = extract_bits(context, input, n_bits)[0..log_domain_size].to_vec();

    // Construct the circle point for each query.
    // Start with the generator of the subgroup of size `2 * domain_size` which is added to all
    // queries (to move it to the canonic coset).
    // See [stwo::core::poly::circle::CanonicCoset].
    let mut point = generator_point_simd(context, log_domain_size + 1, input.len());

    for (i, bit) in bits.iter().enumerate().skip(1) {
        // If the i-th bit (for i > 0) is 1, we need to add the generator of the subgroup
        // of size `2^i` to the current sum.
        let cur_gen_pt = generator_point_simd(context, i, input.len());
        // Select between `point` and `point + cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &point, &cur_gen_pt);
        point = CirclePoint {
            x: Simd::select(context, bit, &point.x, &point_if_bit.x),
            y: Simd::select(context, bit, &point.y, &point_if_bit.y),
        };
    }

    // Handle the first bit, which may negate the sign of `y`.
    let zero = Simd::zero(context, input.len());
    let neg_y = Simd::sub(context, &zero, &point.y);
    point.y = Simd::select(context, &bits[0], &point.y, &neg_y);

    Queries { bits, points: point }
}
