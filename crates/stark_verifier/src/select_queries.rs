use circuits::blake::BLAKE2S_DIGEST_N_WORDS;
use circuits::context::Context;
use circuits::eval;
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
/// octuples of `u32` values, and transforming them into their bitwise representation as a vector of
/// Simd-s, each containing the `i`-th bit of each of the queries, up to `log_domain_size`.
pub fn get_query_bits_from_channel(
    context: &mut Context<impl IValue>,
    channel: &mut Channel,
    n_queries: usize,
    log_domain_size: u32,
) -> Vec<Simd> {
    let n_hashes = n_queries.div_ceil(BLAKE2S_DIGEST_N_WORDS);
    let mut queries_simd_vec = Vec::with_capacity(n_queries);
    for i in 0..n_hashes {
        let u32s = channel.draw_u32s(context);
        for j in 0..BLAKE2S_DIGEST_N_WORDS {
            if BLAKE2S_DIGEST_N_WORDS * i + j == n_queries {
                for k in j..BLAKE2S_DIGEST_N_WORDS {
                    context.mark_as_unused(*u32s[k].raw());
                }
                // Note that this is already necessarily the last iteration of the outer loop,
                // so we do not need to break out of it as well.
                break;
            }
            let simd = Simd::from_packed(vec![*u32s[j].raw()], 2);
            queries_simd_vec.push(simd);
        }
    }
    let mut bits = extract_limb_bits(context, &queries_simd_vec, 0);
    // Only extract the high limbs' bits if any are needed.
    if log_domain_size > 16 {
        bits.extend(extract_limb_bits(context, &queries_simd_vec, 1));
    }
    bits.truncate(log_domain_size.try_into().unwrap());

    bits
}

/// Extracts the bits of the `limb`-th 16-bit limb of every word in `words`: `bits[k]` is a [Simd]
/// whose lane `i` is the `k`-th bit of that limb of `words[i]` (`k = 0` is the LSB).
///
/// Each word is a `u32` drawn from the channel, held as a two-lane [Simd] `(low_u16, high_u16)`,
/// so `limb == 0` yields bits `0..16` of the words and `limb == 1` bits `16..32`.
fn extract_limb_bits(context: &mut Context<impl IValue>, words: &[Simd], limb: usize) -> Vec<Simd> {
    let limbs = words
        .iter()
        // `unpack_idx` yields the `limb`-th M31 coordinate of the word.
        .map(|word| Simd::unpack_idx(context, word, limb))
        .collect::<Vec<_>>();

    let packed_limbs = Simd::pack(context, &limbs);
    extract_bits(context, &packed_limbs, 16)
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

/// Takes the data obtained from [get_query_bits_from_channel], and returns [Queries].
pub fn select_queries(context: &mut Context<impl IValue>, bits: Vec<Simd>) -> Queries {
    let n_queries = bits[0].len();
    let log_domain_size: u32 = bits.len().try_into().unwrap();

    // Construct the circle point for each query.
    // Start with the generator of the subgroup of size `2 * domain_size` which is added to all
    // queries (to move it to the canonic coset).
    // See [stwo::core::poly::circle::CanonicCoset].
    let mut point = generator_point_simd(context, log_domain_size + 1, n_queries);

    for (i, bit) in (1..).zip(bits.iter().skip(1)) {
        // If the i-th bit (for i > 0) is 1, we need to add the generator of the subgroup
        // of size `2^i` to the current sum.
        let cur_gen_pt = generator_point_simd(context, i, n_queries);
        // Select between `point` and `point + cur_gen_pt`.
        let point_if_bit = add_points_simd(context, &point, &cur_gen_pt);
        point = CirclePoint {
            x: Simd::select(context, bit, &point.x, &point_if_bit.x),
            y: Simd::select(context, bit, &point.y, &point_if_bit.y),
        };
    }

    // Handle the first bit, which may negate the sign of `y`.
    let neg_y = eval!(context, -(point.y));
    point.y = Simd::select(context, &bits[0], &point.y, &neg_y);

    Queries { bits, points: point }
}
