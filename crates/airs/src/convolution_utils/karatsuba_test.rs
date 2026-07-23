use air_infra::const_expr;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::variables::AsProverType;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use super::karatsuba::*;

// MOD_BUILTIN_N_WORDS(4) * MOD_BUILTIN_WORD_BIT_LEN(96) / MUL_MOD_LIMB_SIZE(12) / 4
const MUL_MOD_KARATSUBA_N: usize = 8;
// (1 << MUL_MOD_LIMB_SIZE(12)) - 1
const MUL_MOD_MAX_LIMB: i32 = 4095;

pub const FELT252_MAX_LIMB: i32 = (1 << FELT252_BITS_PER_WORD) - 1;
// We assume FELT252_N_WORDS is divisible by 4.
pub const MUL252_KARATSUBA_N: usize = FELT252_N_WORDS / 4;

#[test]
fn test_single_karatsuba_input_len_16() {
    let air_fn = SingleKaratsuba::<MUL_MOD_KARATSUBA_N>::new();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let x_array: [FeltExpr; 2 * MUL_MOD_KARATSUBA_N] = [
        const_expr!(780232741),
        const_expr!(245989546),
        const_expr!(2065280991),
        const_expr!(2069591409),
        const_expr!(1078610017),
        const_expr!(1074648517),
        const_expr!(1719541011),
        const_expr!(2076841332),
        const_expr!(959500694),
        const_expr!(708510517),
        const_expr!(1629754310),
        const_expr!(1460270305),
        const_expr!(449891098),
        const_expr!(1212460780),
        const_expr!(698732234),
        const_expr!(693813550),
    ];

    let y_array: [FeltExpr; 2 * MUL_MOD_KARATSUBA_N] = [
        const_expr!(607079534),
        const_expr!(285946416),
        const_expr!(439890484),
        const_expr!(1013943582),
        const_expr!(2050832306),
        const_expr!(663955453),
        const_expr!(1600812298),
        const_expr!(194401748),
        const_expr!(1256185754),
        const_expr!(1210416471),
        const_expr!(1030265932),
        const_expr!(159544186),
        const_expr!(1798479812),
        const_expr!(1113807726),
        const_expr!(289999313),
        const_expr!(1935783627),
    ];

    let simple_convolution_output: [FeltExpr; 4 * MUL_MOD_KARATSUBA_N - 1] =
        simple_convolution(&x_array, &y_array);

    let (state, single_karatsuba_output) = registry.run_air(&air_fn, (), [x_array, y_array]);

    assert_eq!(single_karatsuba_output.len(), simple_convolution_output.len());

    for (i, z) in single_karatsuba_output.iter().enumerate() {
        assert_eq!(z.calc(), simple_convolution_output[i].calc());
    }

    assert!(state.is_empty());
}

#[test]
fn test_double_karatsuba_input_len_32() {
    let air_fn =
        DoubleKaratsuba::<MUL_MOD_KARATSUBA_N>::new(MUL_MOD_MAX_LIMB * MUL_MOD_MAX_LIMB, 0);
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let x_array: [FeltExpr; 4 * MUL_MOD_KARATSUBA_N] = [
        const_expr!(620743303),
        const_expr!(870649070),
        const_expr!(1139969744),
        const_expr!(56305448),
        const_expr!(1003262244),
        const_expr!(1833222190),
        const_expr!(1574237613),
        const_expr!(878481938),
        const_expr!(2055752278),
        const_expr!(2038523389),
        const_expr!(1518228450),
        const_expr!(938410050),
        const_expr!(1427364848),
        const_expr!(659663046),
        const_expr!(167445015),
        const_expr!(1659655364),
        const_expr!(1018720653),
        const_expr!(1343656508),
        const_expr!(528678983),
        const_expr!(919878833),
        const_expr!(367711712),
        const_expr!(1809128709),
        const_expr!(615389221),
        const_expr!(918580472),
        const_expr!(1040728080),
        const_expr!(28306943),
        const_expr!(1219840967),
        const_expr!(1218810927),
        const_expr!(898020876),
        const_expr!(856976893),
        const_expr!(1032152853),
        const_expr!(806761287),
    ];

    let y_array: [FeltExpr; 4 * MUL_MOD_KARATSUBA_N] = [
        const_expr!(1210442425),
        const_expr!(701126766),
        const_expr!(258938760),
        const_expr!(1687685095),
        const_expr!(497004537),
        const_expr!(1636963783),
        const_expr!(132905627),
        const_expr!(373106161),
        const_expr!(1747473939),
        const_expr!(731869736),
        const_expr!(463032838),
        const_expr!(402189387),
        const_expr!(1743729855),
        const_expr!(716536865),
        const_expr!(979967915),
        const_expr!(703106135),
        const_expr!(1991535929),
        const_expr!(1052561955),
        const_expr!(878039668),
        const_expr!(2094414200),
        const_expr!(2067448713),
        const_expr!(682802235),
        const_expr!(545364327),
        const_expr!(2040487431),
        const_expr!(1739803014),
        const_expr!(1210385395),
        const_expr!(389805167),
        const_expr!(1198790417),
        const_expr!(1012419853),
        const_expr!(1050178572),
        const_expr!(1821213998),
        const_expr!(691641612),
    ];

    let simple_convolution_output: [FeltExpr; 8 * MUL_MOD_KARATSUBA_N - 1] =
        simple_convolution(&x_array, &y_array);

    let (state, double_karatsuba_output) = registry.run_air(&air_fn, (), [x_array, y_array]);

    assert_eq!(double_karatsuba_output.len(), simple_convolution_output.len());

    for (i, z) in double_karatsuba_output.iter().enumerate() {
        assert_eq!(z.calc(), simple_convolution_output[i].calc());
    }

    assert!(state.is_empty());
}

#[test]
fn test_single_karatsuba_input_len_14() {
    let air_fn = SingleKaratsuba::<MUL252_KARATSUBA_N>::new();
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let x_array: [FeltExpr; 2 * MUL252_KARATSUBA_N] = [
        const_expr!(780232741),
        const_expr!(245989546),
        const_expr!(2065280991),
        const_expr!(2069591409),
        const_expr!(1078610017),
        const_expr!(1074648517),
        const_expr!(1719541011),
        const_expr!(708510517),
        const_expr!(1629754310),
        const_expr!(1460270305),
        const_expr!(449891098),
        const_expr!(1212460780),
        const_expr!(698732234),
        const_expr!(693813550),
    ];

    let y_array: [FeltExpr; 2 * MUL252_KARATSUBA_N] = [
        const_expr!(607079534),
        const_expr!(285946416),
        const_expr!(439890484),
        const_expr!(1013943582),
        const_expr!(2050832306),
        const_expr!(663955453),
        const_expr!(1600812298),
        const_expr!(194401748),
        const_expr!(1256185754),
        const_expr!(1210416471),
        const_expr!(1798479812),
        const_expr!(1113807726),
        const_expr!(289999313),
        const_expr!(1935783627),
    ];

    let simple_convolution_output: [FeltExpr; 4 * MUL252_KARATSUBA_N - 1] =
        simple_convolution(&x_array, &y_array);

    let (state, single_karatsuba_output) = registry.run_air(&air_fn, (), [x_array, y_array]);

    assert_eq!(single_karatsuba_output.len(), simple_convolution_output.len());

    for (i, z) in single_karatsuba_output.iter().enumerate() {
        assert_eq!(z.calc(), simple_convolution_output[i].calc());
    }

    assert!(state.is_empty());
}

#[test]
fn test_double_karatsuba_input_len_28() {
    let air_fn = DoubleKaratsuba::<MUL252_KARATSUBA_N>::new(
        FELT252_MAX_LIMB * FELT252_MAX_LIMB,
        -FELT252_MAX_LIMB * FELT252_MAX_LIMB,
    );
    let (registry, _) = AirFnRegistry::new(&air_fn);

    let x_array: [FeltExpr; 28] = [
        const_expr!(620743303),
        const_expr!(870649070),
        const_expr!(1139969744),
        const_expr!(56305448),
        const_expr!(1003262244),
        const_expr!(1833222190),
        const_expr!(1574237613),
        const_expr!(878481938),
        const_expr!(1518228450),
        const_expr!(938410050),
        const_expr!(1427364848),
        const_expr!(659663046),
        const_expr!(167445015),
        const_expr!(1659655364),
        const_expr!(1018720653),
        const_expr!(1343656508),
        const_expr!(528678983),
        const_expr!(919878833),
        const_expr!(367711712),
        const_expr!(1809128709),
        const_expr!(1040728080),
        const_expr!(28306943),
        const_expr!(1219840967),
        const_expr!(1218810927),
        const_expr!(898020876),
        const_expr!(856976893),
        const_expr!(1032152853),
        const_expr!(806761287),
    ];

    let y_array: [FeltExpr; 28] = [
        const_expr!(1210442425),
        const_expr!(701126766),
        const_expr!(258938760),
        const_expr!(1687685095),
        const_expr!(497004537),
        const_expr!(373106161),
        const_expr!(1747473939),
        const_expr!(731869736),
        const_expr!(463032838),
        const_expr!(402189387),
        const_expr!(1743729855),
        const_expr!(716536865),
        const_expr!(979967915),
        const_expr!(1052561955),
        const_expr!(878039668),
        const_expr!(2094414200),
        const_expr!(2067448713),
        const_expr!(682802235),
        const_expr!(545364327),
        const_expr!(2040487431),
        const_expr!(1739803014),
        const_expr!(1210385395),
        const_expr!(389805167),
        const_expr!(1198790417),
        const_expr!(1012419853),
        const_expr!(1050178572),
        const_expr!(1821213998),
        const_expr!(691641612),
    ];

    let simple_convolution_output: [FeltExpr; 8 * MUL252_KARATSUBA_N - 1] =
        simple_convolution(&x_array, &y_array);

    let (state, double_karatsuba_output) = registry.run_air(&air_fn, (), [x_array, y_array]);

    assert_eq!(double_karatsuba_output.len(), simple_convolution_output.len());

    for (i, z) in double_karatsuba_output.iter().enumerate() {
        assert_eq!(z.calc(), simple_convolution_output[i].calc());
    }

    assert!(state.is_empty());
}
