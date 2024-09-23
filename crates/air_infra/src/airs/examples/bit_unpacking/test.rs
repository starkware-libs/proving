use super::bit_unpack::*;
use super::div2::*;

use crate::core::air_fn::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_expr;
use crate::u16_expr;

#[test]
fn test_bit_unpacking() {
    let func = BitUnpack::<4> {};
    let registry = AirFnRegistry::new(&func);

    let (state, output) = registry.run_air(&func, u16_expr!("x", 10));
    assert_eq!(state.calc(), ["10", "5", "2", "1", "0"]);
    assert!(
        output.iter().map(|x| x.calc()).collect::<Vec<String>>()
            == ["false", "true", "false", "true"]
    );

    // Check entry
    compare_test_json(
        &registry,
        &func.name(),
        &(TEST_JSONS_EXAMPLES_DIR.to_owned() + "bit_unpacking.json"),
    );
}

#[derive(Debug)]
struct AirFnBitMux {}

impl AirFn for AirFnBitMux {
    type In = UInt16Expr;
    type Out = BoolExpr;

    fn call(&self, air_builder: &mut AirBuilder, mut x: Self::In) -> Self::Out {
        let x_f = air_builder.deduce(x.as_felt_mut());
        let air_fn = Div2 {};

        let (bit, _) = air_builder.call(&air_fn, x.clone());
        air_builder.constrain(
            (bit.as_felt() * x_f.clone())
                + (const_expr!(1) - bit.as_felt()) * (x_f - const_expr!(2)),
        );
        bit
    }
}

#[test]
fn test_bit_mux() {
    let func = AirFnBitMux {};
    let registry = AirFnRegistry::new(&func);
    let (_, out) = registry.run_air(&func, u16_expr!("x", 2));
    assert!(out.calc() == "false");

    // Check entry
    compare_test_json(
        &registry,
        &func.name(),
        &(TEST_JSONS_EXAMPLES_DIR.to_owned() + "bit_mux.json"),
    );
}
