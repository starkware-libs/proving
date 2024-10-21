use compiled_casm_air::utils::JSONS_LOOKUPS_DIR;

use super::verify_inst::*;

use crate::airs::casm::common::*;
use crate::airs::felt252_id_memory::memory::*;
use crate::core::air_fn_registry::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::utils::test_utils::*;

// Macro
use crate::const_expr;
use crate::const_felt252_expr;

#[test]
fn test_verify_inst() {
    let (flags_b, flags, offsets_i16, offsets) = init_flags_and_offsets();
    let memory = Felt252IdMemory::new_with_data(vec![(
        const_expr!(0),
        const_felt252_expr!(
            assemble_instruction(offsets_i16[0], offsets_i16[1], offsets_i16[2], flags_b) as u128,
            0
        ),
    )]);

    let air_fn = VerifyInstruction { memory };
    let (registry, entry) = AirFnRegistry::new(&air_fn);

    // Check entry
    let name = entry.name.to_lowercase();
    compare_json(
        &entry.compile(),
        &format!("{}{}.json", JSONS_LOOKUPS_DIR, name),
    );

    // Check state
    let (state, _) = registry.run_air(&air_fn, (const_expr!(0), offsets, flags));
    let expected_state = vec![
        (0, ""),     // pc
        (32769, ""), // offset_0
        (32767, ""), // offset_1
        (32770, ""), // offset_2
        (0, ""),     // dst_base_fp
        (1, ""),     // op0_base_fp
        (0, ""),     // op1_imm
        (1, ""),     // op1_base_fp
        (0, ""),     // op1_base_ap
        (0, ""),     // res_add
        (0, ""),     // res_mul
        (1, ""),     // pc_update_jump
        (0, ""),     // pc_update_jump_rel
        (1, ""),     // pc_update_jnz
        (1, ""),     // ap_update_add
        (0, ""),     // ap_update_add_1
        (0, ""),     // opcode_call
        (0, ""),     // opcode_ret
        (1, ""),     // opcode_assert_eq
        (1, "offset0_low"),
        (64, "offset0_mid"),
        (3, "offset1_low"),
        (511, "offset1_mid"),
        (15, "offset1_high"),
        (2, "offset2_low"),
        (0, "offset2_mid"),
        (4, "offset2_high"),
        (0, "id"),
    ]
    .into();
    assert!(
        state == expected_state,
        "State {} does not match {}",
        state,
        expected_state
    );
}

fn init_flags_and_offsets() -> ([bool; 15], [FeltExpr; 15], [i16; 3], [FeltExpr; 3]) {
    let named_flags = Flags {
        dst_base_fp: Some(false),
        op0_base_fp: Some(true),
        op1_imm: Some(false),
        op1_base_fp: Some(true),
        op1_base_ap: Some(false),
        res_add: Some(false),
        res_mul: Some(false),
        pc_update_jump: Some(true),
        pc_update_jump_rel: Some(false),
        pc_update_jnz: Some(true),
        ap_update_add: Some(true),
        ap_update_add_1: Some(false),
        opcode_call: Some(false),
        opcode_ret: Some(false),
        opcode_assert_eq: Some(true),
    };
    let flags = named_flags
        .to_arr()
        .iter()
        .map(|f| const_expr!(f.unwrap() as u32))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    let flags_b = named_flags
        .to_arr()
        .iter()
        .map(|f| f.unwrap())
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();

    let offsets_i16 = [1, -1, 2];
    let offsets = [
        const_expr!(offset_as_u16(offsets_i16[0]) as u32),
        const_expr!(offset_as_u16(offsets_i16[1]) as u32),
        const_expr!(offset_as_u16(offsets_i16[2]) as u32),
    ];

    (flags_b, flags, offsets_i16, offsets)
}
