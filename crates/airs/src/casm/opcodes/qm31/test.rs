use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::state::State;
use air_infra::core::variables::AsProverType;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;
use stwo_cairo_common::prover_types::cpu::{PRIME, QM31};

use super::qm31_add_mul_opcode::*;
use super::qm31_read_reduced::*;
use crate::casm::common::*;

const PRIME128: u128 = PRIME as u128;

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

fn verify_qm31_mul(x_coordinates: [u128; 4], y_coordinates: [u128; 4], xy_coordinates: [u128; 4]) {
    let x_qm31 = QM31::from_u32_unchecked(
        x_coordinates[0] as u32,
        x_coordinates[1] as u32,
        x_coordinates[2] as u32,
        x_coordinates[3] as u32,
    );
    let y_qm31 = QM31::from_u32_unchecked(
        y_coordinates[0] as u32,
        y_coordinates[1] as u32,
        y_coordinates[2] as u32,
        y_coordinates[3] as u32,
    );
    let xy_qm31 = QM31::from_u32_unchecked(
        xy_coordinates[0] as u32,
        xy_coordinates[1] as u32,
        xy_coordinates[2] as u32,
        xy_coordinates[3] as u32,
    );
    assert_eq!(xy_qm31, x_qm31 * y_qm31);
}

fn test_qm31_read_reduced(coordinates: [u128; 4]) -> State {
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

    state
}

fn test_qm31_add_mul_opcode(
    non_consts_flags: [bool; 8],
    offset_values: [i16; 3],
    dst: Felt252Expr,
    op0: Felt252Expr,
    op1: Felt252Expr,
) -> State {
    // Read the non-constant flags
    let [
        flag_dst_base_fp,
        flag_op0_base_fp,
        flag_op1_imm,
        flag_op1_base_fp,
        flag_op1_base_ap,
        flag_res_add,
        flag_res_mul,
        flag_ap_update_add_1,
    ] = non_consts_flags;

    let [offset_dst_val, offset0_val, mut offset1_val] = offset_values;
    if flag_op1_imm {
        offset1_val = 1;
    }

    // Create the air function
    let mut qm31_add_mul_opcode = QM31AddMulOpcode { memory: Felt252IdMemory::default() };

    // Register values at opcode start
    let pc_value = 10;
    let ap_value = 50;
    let fp_value = 100;

    let pc = const_expr!(pc_value);
    let ap = const_expr!(ap_value);
    let fp = const_expr!(fp_value);

    // Create the non-constant flags
    let non_consts_flags = vec![
        flag_dst_base_fp,
        flag_op0_base_fp,
        flag_op1_imm,
        flag_op1_base_fp,
        flag_op1_base_ap,
        flag_res_add,
        flag_res_mul,
        flag_ap_update_add_1,
    ];

    // Fill memory
    let mut memory_values = vec![(
        pc.clone(),
        const_felt252_expr!(
            assemble_instruction(
                offset_dst_val,
                offset0_val,
                offset1_val,
                qm31_add_mul_opcode.get_flags().non_constants_to_arr(&non_consts_flags),
                OpcodeExtension::QM31Operation,
            ),
            0
        ),
    )];
    if flag_dst_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset_dst_val) as u32), dst));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset_dst_val) as u32), dst));
    };
    if flag_op0_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset0_val) as u32), op0));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset0_val) as u32), op0));
    }
    if flag_op1_imm {
        memory_values.push((const_expr!(pc_value + 1), op1));
    } else if flag_op1_base_fp {
        memory_values.push((const_expr!((fp_value as i16 + offset1_val) as u32), op1));
    } else {
        memory_values.push((const_expr!((ap_value as i16 + offset1_val) as u32), op1));
    };
    qm31_add_mul_opcode.memory = Felt252IdMemory::new_with_data(memory_values);

    // Run air function
    let (registry, _) = AirFnRegistry::new(&qm31_add_mul_opcode);
    let (state, next_state) = registry.run_air(
        &qm31_add_mul_opcode,
        (),
        CasmStateVar::new(pc.clone(), ap.clone(), fp.clone()),
    );

    // Check output
    assert_eq!(next_state.fp().calc(), fp.calc());
    if flag_ap_update_add_1 {
        assert_eq!(next_state.ap().calc(), (ap_value + 1).to_string());
    } else {
        assert_eq!(next_state.ap().calc(), ap.calc());
    }
    if flag_op1_imm {
        assert_eq!(next_state.pc().calc(), (pc_value + 2).to_string());
    } else {
        assert_eq!(next_state.pc().calc(), (pc_value + 1).to_string());
    };

    state
}

#[test]
fn test_qm31_read_reduced_ok() {
    let state = test_qm31_read_reduced([PRIME128 - 1, 0x12345678, 0x7fedcba9, 0x1033c4d6]);
    expect![[r#"
        (0, "id"),
        (510, "value_limb_0"),
        (511, "value_limb_1"),
        (511, "value_limb_2"),
        (15, "value_limb_3"),
        (120, "value_limb_4"),
        (43, "value_limb_5"),
        (141, "value_limb_6"),
        (2, "value_limb_7"),
        (425, "value_limb_8"),
        (229, "value_limb_9"),
        (507, "value_limb_10"),
        (15, "value_limb_11"),
        (214, "value_limb_12"),
        (482, "value_limb_13"),
        (12, "value_limb_14"),
        (2, "value_limb_15"),
        (1367680809, "delta_ab_inv"),
        (1892495108, "delta_cd_inv"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "RangeCheck failed on element 1: RangeCheck4 on input 16")]
fn test_qm31_read_reduced_exceeds_31_bits() {
    test_qm31_read_reduced([(1 << 31) - 2, 1 << 31, (1 << 31) - 2, (1 << 31) - 2]);
}

#[test]
#[should_panic(expected = "0 has no inverse")]
fn test_qm31_read_reduced_exactly_prime() {
    test_qm31_read_reduced([314159, PRIME128, 654321, 123456]);
}

#[test]
fn test_qm31_add() {
    let state = test_qm31_add_mul_opcode(
        [true, false, false, false, true, true, false, false],
        [3, 5, 7],
        coordinates_to_packed([
            (1414213562 + 1234567890) % PRIME128,
            (1732050807 + 1414213562) % PRIME128,
            (1618033988 + 1732050807) % PRIME128,
            (1234567890 + 1618033988) % PRIME128,
        ]),
        coordinates_to_packed([1414213562, 1732050807, 1618033988, 1234567890]),
        coordinates_to_packed([1234567890, 1414213562, 1732050807, 1618033988]),
    );
    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32775, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (0, "op1_imm"),
        (0, "op1_base_fp"),
        (1, "res_add"),
        (0, "ap_update_add_1"),
        (100, "mem_dst_base"),
        (50, "mem0_base"),
        (50, "mem1_base"),
        (1, "dst_id"),
        (141, "dst_limb_0"),
        (153, "dst_limb_1"),
        (376, "dst_limb_2"),
        (3, "dst_limb_3"),
        (306, "dst_limb_4"),
        (23, "dst_limb_5"),
        (226, "dst_limb_6"),
        (7, "dst_limb_7"),
        (188, "dst_limb_8"),
        (286, "dst_limb_9"),
        (491, "dst_limb_10"),
        (8, "dst_limb_11"),
        (23, "dst_limb_12"),
        (416, "dst_limb_13"),
        (129, "dst_limb_14"),
        (5, "dst_limb_15"),
        (245220106, "dst_delta_ab_inv"),
        (1966556833, "dst_delta_cd_inv"),
        (2, "op0_id"),
        (442, "op0_limb_0"),
        (407, "op0_limb_1"),
        (274, "op0_limb_2"),
        (10, "op0_limb_3"),
        (375, "op0_limb_4"),
        (127, "op0_limb_5"),
        (463, "op0_limb_6"),
        (12, "op0_limb_7"),
        (324, "op0_limb_8"),
        (158, "op0_limb_9"),
        (28, "op0_limb_10"),
        (12, "op0_limb_11"),
        (210, "op0_limb_12"),
        (257, "op0_limb_13"),
        (101, "op0_limb_14"),
        (9, "op0_limb_15"),
        (415395556, "op0_delta_ab_inv"),
        (647614690, "op0_delta_cd_inv"),
        (3, "op1_id"),
        (210, "op1_limb_0"),
        (257, "op1_limb_1"),
        (101, "op1_limb_2"),
        (9, "op1_limb_3"),
        (442, "op1_limb_4"),
        (407, "op1_limb_5"),
        (274, "op1_limb_6"),
        (10, "op1_limb_7"),
        (375, "op1_limb_8"),
        (127, "op1_limb_9"),
        (463, "op1_limb_10"),
        (12, "op1_limb_11"),
        (324, "op1_limb_12"),
        (158, "op1_limb_13"),
        (28, "op1_limb_14"),
        (12, "op1_limb_15"),
        (1471724291, "op1_delta_ab_inv"),
        (247557051, "op1_delta_cd_inv"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "assertion `left == right` failed: given value != value in memory
  left: Some([M31(3), M31(192), M31(1), M31(240), M31(0), M31(76), M31(256), M31(3), M31(0), \
                           M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), \
                           M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), \
                           M31(0), M31(0), M31(0)])
 right: Some([M31(3), M31(192), M31(1), M31(240), M31(0), M31(76), M31(257), M31(3), M31(0), \
                           M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), \
                           M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), M31(0), \
                           M31(0), M31(0), M31(0)])")]
fn test_qm31_neither_add_res_nor_mul_res() {
    test_qm31_add_mul_opcode(
        [true, false, false, true, false, false, false, false],
        [3, 5, 7],
        coordinates_to_packed([
            (1414213562 + 1234567890) % PRIME128,
            (1732050807 + 1414213562) % PRIME128,
            (1618033988 + 1732050807) % PRIME128,
            (1234567890 + 1618033988) % PRIME128,
        ]),
        coordinates_to_packed([1414213562, 1732050807, 1618033988, 1234567890]),
        coordinates_to_packed([1234567890, 1414213562, 1732050807, 1618033988]),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_qm31_add_both_op1_base_fp_and_op1_imm() {
    test_qm31_add_mul_opcode(
        [false, true, true, true, false, true, false, true],
        [3, 5, 7],
        coordinates_to_packed([
            (1414213562 + 1234567890) % PRIME128,
            (1732050807 + 1414213562) % PRIME128,
            (1618033988 + 1732050807) % PRIME128,
            (1234567890 + 1618033988) % PRIME128,
        ]),
        coordinates_to_packed([1414213562, 1732050807, 1618033988, 1234567890]),
        coordinates_to_packed([1234567890, 1414213562, 1732050807, 1618033988]),
    );
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_qm31_add_not_equal() {
    test_qm31_add_mul_opcode(
        [true, false, false, false, true, true, false, false],
        [3, 5, 7],
        coordinates_to_packed([
            (1414213562 + 1234567890) % PRIME128,
            (1732050807 + 1414213562) % PRIME128,
            (1618033988 + 1732050807 + 1) % PRIME128,
            (1234567890 + 1618033988) % PRIME128,
        ]),
        coordinates_to_packed([1414213562, 1732050807, 1618033988, 1234567890]),
        coordinates_to_packed([1234567890, 1414213562, 1732050807, 1618033988]),
    );
}

#[test]
fn test_qm31_mul() {
    let dst_coordinates = [1061611715, 1937705850, 1725588458, 638022338];
    let op0_coordinates = [1374529783, 2085302751, 630173584, 1752038619];
    let op1_coordinates = [1507396218, 1403862093, 1269472053, 370497316];
    verify_qm31_mul(op0_coordinates, op1_coordinates, dst_coordinates);

    let state = test_qm31_add_mul_opcode(
        [true, false, false, true, false, false, true, false],
        [3, 5, 7],
        coordinates_to_packed(dst_coordinates),
        coordinates_to_packed(op0_coordinates),
        coordinates_to_packed(op1_coordinates),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32775, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (0, "op1_imm"),
        (1, "op1_base_fp"),
        (0, "res_add"),
        (0, "ap_update_add_1"),
        (100, "mem_dst_base"),
        (50, "mem0_base"),
        (100, "mem1_base"),
        (1, "dst_id"),
        (195, "dst_limb_0"),
        (372, "dst_limb_1"),
        (465, "dst_limb_2"),
        (7, "dst_limb_3"),
        (378, "dst_limb_4"),
        (389, "dst_limb_5"),
        (223, "dst_limb_6"),
        (14, "dst_limb_7"),
        (490, "dst_limb_8"),
        (305, "dst_limb_9"),
        (438, "dst_limb_10"),
        (12, "dst_limb_11"),
        (194, "dst_limb_12"),
        (441, "dst_limb_13"),
        (385, "dst_limb_14"),
        (4, "dst_limb_15"),
        (415457614, "dst_delta_ab_inv"),
        (1073079070, "dst_delta_cd_inv"),
        (2, "op0_id"),
        (247, "op0_limb_0"),
        (212, "op0_limb_1"),
        (123, "op0_limb_2"),
        (10, "op0_limb_3"),
        (479, "op0_limb_4"),
        (408, "op0_limb_5"),
        (274, "op0_limb_6"),
        (15, "op0_limb_7"),
        (400, "op0_limb_8"),
        (471, "op0_limb_9"),
        (355, "op0_limb_10"),
        (4, "op0_limb_11"),
        (219, "op0_limb_12"),
        (254, "op0_limb_13"),
        (27, "op0_limb_14"),
        (13, "op0_limb_15"),
        (1038917797, "op0_delta_ab_inv"),
        (1609696011, "op0_delta_cd_inv"),
        (3, "op1_id"),
        (122, "op1_limb_0"),
        (133, "op1_limb_1"),
        (118, "op1_limb_2"),
        (11, "op1_limb_3"),
        (77, "op1_limb_4"),
        (158, "op1_limb_5"),
        (235, "op1_limb_6"),
        (10, "op1_limb_7"),
        (309, "op1_limb_8"),
        (333, "op1_limb_9"),
        (234, "op1_limb_10"),
        (9, "op1_limb_11"),
        (292, "op1_limb_12"),
        (171, "op1_limb_13"),
        (389, "op1_limb_14"),
        (2, "op1_limb_15"),
        (1756722233, "op1_delta_ab_inv"),
        (796687207, "op1_delta_cd_inv"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_qm31_mul_not_equal() {
    let dst_coordinates = [1061611715, 1937705850, 1725588458, 638022338];
    let op0_coordinates = [1374529783, 2085302751, 630173584, 1752038619];
    let op1_coordinates = [1507396218, 1403862093, 1269472053, 370497316];
    verify_qm31_mul(op0_coordinates, op1_coordinates, dst_coordinates);

    test_qm31_add_mul_opcode(
        [true, false, false, true, false, false, true, false],
        [3, 5, 7],
        coordinates_to_packed([
            dst_coordinates[0],
            dst_coordinates[1] + 1,
            dst_coordinates[2],
            dst_coordinates[3],
        ]),
        coordinates_to_packed(op0_coordinates),
        coordinates_to_packed(op1_coordinates),
    );
}

#[test]
fn test_qm31_mul_imm() {
    let dst_coordinates = [947980980, 1510986506, 623360030, 1260310989];
    let op0_coordinates = [1414213562, 1732050807, 1618033988, 1234567890];
    let op1_coordinates = [1259921049, 1442249570, 1847759065, 2094551481];
    verify_qm31_mul(op0_coordinates, op1_coordinates, dst_coordinates);

    let state = test_qm31_add_mul_opcode(
        [true, false, true, false, false, false, true, false],
        [3, 5, 7],
        coordinates_to_packed(dst_coordinates),
        coordinates_to_packed(op0_coordinates),
        coordinates_to_packed(op1_coordinates),
    );

    expect![[r#"
        (1, "enabler"),
        (10, "input_pc"),
        (50, "input_ap"),
        (100, "input_fp"),
        (32771, "offset0"),
        (32773, "offset1"),
        (32769, "offset2"),
        (1, "dst_base_fp"),
        (0, "op0_base_fp"),
        (1, "op1_imm"),
        (0, "op1_base_fp"),
        (0, "res_add"),
        (0, "ap_update_add_1"),
        (100, "mem_dst_base"),
        (50, "mem0_base"),
        (10, "mem1_base"),
        (1, "dst_id"),
        (180, "dst_limb_0"),
        (133, "dst_limb_1"),
        (32, "dst_limb_2"),
        (7, "dst_limb_3"),
        (266, "dst_limb_4"),
        (489, "dst_limb_5"),
        (131, "dst_limb_6"),
        (11, "dst_limb_7"),
        (30, "dst_limb_8"),
        (476, "dst_limb_9"),
        (329, "dst_limb_10"),
        (4, "dst_limb_11"),
        (461, "dst_limb_12"),
        (360, "dst_limb_13"),
        (199, "dst_limb_14"),
        (9, "dst_limb_15"),
        (1418485604, "dst_delta_ab_inv"),
        (1021273146, "dst_delta_cd_inv"),
        (2, "op0_id"),
        (442, "op0_limb_0"),
        (407, "op0_limb_1"),
        (274, "op0_limb_2"),
        (10, "op0_limb_3"),
        (375, "op0_limb_4"),
        (127, "op0_limb_5"),
        (463, "op0_limb_6"),
        (12, "op0_limb_7"),
        (324, "op0_limb_8"),
        (158, "op0_limb_9"),
        (28, "op0_limb_10"),
        (12, "op0_limb_11"),
        (210, "op0_limb_12"),
        (257, "op0_limb_13"),
        (101, "op0_limb_14"),
        (9, "op0_limb_15"),
        (415395556, "op0_delta_ab_inv"),
        (647614690, "op0_delta_cd_inv"),
        (3, "op1_id"),
        (153, "op1_limb_0"),
        (111, "op1_limb_1"),
        (198, "op1_limb_2"),
        (9, "op1_limb_3"),
        (354, "op1_limb_4"),
        (381, "op1_limb_5"),
        (381, "op1_limb_6"),
        (10, "op1_limb_7"),
        (217, "op1_limb_8"),
        (328, "op1_limb_9"),
        (392, "op1_limb_10"),
        (13, "op1_limb_11"),
        (441, "op1_limb_12"),
        (40, "op1_limb_13"),
        (310, "op1_limb_14"),
        (15, "op1_limb_15"),
        (128080545, "op1_delta_ab_inv"),
        (1111409758, "op1_delta_cd_inv"),
    "#]]
    .assert_eq(&state.to_string());
}

#[test]
#[should_panic(expected = "Added incorrect constraint (does not evaluate to 0)")]
fn test_qm31_mul_imm_not_equal() {
    let dst_coordinates = [947980980, 1510986506, 623360030, 1260310989];
    let op0_coordinates = [1414213562, 1732050807, 1618033988, 1234567890];
    let op1_coordinates = [1259921049, 1442249570, 1847759065, 2094551481];
    verify_qm31_mul(op0_coordinates, op1_coordinates, dst_coordinates);

    test_qm31_add_mul_opcode(
        [true, false, true, false, false, false, true, false],
        [3, 5, 7],
        coordinates_to_packed([
            dst_coordinates[0],
            dst_coordinates[1],
            dst_coordinates[2],
            dst_coordinates[3] + 1,
        ]),
        coordinates_to_packed(op0_coordinates),
        coordinates_to_packed(op1_coordinates),
    );
}
