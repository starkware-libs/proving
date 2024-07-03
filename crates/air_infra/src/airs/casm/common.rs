use std::array::from_fn;
use std::vec;

use crate::core::expressions::bool_expr::*;
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
pub type Flags = [BoolExpr; 15];
pub type Offsets = Vec<UInt16Expr>;

pub const OFFSET_BITS: u32 = 16;

pub struct NamedFlags {
    pub dst_base_fp: BoolExpr,
    pub op0_base_fp: BoolExpr,
    pub op1_imm: BoolExpr,
    pub op1_base_fp: BoolExpr,
    pub op1_base_ap: BoolExpr,
    pub res_add: BoolExpr,
    pub res_mul: BoolExpr,
    pub pc_update_jump: BoolExpr,
    pub pc_update_jump_rel: BoolExpr,
    pub pc_update_jnz: BoolExpr,
    pub ap_update_add: BoolExpr,
    pub ap_update_add_1: BoolExpr,
    pub opcode_call: BoolExpr,
    pub opcode_ret: BoolExpr,
    pub opcode_assert_eq: BoolExpr,
}

impl From<NamedFlags> for [BoolExpr; 15] {
    fn from(flags: NamedFlags) -> [BoolExpr; 15] {
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
impl_air_var!([BoolExpr; 15]);
impl_air_var!([UInt16Expr; 3]);
impl_air_var!(Vec<UInt16Expr>);
impl_air_var!((FeltExpr, Offsets, Flags));
