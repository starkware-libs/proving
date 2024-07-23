use std::collections::BTreeMap;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::variables::*;

use super::check_instruction::*;
use super::common::*;

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
    pub flag_op1_base_fp: bool,
    pub flag_ap_update_add_1: bool,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for JumpOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset0 = offset_as_u16(-1);
        let offset1 = offset_as_u16(-1);
        let offset2 = if self.is_rel {
            Some(offset_as_u16(1))
        } else {
            None
        };

        // Create the flags.
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.flag_op1_base_fp);
            false
        } else {
            !self.flag_op1_base_fp
        };
        let flags = Flags {
            dst_base_fp: Some(true),
            op0_base_fp: Some(true),
            op1_imm: Some(self.is_rel),
            op1_base_fp: Some(self.flag_op1_base_fp),
            op1_base_ap: Some(flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.is_rel),
            pc_update_jump_rel: Some(self.is_rel),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: Some(self.flag_ap_update_add_1),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        };

        // Check the instruction.
        let ([_, _, offset2], _) = ab.call(
            &CheckInstruction {
                const_offsets: [Some(offset0), Some(offset1), offset2],
                const_flags: flags,
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Fetch op1.
        let mem1_base = if self.is_rel {
            pc.clone()
        } else if self.flag_op1_base_fp {
            fp.clone()
        } else {
            ap.clone()
        };
        let key = mem1_base + offset_as_signed(offset2);
        let mut op1_value = ab.get_from_memory(&self.memory, &key);
        let op1 = ab.deduce(op1_value.as_felts_mut()[0]);
        ab.set_in_memory(&self.memory, key, Felt252Expr::from(vec![op1.clone()]));

        // Calculate the next pc
        let next_pc = if self.is_rel { pc + op1 } else { op1 };

        // Calculate the next ap
        let next_ap = if self.flag_ap_update_add_1 {
            ap + const_expr!(1)
        } else {
            ap
        };

        [next_pc, next_ap, fp]
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            ("is_rel".to_string(), self.is_rel.to_string()),
            (
                "flag_op1_base_fp".to_string(),
                self.flag_op1_base_fp.to_string(),
            ),
            (
                "flag_ap_update_add".to_string(),
                self.flag_ap_update_add_1.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for JumpOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
