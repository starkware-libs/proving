use std::collections::BTreeSet;

use air_common::TraceType;
use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::range_check::range_check;
use air_infra::{const_expr, const_u32_expr};
use serde::Serialize;

use super::super::decode_instruction::decode_inst::*;
use crate::casm::common::*;

/// The add ap opcode.
/// Implements the Cairo0 instructions:
/// - ap += imm
/// - ap += [fp/ap + offset]
#[derive(Clone, Debug, Serialize)]
pub struct AddApOpcode {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AddApOpcode {
    pub fn get_flags(&self) -> Flags {
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: None,
            op1_base_fp: None,
            op1_base_ap: None,
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(true),
            ap_update_add_1: Some(false),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for AddApOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        // Decode the instruction.
        let flag_sets_of_sum_1 = BTreeSet::from([BTreeSet::from([
            FLAG_OP1_IMM_INDEX,
            FLAG_OP1_BASE_FP_INDEX,
            FLAG_OP1_BASE_AP_INDEX,
        ])]);
        let ([_, _, offset2], flags, _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), Some(-1), None],
                const_flags: self.get_flags(),
                const_opcode_extension: Some(OpcodeExtension::Stone),
                flag_sets_of_sum_1,
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        let flag_op1_imm = flags[FLAG_OP1_IMM_INDEX].clone();
        let flag_op1_base_fp = flags[FLAG_OP1_BASE_FP_INDEX].clone();
        let flag_op1_base_ap = flags[FLAG_OP1_BASE_AP_INDEX].clone();

        ab.constrain(
            flag_op1_imm.clone() * (const_expr!(1) - offset2.clone()),
            "if imm then offset2 is 1",
        );

        let mem1_base = ab.assign(
            &mut (flag_op1_imm.clone() * casm_state.pc().var
                + flag_op1_base_fp * casm_state.fp().var
                + flag_op1_base_ap * casm_state.ap().var),
            "mem1_base",
        );

        let op1 = self.memory.read_rel_imm(ab, CasmAddress::new(mem1_base + offset2, "op1"));

        let next_ap = ab.let_(casm_state.ap().var + op1, "next_ap");

        ab.call(&RangeCheck29 {}, next_ap.clone());

        CasmStateVar::new(
            casm_state.pc().var + (const_expr!(1) + flag_op1_imm),
            next_ap,
            casm_state.fp().var,
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}

/// Inlined AirFn which verifies that ap (FeltExpr) is in the range [0, 2^29).
/// ap must be a linear expression with respect to the component's columns.
/// Note: this range corresponds to the address space.
#[derive(Clone, Debug, Serialize)]
pub struct RangeCheck29 {}

impl AirFn for RangeCheck29 {
    type ExtIn = ();
    type In = FeltExpr;
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), x: Self::In) -> Self::Out {
        let x_u32 = UInt32Expr::from(x.clone());
        let x_bot11bits_u32 =
            ab.let_for_deduction(x_u32 & const_u32_expr!(0x7FF), "range_check_29_bot11bits_u32");

        let x_bot11bits =
            ab.deduce(&mut x_bot11bits_u32.low().as_felt(), "range_check_29_bot11bits");
        let x_top18bits = (x.clone() - x_bot11bits.clone()) / const_expr!(1 << 11);

        range_check(ab, &[18], &[x_top18bits]);
        range_check(ab, &[11], &[x_bot11bits]);
    }
}
