use indexmap::IndexMap;

use super::super::common::*;
use super::decode_instruction::*;

use crate::airs::memory::felt252_id_memory::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

/// The jump opcode.
/// Implements the Cairo0 instructions:
/// - jump rel imm
/// - jump abs [ap + offset]
/// - jump abs [fp + offset]

#[derive(Clone, Debug)]
pub struct JumpOpcode {
    pub is_rel: bool,
    pub op1_base_fp: bool,
    pub ap_update_add_1: bool,
    pub memory: Felt252IdMemory,
}

impl JumpOpcode {
    pub fn get_flags(&self) -> Flags {
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.op1_base_fp);
            false
        } else {
            !self.op1_base_fp
        };
        Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: Some(self.is_rel),
            op1_base_fp: Some(self.op1_base_fp),
            op1_base_ap: Some(flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.is_rel),
            pc_update_jump_rel: Some(self.is_rel),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: Some(self.ap_update_add_1),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        }
    }
}

impl AirFn for JumpOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset2 = if self.is_rel { Some(1) } else { None };

        // Create the flags.
        let flags = self.get_flags();

        // Check the instruction.
        let ([_, _, offset2], _) = ab.call(
            &DecodeInstruction {
                const_offsets: [Some(-1), Some(-1), offset2],
                const_flags: flags,
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Calculate the next pc
        let next_pc = if self.is_rel {
            pc.clone() + self.memory.read_rel_imm(ab, pc + const_expr!(1))
        } else {
            let mem1_base = if self.op1_base_fp {
                fp.clone()
            } else {
                ap.clone()
            };
            self.memory.read_address(ab, mem1_base + offset2)
        };

        // Calculate the next ap
        let next_ap = if self.ap_update_add_1 {
            ap + const_expr!(1)
        } else {
            ap
        };

        [next_pc, next_ap, fp]
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [
            ("is_rel".to_string(), self.is_rel.to_string()),
            ("op1_base_fp".to_string(), self.op1_base_fp.to_string()),
            (
                "ap_update_add_1".to_string(),
                self.ap_update_add_1.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
