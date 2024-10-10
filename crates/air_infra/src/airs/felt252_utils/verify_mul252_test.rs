use super::verify_mul252::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::utils::test_utils::*;

// Macros
use crate::const_felt252_expr;

#[test]
fn test_entry_json() {
    let (_, entry) = AirFnRegistry::new(&VerifyMul252 {});
    compare_json(
        &entry,
        &format!(
            "{}{}.json",
            TEST_JSONS_FELT252_DIR,
            entry.name.to_lowercase()
        ),
    );
}

#[test]
fn test_verify_mul252_no_overflow() {
    let air_fn = VerifyMul252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0x1008020001u128, 0u128),
            const_felt252_expr!(0x1ff8020001u128, 0u128),
            const_felt252_expr!(0x2008020003400040001u128, 0u128),
        ],
    );
    let expected_state = [
        "0", "0", "32", "4097", "160", "8193", "288", "33", "33", "3", "256", "2", "512", "2", "2",
        "2", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0", "0",
    ];

    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_verify_mul252_with_overflow() {
    let air_fn = VerifyMul252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(0, 1u128 << (251 - 128)),
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8002u128,
                0x7fffff52ad78032ffffffffffffdbe0u128
            ),
        ],
    );
    let expected_state = [
        "540",
        "2",
        "2147483619",
        "2147483630",
        "2147483618",
        "2147483618",
        "995",
        "2147483618",
        "2147483614",
        "2147483632",
        "2147483643",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483621",
        "2147483618",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483501",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483645",
        "8190",
    ];
    assert_eq!(state.calc(), expected_state);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
            const_felt252_expr!(
                0x4cc3ffffffffff5cdf8005u128,
                0x7fffff52ad78054ffffffffffffdbe0u128
            ),
        ],
    );
    let expected_state = [
        "18932",
        "2147475443",
        "2498",
        "13240",
        "23959",
        "34690",
        "46445",
        "62284",
        "82203",
        "102150",
        "122090",
        "142021",
        "161950",
        "181879",
        "201808",
        "221713",
        "241639",
        "261564",
        "281493",
        "301422",
        "321351",
        "341280",
        "160306",
        "129790",
        "99130",
        "68470",
        "37810",
        "15342",
    ];
    assert_eq!(state.calc(), expected_state);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(
                0x01234567_89abcdef_fedcba98_76543210u128,
                0x01234567_89abcdef_fedcba98_76543210u128
            ),
            const_felt252_expr!(
                0xffffffff_ffffffff_ffffffff_ffffffffu128,
                0x07ffffff_ffffffff_ffffffff_ffffffffu128
            ),
            const_felt252_expr!(
                0x4d5e6f8091adf6392ea61d94f496c460u128,
                0x0369d0350642c1926d3a06d3a06d34bau128
            ),
        ],
    );
    let expected_state = [
        "7240",
        "2147472679",
        "2147469618",
        "2147478266",
        "2147483469",
        "11390",
        "17350",
        "33535",
        "49488",
        "65605",
        "75072",
        "92636",
        "100768",
        "104544",
        "107895",
        "112780",
        "118318",
        "122926",
        "127148",
        "133306",
        "146715",
        "163065",
        "82686",
        "61648",
        "37898",
        "19428",
        "7904",
        "2290",
    ];
    assert_eq!(state.calc(), expected_state);
}

#[test]
fn test_verify_mul252_with_overflow_negative_k() {
    let air_fn = VerifyMul252 {};
    let (registry, _) = AirFnRegistry::new(&air_fn);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 1u128 << 88),
            const_felt252_expr!(0, 1u128 << 88),
            const_felt252_expr!(
                0x43ffffffffffff6f8000000000013310u128,
                0x14640fffe0000000000000u128
            ),
        ],
    );
    let expected_state = [
        "2147483643", // -4
        "2147483631",
        "2147483640",
        "0",
        "0",
        "0",
        "0",
        "0",
        "2147483632",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483644",
        "2147483646",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483615",
        "0",
        "0",
        "0",
        "0",
        "0",
        "0",
    ];
    assert_eq!(state.calc(), expected_state);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 0x1ffu128 << 88),
            const_felt252_expr!(0, 0x1ffu128 << 88),
            const_felt252_expr!(
                0x43fffffffdc0416f80000004c774f310u128,
                0x513ec423907fe0000000010ef0u128
            ),
        ],
    );
    let expected_state = [
        "2147479563", // -4084
        "2147483639",
        "2147483640",
        "2147483620",
        "2147483639",
        "0",
        "0",
        "0",
        "2147483632",
        "2147483642",
        "2147483631",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483644",
        "2147483618",
        "2147483644",
        "2147483646",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483615",
        "1082",
        "2",
        "0",
        "0",
        "0",
        "0",
    ];
    assert_eq!(state.calc(), expected_state);
    let (state, _) = registry.run_air(
        &air_fn,
        [
            const_felt252_expr!(0, 0x7fffffffffffffffu128 << 61),
            const_felt252_expr!(0, 0x7fffffffffffffffu128 << 61),
            const_felt252_expr!(
                0x800000000135530ffffffffd6eaf7e05u128,
                0x7ffffd459a75e997fffffffffff6e6fu128
            ),
        ],
    );
    let expected_state = [
        "2147457211", // -26624
        "24533",
        "20423",
        "16335",
        "12245",
        "8155",
        "4067",
        "2147483624",
        "2147483644",
        "2147483624",
        "2147483635",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483645",
        "2147483619",
        "2147483631",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483614",
        "2147483614",
        "200820",
        "165632",
        "129862",
        "94092",
        "58322",
        "22552",
    ];
    assert_eq!(state.calc(), expected_state);
}
