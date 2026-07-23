use std::path::Path;

use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::core::variables::AsProverType;
use air_infra::test_utils::compare_json;
use air_infra::{const_expr, const_u16_expr};
use expect_test::expect;
use serde::Serialize;

use super::bit_unpack::*;
use super::div2::*;
use crate::examples::TEST_JSONS_EXAMPLES_DIR;

#[test]
fn test_bit_unpacking() {
    let func = BitUnpack::<4>::new();
    let (registry, entry) = AirFnRegistry::new(&func);

    let (state, output) = registry.run_air(&func, (), const_u16_expr!(10));

    expect![[r#"
        (10, ""),
        (5, ""),
        (2, ""),
        (1, ""),
        (0, ""),
    "#]]
    .assert_eq(&state.to_string());

    assert!(
        output.iter().map(|x| x.calc()).collect::<Vec<String>>()
            == ["false", "true", "false", "true"]
    );

    // Check entry
    compare_json(
        registry.compile().get(&entry.name).unwrap(),
        &Path::new(TEST_JSONS_EXAMPLES_DIR).join(format!("{}.json", entry.name)),
    );
}

#[derive(Debug, Serialize)]
struct AirFnBitMux {}

impl AirFn for AirFnBitMux {
    type ExtIn = ();
    type In = UInt16Expr;
    type Out = BoolExpr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), mut x: Self::In) -> Self::Out {
        let x_f = air_builder.deduce(x.as_felt_mut(), "");
        let air_fn = Div2 {};

        let (bit, _) = air_builder.call(&air_fn, x.clone());
        air_builder.constrain(
            (bit.as_felt() * x_f.clone())
                + (const_expr!(1) - bit.as_felt()) * (x_f - const_expr!(2)),
            "",
        );
        bit
    }
}

#[test]
fn test_bit_mux() {
    let func = AirFnBitMux {};
    let (registry, entry) = AirFnRegistry::new(&func);
    let (_, out) = registry.run_air(&func, (), const_u16_expr!(2));
    assert!(out.calc() == "false");

    // Check entry
    compare_json(
        registry.compile().get(&entry.name).unwrap(),
        &Path::new(TEST_JSONS_EXAMPLES_DIR).join(format!("{}.json", entry.name)),
    );
}
