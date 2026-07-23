use air_infra::const_felt252_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::variables::AsProverType;

use super::ec_add::*;
use super::ec_double::*;

#[test]
fn test_ec_add() {
    let air_fn = &ECAdd {};
    let (registry, _) = AirFnRegistry::new(air_fn);

    let x1 =
        const_felt252_expr!(0x8fa8120b6d56eb0c1080d17957ebe47b, 0x234287dcbaffe7f969c748655fca9e5);
    let y1 =
        const_felt252_expr!(0x940135dd7a6c94cc6ed0268ee89e5615, 0x3b056f100f96fb21e889527d41f4e39);
    let x2 =
        const_felt252_expr!(0x99099ec1de5e3018b7a6932dba8aa378, 0x4fa56f376c83db33f9dab2656558f33);
    let y2 =
        const_felt252_expr!(0x562761f92a7a23b45168f4e80ff5b54d, 0x3fa0984c931c9e38113e0c0e47e4401);
    let (state, output) = registry.run_air(air_fn, (), [x1, y1, x2, y2]);
    assert_eq!(
        output[0].calc(),
        "[18168951315545398570, 9986881380086112593, 10206094637869389125, 442580642913464774]"
    );
    assert_eq!(
        output[1].calc(),
        "[71185753667440069, 12894077368487963351, 14618120260975419084, 281472859247110997]"
    );
    assert_eq!(state.get_felts().len(), 168);
}

#[test]
fn test_ec_double() {
    let air_fn = &ECDouble {};
    let (registry, _) = AirFnRegistry::new(air_fn);

    let x =
        const_felt252_expr!(0x8fa8120b6d56eb0c1080d17957ebe47b, 0x234287dcbaffe7f969c748655fca9e5);
    let y =
        const_felt252_expr!(0x940135dd7a6c94cc6ed0268ee89e5615, 0x3b056f100f96fb21e889527d41f4e39);

    let (state, output) = registry.run_air(air_fn, (), [x, y]);
    assert_eq!(
        output[0].calc(),
        "[3806079574567597174, 5185596089501980160, 466306979004812932, 16054420484727752]"
    );
    assert_eq!(
        output[1].calc(),
        "[15328562732460198937, 17488388439525081516, 11335053831013948278, 509520905531177432]"
    );
    assert_eq!(state.get_felts().len(), 253);
}
