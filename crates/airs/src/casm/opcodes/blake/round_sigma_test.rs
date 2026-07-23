use air_infra::const_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::variables::AsProverType;

use super::round_sigma::*;

#[test]
fn test_sigma() {
    let air_fn = BlakeRoundSigma {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (_, output) = registry.run_air(&air_fn, [const_expr!(7)], ());
    assert_eq!(output[0].calc(), "13");
    assert_eq!(output[1].calc(), "11");
    assert_eq!(output[2].calc(), "7");
    assert_eq!(output[3].calc(), "14");
    assert_eq!(output[4].calc(), "12");
    assert_eq!(output[5].calc(), "1");
    assert_eq!(output[6].calc(), "3");
    assert_eq!(output[7].calc(), "9");
    assert_eq!(output[8].calc(), "5");
    assert_eq!(output[9].calc(), "0");
    assert_eq!(output[10].calc(), "15");
    assert_eq!(output[11].calc(), "4");
    assert_eq!(output[12].calc(), "8");
    assert_eq!(output[13].calc(), "6");
    assert_eq!(output[14].calc(), "2");
    assert_eq!(output[15].calc(), "10");
}
