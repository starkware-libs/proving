use crate::core::expressions::felt_expr::*;

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

pub fn offset_as_u16(offset: i16) -> u16 {
    ((offset as i32) + (1 << (OFFSET_BITS - 1))) as u16
}

pub fn assemble_instruction(off_0: i16, off_1: i16, off_2: i16, flags: [bool; 15]) -> u64 {
    let mut flags_int: u64 = 0;
    for (idx, flag) in flags.iter().enumerate() {
        flags_int += (*flag as u64) << idx;
    }
    let biased_off_0: u64 = offset_as_u16(off_0) as u64;
    let biased_off_1: u64 = offset_as_u16(off_1) as u64;
    let biased_off_2: u64 = offset_as_u16(off_2) as u64;
    (flags_int << 48) + (biased_off_2 << 32) + (biased_off_1 << 16) + biased_off_0
}
