use std::array::from_fn;

use crate::core::expressions::bool_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::expressions::uint64_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;
// Macros
use crate::const_u16_expr;
use crate::impl_air_var;

pub type CasmAddress = FeltExpr;
// The state is the triple [pc, ap, fp].
pub type CasmState = [CasmAddress; 3];
pub type Instruction = UInt64Expr;

pub const OFFSET_BITS: u32 = 16;

const DST_REG_BIT: u16 = 0;
const OP0_REG_BIT: u16 = 1;
const OP1_IMM_BIT: u16 = 2;
const OP1_FP_BIT: u16 = 3;
const OP1_AP_BIT: u16 = 4;
const RES_ADD_BIT: u16 = 5;
const RES_MUL_BIT: u16 = 6;
const PC_JUMP_ABS_BIT: u16 = 7;
const PC_JUMP_REL_BIT: u16 = 8;
const PC_JNZ_BIT: u16 = 9;
const AP_ADD_BIT: u16 = 10;
const AP_ADD1_BIT: u16 = 11;
const OPCODE_CALL_BIT: u16 = 12;
const OPCODE_RET_BIT: u16 = 13;
const OPCODE_ASSERT_EQ_BIT: u16 = 14;

#[allow(clippy::too_many_arguments)]
pub fn opcode_from_flags(
    flag_dst_base_fp: BoolExpr,
    flag_op0_base_fp: BoolExpr,
    flag_op1_imm: BoolExpr,
    flag_op1_base_fp: BoolExpr,
    flag_op1_base_ap: BoolExpr,
    flag_res_add: BoolExpr,
    flag_res_mul: BoolExpr,
    flag_pc_update_jump: BoolExpr,
    flag_pc_update_jump_rel: BoolExpr,
    flag_pc_update_jnz: BoolExpr,
    flag_ap_update_add: BoolExpr,
    flag_ap_update_add_1: BoolExpr,
    flag_opcode_call: BoolExpr,
    flag_opcode_ret: BoolExpr,
    flag_opcode_assert_eq: BoolExpr,
) -> UInt16Expr {
    &(&(&(&(&UInt16Expr::from(flag_dst_base_fp) << &const_u16_expr!(DST_REG_BIT))
        + &(&UInt16Expr::from(flag_op0_base_fp) << &const_u16_expr!(OP0_REG_BIT)))
        + &(&(&UInt16Expr::from(flag_op1_imm) << &const_u16_expr!(OP1_IMM_BIT))
            + &(&UInt16Expr::from(flag_op1_base_fp) << &const_u16_expr!(OP1_FP_BIT))))
        + &(&(&(&UInt16Expr::from(flag_op1_base_ap) << &const_u16_expr!(OP1_AP_BIT))
            + &(&UInt16Expr::from(flag_res_add) << &const_u16_expr!(RES_ADD_BIT)))
            + &(&(&UInt16Expr::from(flag_res_mul) << &const_u16_expr!(RES_MUL_BIT))
                + &(&UInt16Expr::from(flag_pc_update_jump) << &const_u16_expr!(PC_JUMP_ABS_BIT)))))
        + &(&(&(&(&UInt16Expr::from(flag_pc_update_jump_rel)
            << &const_u16_expr!(PC_JUMP_REL_BIT))
            + &(&UInt16Expr::from(flag_pc_update_jnz) << &const_u16_expr!(PC_JNZ_BIT)))
            + &(&(&UInt16Expr::from(flag_ap_update_add) << &const_u16_expr!(AP_ADD_BIT))
                + &(&UInt16Expr::from(flag_ap_update_add_1) << &const_u16_expr!(AP_ADD1_BIT))))
            + &(&(&(&UInt16Expr::from(flag_opcode_call) << &const_u16_expr!(OPCODE_CALL_BIT))
                + &(&UInt16Expr::from(flag_opcode_ret) << &const_u16_expr!(OPCODE_RET_BIT)))
                + &(&UInt16Expr::from(flag_opcode_assert_eq)
                    << &const_u16_expr!(OPCODE_ASSERT_EQ_BIT))))
}

pub fn offset_as_u16(offset: i16) -> UInt16Expr {
    const_u16_expr!((offset + (1 << (OFFSET_BITS - 1))) as u16)
}

impl_air_var!([CasmAddress; 3]);
impl_air_var!((CasmState, UInt16Expr));
