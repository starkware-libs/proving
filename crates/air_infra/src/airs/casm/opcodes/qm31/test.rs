use super::qm31_read_reduced::*;
use crate::airs::casm::casm_state::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::state::*;
use crate::core::variables::*;
use crate::utils::test_utils::*;
use crate::{const_expr, const_felt252_expr};

/// Packs 36 bit coordinates into a 252 bit Felt252Expr.
fn coordinates_to_packed(coordinates: [u128; 4]) -> Felt252Expr {
    for &coordinate in coordinates.iter() {
        assert!(coordinate < (1 << 36));
    }
    let coordinate3_a = coordinates[3] & ((1 << 20) - 1);
    let coordinate3_b = coordinates[3] >> 20;
    let low =
        coordinates[0] + (coordinates[1] << 36) + (coordinates[2] << 72) + (coordinate3_a << 108);
    const_felt252_expr!(low, coordinate3_b)
}

fn test_qm31_read_reduced(coordinates: [u128; 4], expected_state: State) {
    // Pack the coordinates into a Felt252Expr and store it in the memory.
    let mem_data = vec![(const_expr!(1), coordinates_to_packed(coordinates))];
    let memory = Felt252IdMemory::new_with_data(mem_data);

    // Create the air function and run it.
    let qm31_read_reduced = QM31ReadReduced { memory };
    let (registry, _) = AirFnRegistry::new(&qm31_read_reduced);
    let (state, output) =
        registry.run_air(&qm31_read_reduced, (), CasmAddress::new(const_expr!(1), ""));

    // Check the output and the state.
    for (i, &coordinate) in coordinates.iter().enumerate() {
        assert_eq!(output.0[i].calc(), coordinate.to_string());
    }
    assert_expected_state(&state, &expected_state);
}

#[test]
fn test_qm31_read_reduced_ok() {
    test_qm31_read_reduced(
        [(1u128 << 31) - 2, 0x12345678, 0x7fedcba9, 0x1033c4d6],
        vec![
            (0, "id"),
            (510, "limb_0"),
            (511, "limb_1"),
            (511, "limb_2"),
            (0xf, "limb_3"),
            (0x78, "limb_4"),
            (0x2b, "limb_5"),
            (0x8d, "limb_6"),
            (2, "limb_7"),
            (0x1a9, "limb_8"),
            (0xe5, "limb_9"),
            (0x1fb, "limb_10"),
            (0xf, "limb_11"),
            (0xd6, "limb_12"),
            (0x1e2, "limb_13"),
            (0xc, "limb_14"),
            (2, "limb_15"),
            (1367680809, "delta_ab_inv"),
            (1892495108, "delta_cd_inv"),
        ]
        .into(),
    );
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 1: RangeCheck4 on input 16")]
fn test_qm31_read_reduced_exceeds_31_bits() {
    test_qm31_read_reduced(
        [(1 << 31) - 2, 1 << 31, (1 << 31) - 2, (1 << 31) - 2],
        vec![].into(),
    );
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_qm31_read_reduced_exactly_prime() {
    test_qm31_read_reduced([314159, (1 << 31) - 1, 654321, 123456], vec![].into());
}
