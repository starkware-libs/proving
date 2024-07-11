use std::fmt::Display;

use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;

// Macros
use crate::const_expr;

pub type CasmAddress = FeltExpr;
// The state is the triple [pc, ap, fp].
pub type CasmState = [CasmAddress; 3];

pub const OFFSET_BITS: u32 = 16;

#[derive(Clone, Debug)]
pub struct Flags {
    pub dst_base_fp: Option<bool>,
    pub op0_base_fp: Option<bool>,
    pub op1_imm: Option<bool>,
    pub op1_base_fp: Option<bool>,
    pub op1_base_ap: Option<bool>,
    pub res_add: Option<bool>,
    pub res_mul: Option<bool>,
    pub pc_update_jump: Option<bool>,
    pub pc_update_jump_rel: Option<bool>,
    pub pc_update_jnz: Option<bool>,
    pub ap_update_add: Option<bool>,
    pub ap_update_add_1: Option<bool>,
    pub opcode_call: Option<bool>,
    pub opcode_ret: Option<bool>,
    pub opcode_assert_eq: Option<bool>,
}

impl Flags {
    pub fn sum_consts(&self, from: usize, to: usize) -> FeltExpr {
        const_expr!(self.to_arr()[from..to]
            .iter()
            .enumerate()
            .filter_map(|(i, f)| f.map(|b| (b as u32) << (i as u32)))
            .sum())
    }

    pub fn get_non_consts_indices(&self) -> Vec<usize> {
        self.to_arr()
            .iter()
            .enumerate()
            .filter_map(|(i, f)| if f.is_none() { Some(i) } else { None })
            .collect()
    }

    fn to_arr(&self) -> [Option<bool>; 15] {
        [
            self.dst_base_fp,
            self.op0_base_fp,
            self.op1_imm,
            self.op1_base_fp,
            self.op1_base_ap,
            self.res_add,
            self.res_mul,
            self.pc_update_jump,
            self.pc_update_jump_rel,
            self.pc_update_jnz,
            self.ap_update_add,
            self.ap_update_add_1,
            self.opcode_call,
            self.opcode_ret,
            self.opcode_assert_eq,
        ]
    }
}

impl Display for Flags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let flags = self.to_arr();
        write!(
            f,
            "Flags {{ {} }}",
            flags
                .iter()
                .enumerate()
                .filter_map(|(i, f)| f.map(|b| format!("{}: {}", i, b)))
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

#[cfg(test)]
impl From<Flags> for [bool; 15] {
    fn from(flags: Flags) -> [bool; 15] {
        flags.to_arr().map(|f| f.unwrap())
    }
}

pub fn offset_as_u16(offset: i16) -> u16 {
    ((offset as i32) + (1 << (OFFSET_BITS - 1))) as u16
}

#[cfg(test)]
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
