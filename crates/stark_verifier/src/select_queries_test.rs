use circuits::context::TraceContext;
use circuits::extract_bits::extract_bits;
use circuits::ivalue::qm31_from_u32s;
use circuits::test_utils::{packed_values, simd_from_u32s};
use itertools::Itertools;
use num_traits::{One, Zero};
use stwo::core::circle::CirclePoint;
use stwo::core::fields::m31::M31;

use crate::channel::Channel;
use crate::circle::generator_point;
use crate::select_queries::{get_query_bits_from_channel, select_queries};

#[test]
fn test_select_queries() {
    let mut context = TraceContext::default();

    const LOG_DOMAIN_SIZE: u32 = 5;

    let input = simd_from_u32s(&mut context, vec![24, 25]);
    let bits = extract_bits(&mut context, &input, LOG_DOMAIN_SIZE);
    let queries = select_queries(&mut context, bits);

    // Check that the first point is on the circle and in the relevant coset
    // (after `LOG_DOMAIN_SIZE` times doubling, we should get `(-1, 0)`).
    let x = packed_values(&context, &queries.points.x)[0].0.0;
    let y = packed_values(&context, &queries.points.y)[0].0.0;
    assert_eq!(x * x + y * y, 1.into());
    assert_eq!(
        CirclePoint { x, y }.repeated_double(LOG_DOMAIN_SIZE),
        CirclePoint { x: -M31::one(), y: M31::zero() }
    );

    // The first query index is `24 = 0b11000`. Removing the LSB and computing bit-reverse we
    // get 0b0011.
    assert_eq!(
        CirclePoint { x, y },
        generator_point(LOG_DOMAIN_SIZE + 1) + generator_point(LOG_DOMAIN_SIZE - 1).mul(0b0011)
    );

    // The second query index is `25 = 0b11001`. Removing the LSB and computing bit-reverse we
    // get 0b0011. Since the LSB is 1, we negate the result.
    let x = packed_values(&context, &queries.points.x)[0].0.1;
    let y = packed_values(&context, &queries.points.y)[0].0.1;
    assert_eq!(
        CirclePoint { x, y },
        -(generator_point(LOG_DOMAIN_SIZE + 1) + generator_point(LOG_DOMAIN_SIZE - 1).mul(0b0011))
    );

    context.validate_circuit();
}

#[test]
fn test_full_select_queries_regression() {
    let mut context = TraceContext::default();

    let init_digest =
        [271333035, 1833401714, 819175623, 1270120203, 1921341900, 364315769, 339695133, 365135865];
    let mut channel = Channel::from_digest(&mut context, init_digest);

    const N_QUERIES: usize = 3;
    const LOG_DOMAIN_SIZE: u32 = 5;

    let bits = get_query_bits_from_channel(&mut context, &mut channel, N_QUERIES, LOG_DOMAIN_SIZE);

    assert_eq!(Ok(bits.len()), LOG_DOMAIN_SIZE.try_into());
    assert_eq!(bits[0].len(), N_QUERIES);

    let queries = select_queries(&mut context, bits);

    assert_eq!(queries.points.x.len(), N_QUERIES);
    assert_eq!(queries.points.y.len(), N_QUERIES);
    assert_eq!(
        packed_values(&context, &queries.points.x),
        vec![qm31_from_u32s(567259857, 1952787376, 194696271, 579625837)]
    );
    assert_eq!(
        packed_values(&context, &queries.points.y),
        vec![qm31_from_u32s(1952787376, 1580223790, 567259857, 1690787918)]
    );
    assert_eq!(
        queries.bits.iter().map(|bits| packed_values(&context, bits)).collect_vec(),
        vec![
            vec![qm31_from_u32s(0, 0, 0, 0)],
            vec![qm31_from_u32s(1, 0, 1, 0)],
            vec![qm31_from_u32s(1, 0, 0, 0)],
            vec![qm31_from_u32s(0, 0, 0, 0)],
            vec![qm31_from_u32s(1, 1, 1, 0)],
        ]
    );

    context.validate_circuit();
}
