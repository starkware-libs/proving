use circuits::context::TraceContext;
use circuits::ops::Guess;
use circuits::simd::Simd;
use circuits::test_utils::{packed_values, simd_from_u32s};
use circuits::wrappers::M31Wrapper;
use itertools::Itertools;
use num_traits::Zero;
use stwo::core::circle::CirclePoint;
use stwo::core::fields::cm31::CM31;
use stwo::core::fields::m31::M31;
use stwo::core::fields::qm31::QM31;

use crate::circle::{
    add_points, add_points_simd, double_point, double_point_simd, double_x, double_x_simd,
};

#[test]
fn test_double_x() {
    let pt0 = CirclePoint::<M31> { x: 102767539.into(), y: 739428083.into() };
    let pt1 = CirclePoint::<M31> { x: 1562688784.into(), y: 946400219.into() };

    let double_pt0 = pt0 + pt0;
    let double_pt1 = pt1 + pt1;

    let mut context = TraceContext::default();

    // Regular version.
    let input = QM31::from(pt0.x).guess(&mut context);
    let res = double_x(&mut context, input);
    assert_eq!(context.get(res), double_pt0.x.into());

    // Simd version.
    let input_simd = simd_from_u32s(&mut context, vec![pt0.x.0, pt1.x.0]);
    let res_simd = double_x_simd(&mut context, &input_simd);
    assert_eq!(res_simd.len(), 2);
    assert_eq!(packed_values(&context, &res_simd)[0].0, CM31(double_pt0.x, double_pt1.x));

    context.validate_circuit();
}

#[test]
fn test_add_points() {
    let pt0 = CirclePoint::<QM31> { x: 102767539.into(), y: 739428083.into() };
    let pt1 = CirclePoint::<QM31> { x: 946122697.into(), y: 337868966.into() };
    let mut context = TraceContext::default();

    let pt0_var = &pt0.guess(&mut context);
    let pt1_var = &pt1.guess(&mut context);

    let res = add_points(&mut context, pt0_var, pt1_var);

    let expected_res = pt0 + pt1;

    assert_eq!(context.get(res.x), expected_res.x);
    assert_eq!(context.get(res.y), expected_res.y);

    context.validate_circuit();
}

#[test]
fn test_add_points_simd() {
    let pt0 = CirclePoint::<M31> { x: 102767539.into(), y: 739428083.into() };
    let pt1 = CirclePoint::<M31> { x: 1562688784.into(), y: 946400219.into() };

    let pt2 = CirclePoint::<M31> { x: 946122697.into(), y: 337868966.into() };
    let pt3 = CirclePoint::<M31> { x: 2104020285.into(), y: 511427956.into() };
    let mut context = TraceContext::default();

    let first_points = CirclePoint {
        x: simd_from_u32s(&mut context, vec![pt0.x.0, pt1.x.0]),
        y: simd_from_u32s(&mut context, vec![pt0.y.0, pt1.y.0]),
    };
    let second_points = CirclePoint {
        x: simd_from_u32s(&mut context, vec![pt2.x.0, pt3.x.0]),
        y: simd_from_u32s(&mut context, vec![pt2.y.0, pt3.y.0]),
    };
    let res = add_points_simd(&mut context, &first_points, &second_points);

    let pt0_plus_pt2 = pt0 + pt2;
    let pt1_plus_pt3 = pt1 + pt3;

    assert_eq!(
        packed_values(&context, &res.x),
        vec![QM31(CM31(pt0_plus_pt2.x, pt1_plus_pt3.x), CM31::zero())]
    );
    assert_eq!(
        packed_values(&context, &res.y),
        vec![QM31(CM31(pt0_plus_pt2.y, pt1_plus_pt3.y), CM31::zero())]
    );

    context.validate_circuit();
}

#[test]
fn test_double_points() {
    let pt0 = CirclePoint::<M31> { x: 102767539.into(), y: 739428083.into() };

    let mut context = TraceContext::default();

    let pt0_var =
        CirclePoint { x: M31Wrapper::from(pt0.x), y: M31Wrapper::from(pt0.y) }.guess(&mut context);

    let res = double_point(&mut context, &pt0_var);
    let double_pt0 = pt0 + pt0;

    assert_eq!(context.get(*res.x.get()).0.0, double_pt0.x);
    assert_eq!(context.get(*res.y.get()).0.0, double_pt0.y);
}

#[test]
fn test_double_points_simd() {
    let pt0 = CirclePoint::<M31> { x: 102767539.into(), y: 739428083.into() };
    let pt1 = CirclePoint::<M31> { x: 1562688784.into(), y: 946400219.into() };

    let mut context = TraceContext::default();

    let points = CirclePoint {
        x: simd_from_u32s(&mut context, vec![pt0.x.0, pt1.x.0]),
        y: simd_from_u32s(&mut context, vec![pt0.y.0, pt1.y.0]),
    };

    let res = double_point_simd(&mut context, &points);

    let unpacked_x = Simd::unpack(&mut context, &res.x);
    let unpacked_y = Simd::unpack(&mut context, &res.y);

    let double_pt0 = pt0 + pt0;
    let double_pt1 = pt1 + pt1;

    assert_eq!(
        unpacked_x.iter().map(|x| context.get(*x).0.0).collect_vec(),
        vec![double_pt0.x, double_pt1.x]
    );
    assert_eq!(
        unpacked_y.iter().map(|y| context.get(*y).0.0).collect_vec(),
        vec![double_pt0.y, double_pt1.y]
    );
}
