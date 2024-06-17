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

const _DST_REG_BIT: u16 = 0;
const _OP0_REG_BIT: u16 = 1;
const _OP1_IMM_BIT: u16 = 2;
const _OP1_FP_BIT: u16 = 3;
const _OP1_AP_BIT: u16 = 4;
const _RES_ADD_BIT: u16 = 5;
const _RES_MUL_BIT: u16 = 6;
const _PC_JUMP_ABS_BIT: u16 = 7;
const _PC_JUMP_REL_BIT: u16 = 8;
const _PC_JNZ_BIT: u16 = 9;
const _AP_ADD_BIT: u16 = 10;
const _AP_ADD1_BIT: u16 = 11;
const _OPCODE_CALL_BIT: u16 = 12;
const _OPCODE_RET_BIT: u16 = 13;
const _OPCODE_ASSERT_EQ_BIT: u16 = 14;

pub fn offset_as_u16(offset: i16) -> UInt16Expr {
    const_u16_expr!((offset + (1 << (OFFSET_BITS - 1))) as u16)
}

impl_air_var!([CasmAddress; 3]);
impl_air_var!((CasmState, UInt16Expr, BoolExpr));
