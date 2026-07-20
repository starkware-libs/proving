use expect_test::expect;

use super::memory::*;
use super::read_positive::*;
use super::read_small::*;
use super::verify::*;
use crate::casm_state::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::variables::*;
use crate::{const_expr, const_felt252_expr};

#[test]
fn test_read_small() {
    let mem_data = vec![
        // Small positive
        (const_expr!(1), const_felt252_expr!(7i128)),
        (
            // Small positive duplicate
            const_expr!(2),
            const_felt252_expr!(7i128),
        ),
        (
            // Minus one
            const_expr!(3),
            const_felt252_expr!(-1i128),
        ),
        (
            // Minus two
            const_expr!(4),
            const_felt252_expr!(-2i128),
        ),
        (
            // P
            const_expr!(5),
            const_felt252_expr!(1, 10633823966279327296825105735305134080),
        ),
        (
            // P + 1
            const_expr!(6),
            const_felt252_expr!(2, 10633823966279327296825105735305134080),
        ),
        (
            // Minus 2**27
            const_expr!(7),
            const_felt252_expr!(-(1i128 << 27)),
        ),
        (
            // Minus 2**28
            const_expr!(8),
            const_felt252_expr!(-(1i128 << 28)),
        ),
        (
            // Minus 2**29
            const_expr!(9),
            const_felt252_expr!(-(1i128 << 29)),
        ),
        (
            // Minus (2**29 - 1)
            const_expr!(10),
            const_felt252_expr!(-(1i128 << 29) + 1),
        ),
        (
            // 2**29 - 1
            const_expr!(11),
            const_felt252_expr!((1i128 << 29) - 1),
        ),
    ];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let (registry, _) = AirFnRegistry::new(&read_small);

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(1), ""));
    assert_eq!(output.0.calc(), "7".to_string());
    expect![[r#"
        (0, "id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (7, "value_limb_0"),
        (0, "value_limb_1"),
        (0, "value_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(2), ""));
    assert_eq!(output.0.calc(), "7".to_string());
    expect![[r#"
        (0, "id"),
        (0, "msb"),
        (0, "mid_limbs_set"),
        (7, "value_limb_0"),
        (0, "value_limb_1"),
        (0, "value_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(3), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 2).to_string());
    expect![[r#"
        (1, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (0, "value_limb_0"),
        (0, "value_limb_1"),
        (0, "value_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(4), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 3).to_string());
    expect![[r#"
        (2, "id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (511, "value_limb_0"),
        (511, "value_limb_1"),
        (511, "value_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(5), ""));
    assert_eq!(output.0.calc(), "0".to_string());
    expect![[r#"
        (3, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (1, "value_limb_0"),
        (0, "value_limb_1"),
        (0, "value_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(6), ""));
    assert_eq!(output.0.calc(), "1".to_string());
    expect![[r#"
        (4, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (2, "value_limb_0"),
        (0, "value_limb_1"),
        (0, "value_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());

    let (_state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(7), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 1 - (1i64 << 27)).to_string());

    let (_state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(8), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 1 - (1i64 << 28)).to_string());

    let (_state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(9), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 1 - (1i64 << 29)).to_string());

    let (_state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(10), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - (1i64 << 29)).to_string());

    let (_state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(11), ""));
    assert_eq!(output.0.calc(), ((1i64 << 29) - 1).to_string());
}

#[should_panic(expected = "given value != value in memory")]
#[test]
fn test_read_small_too_big() {
    let mem_data = vec![(const_expr!(1), const_felt252_expr!(1i128 << 29))];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let (registry, _) = AirFnRegistry::new(&read_small);

    registry.run_air(&read_small, (), CasmAddress::new(const_expr!(1), ""));
}

#[test]
fn test_read_small_p_plus_edge() {
    // P + (2**29 - 2)
    let mem_data = vec![(
        const_expr!(1),
        const_felt252_expr!((1u128 << 29) - 1, 10633823966279327296825105735305134080),
    )];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let (registry, _) = AirFnRegistry::new(&read_small);

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(1), ""));
    assert_eq!(output.0.calc(), ((1i64 << 29) - 2).to_string());
    expect![[r#"
        (0, "id"),
        (1, "msb"),
        (0, "mid_limbs_set"),
        (511, "value_limb_0"),
        (511, "value_limb_1"),
        (511, "value_limb_2"),
        (3, "remainder_bits"),
        (1, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[should_panic(expected = "given value != value in memory")]
#[test]
fn test_read_small_p_plus_too_big() {
    // P + (2**29 - 1)
    let mem_data = vec![(
        const_expr!(1),
        const_felt252_expr!((1u128 << 29), 10633823966279327296825105735305134080),
    )];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let (registry, _) = AirFnRegistry::new(&read_small);

    let (_state, _output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(1), ""));
    // assert_eq!(output.0.calc(), ((1i64 << 29) - 1).to_string());
}

#[test]
fn test_read_small_negative_edge() {
    let mem_data = vec![(const_expr!(1), const_felt252_expr!(-(1 + (1i128 << 29))))];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let (registry, _) = AirFnRegistry::new(&read_small);

    let (state, output) = registry.run_air(&read_small, (), CasmAddress::new(const_expr!(1), ""));
    assert_eq!(output.0.calc(), ((1i64 << 31) - 2 - (1i64 << 29)).to_string());
    expect![[r#"
        (0, "id"),
        (1, "msb"),
        (1, "mid_limbs_set"),
        (0, "value_limb_0"),
        (0, "value_limb_1"),
        (0, "value_limb_2"),
        (0, "remainder_bits"),
        (0, "partial_limb_msb"),
    "#]]
    .assert_eq(&state.to_string());
}

#[should_panic(expected = "given value != value in memory")]
#[test]
fn test_read_small_too_negative() {
    let mem_data = vec![(const_expr!(1), const_felt252_expr!(-(2 + (1i128 << 29))))];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    let read_small = ReadSmall { memory };
    let (registry, _) = AirFnRegistry::new(&read_small);

    registry.run_air(&read_small, (), CasmAddress::new(const_expr!(1), ""));
}

fn test_read_positive(value: Felt252Expr, num_bits: usize) {
    let memory = Felt252IdMemory::new_with_data(vec![(const_expr!(0), value.clone())]);

    let read_positive = ReadPositive { memory, num_bits };

    let (registry, _) = AirFnRegistry::new(&read_positive);
    let (_state, output) =
        registry.run_air(&read_positive, (), CasmAddress::new(const_expr!(0), ""));

    assert_eq!(output.0.calc(), value.calc());
}

#[test]
fn test_read_positive_whole_limbs() {
    test_read_positive(const_felt252_expr!(1u128 << 35, 0), 36);
}

#[test]
fn test_read_positive_partial_limbs() {
    test_read_positive(const_felt252_expr!(9, 0), 6);
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 0: RangeCheck6 on input 510")]
fn test_read_positive_failure() {
    // Try to read a small negative number using ReadPositive
    test_read_positive(const_felt252_expr!(u128::MAX - 1, u128::MAX), 6);
}

#[test]
fn test_verify_all() {
    let mem_data = vec![
        (const_expr!(54665), const_felt252_expr!(78945)),
        (const_expr!(456), const_felt252_expr!(78945)),
        (const_expr!(21321), const_felt252_expr!(78945)),
        (const_expr!(4), const_felt252_expr!(78945)),
        (const_expr!(64356), const_felt252_expr!(78945)),
        (const_expr!(12343), const_felt252_expr!(78945, 0)),
    ];
    let memory = Felt252IdMemory::new_with_data(mem_data.clone());
    let verify_all = MemVerifyAll::<6>::new(memory);
    let (registry, _) = AirFnRegistry::new(&verify_all);
    let (state, _) = registry.run_air(
        &verify_all,
        (),
        (
            mem_data
                .into_iter()
                .map(|(a, _)| CasmAddress::new(a, ""))
                .collect::<Vec<_>>()
                .try_into()
                .expect("Invalid size of array"),
            const_felt252_expr!(78945),
        ),
    );
    expect![[r#"
        (0, "id"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "assertion `left == right` failed")]
fn test_failed_verify_all() {
    let mem_data = vec![
        (const_expr!(54665), const_felt252_expr!(78945)),
        (const_expr!(456), const_felt252_expr!(78945)),
        (const_expr!(21321), const_felt252_expr!(78945)),
        (const_expr!(4), const_felt252_expr!(78945)),
        (const_expr!(64356), const_felt252_expr!(78944)),
        (const_expr!(12343), const_felt252_expr!(78945, 0)),
    ];
    let memory = Felt252IdMemory::new_with_data(mem_data.clone());
    let verify_all = MemVerifyAll::<6>::new(memory);
    let (registry, _) = AirFnRegistry::new(&verify_all);
    registry.run_air(
        &verify_all,
        (),
        (
            mem_data
                .into_iter()
                .map(|(a, _)| CasmAddress::new(a, ""))
                .collect::<Vec<_>>()
                .try_into()
                .expect("Invalid size of array"),
            const_felt252_expr!(78945),
        ),
    );
}
