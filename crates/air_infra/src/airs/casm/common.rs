use std::array::from_fn;

use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

// Macros
use crate::const_u16_expr;
use crate::impl_air_var;

pub type CasmAddress = FeltExpr;
// The state is the triple [pc, ap, fp].
pub type CasmState = [CasmAddress; 3];

pub const OFFSET_BITS: u32 = 16;

pub struct NamedFlags {
    pub dst_base_fp: bool,
    pub op0_base_fp: bool,
    pub op1_imm: bool,
    pub op1_base_fp: bool,
    pub op1_base_ap: bool,
    pub res_add: bool,
    pub res_mul: bool,
    pub pc_update_jump: bool,
    pub pc_update_jump_rel: bool,
    pub pc_update_jnz: bool,
    pub ap_update_add: bool,
    pub ap_update_add_1: bool,
    pub opcode_call: bool,
    pub opcode_ret: bool,
    pub opcode_assert_eq: bool,
}

impl From<NamedFlags> for [bool; 15] {
    fn from(flags: NamedFlags) -> [bool; 15] {
        [
            flags.dst_base_fp,
            flags.op0_base_fp,
            flags.op1_imm,
            flags.op1_base_fp,
            flags.op1_base_ap,
            flags.res_add,
            flags.res_mul,
            flags.pc_update_jump,
            flags.pc_update_jump_rel,
            flags.pc_update_jnz,
            flags.ap_update_add,
            flags.ap_update_add_1,
            flags.opcode_call,
            flags.opcode_ret,
            flags.opcode_assert_eq,
        ]
    }
}

pub fn offset_as_u16(offset: i16) -> UInt16Expr {
    const_u16_expr!((offset + (1 << (OFFSET_BITS - 1))) as u16)
}

impl_air_var!([CasmAddress; 3]);
impl_air_var!([UInt16Expr; 3]);
