use air_infra::core::Felt;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr, const_felt252_expr_from_felt252};

use super::ec_op_builtin::*;
use super::partial_ec_mul_generic::*;
use crate::casm::builtins::ec_utils::utils::*;

#[test]
fn test_partial_mul_generic() {
    let air_fn = &PartialECMulGeneric {};
    let (registry, _) = AirFnRegistry::new(air_fn);

    let call_id = const_expr!(0);

    let q = P_1;
    let [q_x, q_y] = [const_felt252_expr_from_felt252!(q.x), const_felt252_expr_from_felt252!(q.y)];

    let p = P_3;
    let [p_x, p_y] = [const_felt252_expr_from_felt252!(p.x), const_felt252_expr_from_felt252!(p.y)];

    let m = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_76543211,
        0x01234567_89abcdef_fedcba98_76543210
    )
    .into();

    let (_, output) = registry.run_air(
        air_fn,
        (),
        (
            call_id.clone(),
            const_expr!(0),
            (m, [q_x.clone(), q_y.clone()], [p_x.clone(), p_y.clone()], const_expr!(26)),
        ),
    );

    let m2: Felt252Width27Expr = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_732a1908,
        0x01234567_89abcdef_fedcba98_76543210
    )
    .into();

    let q2 = &ec_mul(&q, 2);
    let [q2_x, q2_y] =
        [const_felt252_expr_from_felt252!(q2.x), const_felt252_expr_from_felt252!(q2.y)];

    let res = ec_add(&p, &q);
    let [res_x, res_y] =
        [const_felt252_expr_from_felt252!(res.x), const_felt252_expr_from_felt252!(res.y)];

    assert_eq!(output.0.calc(), call_id.calc());
    assert_eq!(output.1.calc(), const_expr!(1).calc());

    assert_eq!(output.2.0.calc(), m2.calc());
    assert_eq!(output.2.1[0].calc(), q2_x.calc());
    assert_eq!(output.2.1[1].calc(), q2_y.calc());
    assert_eq!(output.2.2[0].calc(), res_x.calc());
    assert_eq!(output.2.2[1].calc(), res_y.calc());
    assert_eq!(output.2.3.calc(), const_expr!(25).calc());

    let (_, output2) = registry.run_air(air_fn, (), output);

    let m3: Felt252Width27Expr = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_71950c84,
        0x01234567_89abcdef_fedcba98_76543210
    )
    .into();

    let q4 = &ec_mul(&q, 4);
    let [q4_x, q4_y] =
        [const_felt252_expr_from_felt252!(q4.x), const_felt252_expr_from_felt252!(q4.y)];

    assert_eq!(output2.0.calc(), call_id.calc());
    assert_eq!(output2.1.calc(), const_expr!(2).calc());

    assert_eq!(output2.2.0.calc(), m3.calc());
    assert_eq!(output2.2.1[0].calc(), q4_x.calc());
    assert_eq!(output2.2.1[1].calc(), q4_y.calc());
    assert_eq!(output2.2.2[0].calc(), res_x.calc());
    assert_eq!(output2.2.2[1].calc(), res_y.calc());
    assert_eq!(output2.2.3.calc(), const_expr!(24).calc());

    let m_spec = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_78000001,
        0x01234567_89abcdef_fedcba98_76543210
    )
    .into();

    let (state, output3) = registry.run_air(
        air_fn,
        (),
        (call_id.clone(), const_expr!(26), (m_spec, [q_x, q_y], [p_x, p_y], const_expr!(0))),
    );

    let m_spec_2: Felt252Width27Expr =
        const_felt252_expr!(0xca8642002468acf13579bdffdb97530f, 0x2468acf13579bdffdb97530e).into();

    assert_eq!(output3.0.calc(), call_id.calc());
    assert_eq!(output3.1.calc(), const_expr!(27).calc());

    assert_eq!(output3.2.0.calc(), m_spec_2.calc());
    assert_eq!(output3.2.1[0].calc(), q2_x.calc());
    assert_eq!(output3.2.1[1].calc(), q2_y.calc());
    assert_eq!(output3.2.2[0].calc(), res_x.calc());
    assert_eq!(output3.2.2[1].calc(), res_y.calc());
    assert_eq!(output3.2.3.calc(), const_expr!(26).calc());

    assert_eq!(state.get_felts().len(), 624);
}

#[test]
#[should_panic(expected = "Added incorrect constraint")]
fn test_partial_mul_generic_special_round_check() {
    let air_fn = &PartialECMulGeneric {};
    let (registry, _) = AirFnRegistry::new(air_fn);

    let call_id = const_expr!(0);

    let q = P_1;
    let [q_x, q_y] = [const_felt252_expr_from_felt252!(q.x), const_felt252_expr_from_felt252!(q.y)];

    let p = P_3;
    let [p_x, p_y] = [const_felt252_expr_from_felt252!(p.x), const_felt252_expr_from_felt252!(p.y)];

    let m = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_7f000001,
        0x01234567_89abcdef_fedcba98_76543210
    )
    .into();

    registry.run_air(
        air_fn,
        (),
        (call_id.clone(), const_expr!(26), (m, [q_x, q_y], [p_x, p_y], const_expr!(0))),
    );
}

#[test]
fn test_ec_op_builtin() {
    let segment_start = 500;

    let q = P_1;
    let [q_x, q_y] = [const_felt252_expr_from_felt252!(q.x), const_felt252_expr_from_felt252!(q.y)];

    let p = P_3;
    let [p_x, p_y] = [const_felt252_expr_from_felt252!(p.x), const_felt252_expr_from_felt252!(p.y)];

    let m = const_felt252_expr!(
        0x01234567_89abcdef_fedcba98_76543210,
        0x01234567_89abcdef_fedcba98_76543210
    );

    let res_x =
        const_felt252_expr!(0x71e2b508e8181cbb1d1d955e69f8ad85, 0x6b9fe6322bda8c705fa87d37b75007d);
    let res_y =
        const_felt252_expr!(0x103e5924ba3b26345d80a182ed14e859, 0x6150ca9cae42b6c63f4a694429d0984);

    let memory = Felt252IdMemory::new_with_data(vec![
        (const_expr!(segment_start), p_x),
        (const_expr!(segment_start + 1), p_y),
        (const_expr!(segment_start + 2), q_x),
        (const_expr!(segment_start + 3), q_y),
        (const_expr!(segment_start + 4), m),
        (const_expr!(segment_start + 5), res_x),
        (const_expr!(segment_start + 6), res_y),
    ]);

    let ec_op = ECOpBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::ECOpBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&ec_op);

    let (state, _) = registry.run_air_with_row_number(&ec_op, (), (), 0);
    assert_eq!(state.get_felts().len(), 273);
}

#[test]
#[should_panic(expected = "Added incorrect constraint")]
fn test_ec_op_builtin_too_large_m() {
    let segment_start = 500;

    let q = P_1;
    let [q_x, q_y] = [const_felt252_expr_from_felt252!(q.x), const_felt252_expr_from_felt252!(q.y)];

    let p = P_3;
    let [p_x, p_y] = [const_felt252_expr_from_felt252!(p.x), const_felt252_expr_from_felt252!(p.y)];

    let m = const_felt252_expr!(1, 0x8000000000000110000000000000000);

    let res_x =
        const_felt252_expr!(0x71e2b508e8181cbb1d1d955e69f8ad85, 0x6b9fe6322bda8c705fa87d37b75007d);
    let res_y =
        const_felt252_expr!(0x103e5924ba3b26345d80a182ed14e859, 0x6150ca9cae42b6c63f4a694429d0984);

    let memory = Felt252IdMemory::new_with_data(vec![
        (const_expr!(segment_start), p_x),
        (const_expr!(segment_start + 1), p_y),
        (const_expr!(segment_start + 2), q_x),
        (const_expr!(segment_start + 3), q_y),
        (const_expr!(segment_start + 4), m),
        (const_expr!(segment_start + 5), res_x),
        (const_expr!(segment_start + 6), res_y),
    ]);

    let ec_op = ECOpBuiltin { memory: memory.clone() };
    let mut registry = AirFnRegistry::new_empty();
    registry.public_params.set(PublicParam::ECOpBuiltinSegmentStart, Felt::from(segment_start));
    registry.add_entry(&ec_op);

    registry.run_air_with_row_number(&ec_op, (), (), 0);
}
