use std::array::from_fn;

use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::ec_add::*;
use crate::airs::casm::builtins::pedersen::partial_ec_mul::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;
use crate::{const_expr, const_felt252_expr};

#[test]
fn test_ec_add() {
    let air_fn = &ECAdd {};
    let (registry, _) = AirFnRegistry::new(air_fn);

    let x1 = const_felt252_expr!(
        0x8fa8120b6d56eb0c1080d17957ebe47b,
        0x234287dcbaffe7f969c748655fca9e5
    );
    let y1 = const_felt252_expr!(
        0x940135dd7a6c94cc6ed0268ee89e5615,
        0x3b056f100f96fb21e889527d41f4e39
    );
    let x2 = const_felt252_expr!(
        0x99099ec1de5e3018b7a6932dba8aa378,
        0x4fa56f376c83db33f9dab2656558f33
    );
    let y2 = const_felt252_expr!(
        0x562761f92a7a23b45168f4e80ff5b54d,
        0x3fa0984c931c9e38113e0c0e47e4401
    );
    let (state, output) = registry.run_air(air_fn, (), [x1, y1, x2, y2]);
    assert_eq!(
        output[0].calc(),
        "[18168951315545398570, 9986881380086112593, 10206094637869389125, 442580642913464774]"
    );
    assert_eq!(
        output[1].calc(),
        "[71185753667440069, 12894077368487963351, 14618120260975419084, 281472859247110997]"
    );
    assert_eq!(state.get_felts().len(), 342);
}

fn pack_to_double_limbs(mut value: u64) -> PackedECMultiplier {
    const MASK: u64 = (1 << (2 * FELT252_BITS_PER_WORD)) - 1;
    from_fn(|_| {
        let double_limb = const_expr!(TryInto::<u32>::try_into(value & MASK).unwrap());
        value >>= FELT252_BITS_PER_WORD * 2;
        double_limb
    })
}

#[test]
fn test_partial_mul() {
    let air_fn = &PartialECMul {};
    let (registry, _) = AirFnRegistry::new(air_fn);

    let call_id = const_expr!(0);
    let round_num = const_expr!(0);

    // The offset of P_2 data in the PedersenPoints table
    let table_offset = const_expr!(14 * (1 << 18) + 16);

    // The coordinates of the P_1 Pedersen point
    let p1_x = const_felt252_expr!(
        0x99099ec1de5e3018b7a6932dba8aa378,
        0x4fa56f376c83db33f9dab2656558f33
    );
    let p1_y = const_felt252_expr!(
        0x562761f92a7a23b45168f4e80ff5b54d,
        0x3fa0984c931c9e38113e0c0e47e4401
    );

    // The coordinates of 123*P_2 + P_1 - P_shift
    let result_x = const_felt252_expr!(
        0x7a4c1984a6378d044dc713b82334ae95,
        0x3cc665052f9f6ef74cf12115f581ceb
    );
    let result_y = const_felt252_expr!(
        0x5ea50aed74a13750dadabc68ec02f0b6,
        0x5c37356ef85d203b9c0aa52bb75b3df
    );

    let multiplier = (7 << 18) + 123;
    let (state, output) = registry.run_air(
        air_fn,
        (),
        (
            call_id.clone(),
            round_num,
            (
                table_offset.clone(),
                pack_to_double_limbs(multiplier),
                [p1_x, p1_y],
            ),
        ),
    );
    assert_eq!(output.0.calc(), call_id.calc());
    assert_eq!(output.1.calc(), const_expr!(1).calc());
    assert_eq!(output.2 .0.calc(), table_offset.calc());

    let expected_new_multiplier = pack_to_double_limbs(multiplier >> 18);
    for (output_elem, expected_elem) in output.2 .1.iter().zip(expected_new_multiplier.iter()) {
        assert_eq!(output_elem.calc(), expected_elem.calc());
    }
    assert_eq!(output.2 .2[0].calc(), result_x.calc());
    assert_eq!(output.2 .2[1].calc(), result_y.calc());
    assert_eq!(state.get_felts().len(), 471);
}
