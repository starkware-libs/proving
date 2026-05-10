use air_infra::const_expr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use serde::Serialize;

pub const OFFSET_BITS: u32 = 16;

pub const FLAG_DST_BASE_FP_INDEX: usize = 0;
pub const FLAG_OP0_BASE_FP_INDEX: usize = 1;
pub const FLAG_OP1_IMM_INDEX: usize = 2;
pub const FLAG_OP1_BASE_FP_INDEX: usize = 3;
pub const FLAG_OP1_BASE_AP_INDEX: usize = 4;
pub const FLAG_RES_ADD_INDEX: usize = 5;
pub const FLAG_RES_MUL_INDEX: usize = 6;
pub const FLAG_PC_UPDATE_JUMP_INDEX: usize = 7;
pub const FLAG_PC_UPDATE_JUMP_REL_INDEX: usize = 8;
pub const FLAG_PC_UPDATE_JNZ_INDEX: usize = 9;
pub const FLAG_AP_UPDATE_ADD_INDEX: usize = 10;
pub const FLAG_AP_UPDATE_ADD_1_INDEX: usize = 11;
pub const FLAG_OPCODE_CALL_INDEX: usize = 12;
pub const FLAG_OPCODE_RET_INDEX: usize = 13;
pub const FLAG_OPCODE_ASSERT_EQ_INDEX: usize = 14;

pub const FLAG_NAMES: [&str; 15] = [
    "dst_base_fp",
    "op0_base_fp",
    "op1_imm",
    "op1_base_fp",
    "op1_base_ap",
    "res_add",
    "res_mul",
    "pc_update_jump",
    "pc_update_jump_rel",
    "pc_update_jnz",
    "ap_update_add",
    "ap_update_add_1",
    "opcode_call",
    "opcode_ret",
    "opcode_assert_eq",
];

#[derive(Clone, Debug, Copy, Serialize)]
pub enum OpcodeExtension {
    Stone,
    Blake,
    BlakeFinalize,
    QM31Operation,
}

impl From<OpcodeExtension> for FeltExpr {
    fn from(op: OpcodeExtension) -> Self {
        match op {
            OpcodeExtension::Stone => const_expr!(0),
            OpcodeExtension::Blake => const_expr!(1),
            OpcodeExtension::BlakeFinalize => const_expr!(2),
            OpcodeExtension::QM31Operation => const_expr!(3),
        }
    }
}

#[derive(Clone, Debug, Default, Serialize)]
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
    pub fn to_arr(&self) -> [Option<bool>; 15] {
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

    #[cfg(test)]
    pub fn non_constants_to_arr(&self, non_consts_flags: &[bool]) -> [bool; 15] {
        let mut non_consts_flags_iter = non_consts_flags.iter();
        self.to_arr()
            .iter()
            .map(|f| f.unwrap_or_else(|| *non_consts_flags_iter.next().unwrap()))
            .collect::<Vec<_>>()
            .try_into()
            .unwrap()
    }

    #[cfg(test)]
    pub fn from_arr(arr: [Option<bool>; 15]) -> Self {
        Self {
            dst_base_fp: arr[0],
            op0_base_fp: arr[1],
            op1_imm: arr[2],
            op1_base_fp: arr[3],
            op1_base_ap: arr[4],
            res_add: arr[5],
            res_mul: arr[6],
            pc_update_jump: arr[7],
            pc_update_jump_rel: arr[8],
            pc_update_jnz: arr[9],
            ap_update_add: arr[10],
            ap_update_add_1: arr[11],
            opcode_call: arr[12],
            opcode_ret: arr[13],
            opcode_assert_eq: arr[14],
        }
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

pub fn offset_as_signed(offset: FeltExpr) -> FeltExpr {
    offset - const_expr!(1 << (OFFSET_BITS - 1))
}

#[cfg(test)]
pub fn assemble_instruction(
    off_0: i16,
    off_1: i16,
    off_2: i16,
    flags: [bool; 15],
    opcode_extension: OpcodeExtension,
) -> u128 {
    let mut flags_int: u128 = 0;
    for (idx, flag) in flags.iter().enumerate() {
        flags_int += (*flag as u128) << idx;
    }
    let biased_off_0: u128 = offset_as_u16(off_0) as u128;
    let biased_off_1: u128 = offset_as_u16(off_1) as u128;
    let biased_off_2: u128 = offset_as_u16(off_2) as u128;
    ((opcode_extension as u128) << 63)
        + (flags_int << 48)
        + (biased_off_2 << 32)
        + (biased_off_1 << 16)
        + biased_off_0
}
