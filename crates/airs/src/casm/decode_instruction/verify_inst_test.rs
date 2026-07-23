use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn_registry::AirFnRegistry;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_felt252_expr};
use expect_test::expect;

use super::decode_inst::*;
use super::verify_inst::*;
use crate::casm::common::*;

#[test]
fn test_verify_inst() {
    let (flags_b, [felt5_high, felt6], offsets_i16, offsets) = init_flags_and_offsets();
    let memory = Felt252IdMemory::new_with_data(vec![(
        const_expr!(0),
        const_felt252_expr!(
            assemble_instruction(
                offsets_i16[0],
                offsets_i16[1],
                offsets_i16[2],
                flags_b,
                OpcodeExtension::Stone
            ),
            0
        ),
    )]);

    let air_fn = VerifyInstruction { memory };
    let (registry, _) = AirFnRegistry::new(&air_fn);

    // Check state
    let (state, _) = registry.run_air(
        &air_fn,
        (),
        (
            CasmAddress::new(const_expr!(0), ""),
            offsets,
            [felt5_high, felt6],
            OpcodeExtension::Stone.into(),
        ),
    );

    expect![[r#"
        (1, "multiplicity_0"),
        (0, "input_pc"),
        (32769, "input_offset0"),
        (32767, "input_offset1"),
        (32770, "input_offset2"),
        (80, "input_inst_felt5_high"),
        (282, "input_inst_felt6"),
        (0, "input_opcode_extension"),
        (1, "offset0_low"),
        (64, "offset0_mid"),
        (3, "offset1_low"),
        (511, "offset1_mid"),
        (15, "offset1_high"),
        (2, "offset2_low"),
        (0, "offset2_mid"),
        (4, "offset2_high"),
        (0, "instruction_id"),
    "#]]
    .assert_eq(&state.to_string());
}

fn init_flags_and_offsets() -> ([bool; 15], [FeltExpr; 2], [i16; 3], [FeltExpr; 3]) {
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
    let [felt5_high, felt6] = DecodeInstruction::flags_to_felts(flags);
    let flags_b =
        named_flags.to_arr().iter().map(|f| f.unwrap()).collect::<Vec<_>>().try_into().unwrap();

    let offsets_i16 = [1, -1, 2];
    let offsets = [
        const_expr!(offset_as_u16(offsets_i16[0]) as u32),
        const_expr!(offset_as_u16(offsets_i16[1]) as u32),
        const_expr!(offset_as_u16(offsets_i16[2]) as u32),
    ];

    (flags_b, [felt5_high, felt6], offsets_i16, offsets)
}
