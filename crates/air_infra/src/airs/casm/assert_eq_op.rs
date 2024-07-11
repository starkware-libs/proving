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

/// The assert_eq opcode.
/// Implements the Cairo0 instructions:
/// - [fp + offset] / [ap + offset] = [ap + offset] / [fp + offset]/ [offset + [ap/fp + offset]] / imm

#[derive(Clone, Debug)]
pub struct AssertEqOpcode {
    pub flag_dst_base_fp: bool,
    pub flag_op0_base_fp: bool,
    pub flag_op1_imm: bool,
    pub flag_op1_base_fp: bool,
    pub flag_op1_base_ap: bool,
    pub flag_ap_update_add_1: bool,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for AssertEqOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        let double_deref = !self.flag_op1_imm && !self.flag_op1_base_fp && !self.flag_op1_base_ap;

        // Create the constant offsets.
        let offset1 = if double_deref {
            None
        } else {
            Some(offset_as_u16(-1))
        };
        let offset2 = if self.flag_op1_imm {
            Some(offset_as_u16(1))
        } else {
            None
        };

        // Create the flags.
        let flags = Flags {
            dst_base_fp: Some(self.flag_dst_base_fp),
            op0_base_fp: Some(self.flag_op0_base_fp),
            op1_imm: Some(self.flag_op1_imm),
            op1_base_fp: Some(self.flag_op1_base_fp),
            op1_base_ap: Some(self.flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(false),
            pc_update_jump_rel: Some(false),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: Some(self.flag_ap_update_add_1),
            opcode_call: Some(false),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(true),
        };

        // Check the instruction.
        let [offset0, offset1, offset2] = ab.call(
            &CheckInstruction {
                const_offsets: [None, offset1, offset2],
                const_flags: flags,
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Fetch op0
        let mem_dst_base = if self.flag_dst_base_fp {
            fp.clone()
        } else {
            ap.clone()
        };
        let key = mem_dst_base + offset0;
        let mut op0_value = ab.get_from_memory(&self.memory, &key);
        let op0 = ab.deduce(op0_value.as_felts_mut()[0]);
        ab.set_in_memory(&self.memory, key, Felt252Expr::from(vec![op0.clone()]));

        // Fetch op1
        let op1 = if double_deref {
            let mem0_base = if self.flag_op0_base_fp {
                fp.clone()
            } else {
                ap.clone()
            };
            let key = mem0_base + offset1;
            let mut op1_value = ab.get_from_memory(&self.memory, &key);
            let op1 = ab.deduce(op1_value.as_felts_mut()[0]);
            ab.set_in_memory(&self.memory, key, Felt252Expr::from(vec![op1.clone()]));
            Some(op1)
        } else {
            None
        };

        // Fetch op2
        let mem1_base = if double_deref {
            op1.unwrap()
        } else if self.flag_op1_imm {
            assert!(!self.flag_op1_base_fp);
            assert!(!self.flag_op1_base_ap);
            pc.clone()
        } else if self.flag_op1_base_ap {
            assert!(!self.flag_op1_base_fp);
            ap.clone()
        } else {
            fp.clone()
        };
        let key = mem1_base.clone() + offset2.clone();
        let mut op2_value = ab.get_from_memory(&self.memory, &key);
        let op2 = ab.deduce(op2_value.as_felts_mut()[0]);
        ab.set_in_memory(&self.memory, key, Felt252Expr::from(vec![op2.clone()]));

        // Calculate the next ap
        let next_ap = if self.flag_ap_update_add_1 {
            ap + const_expr!(1)
        } else {
            ap
        };

        // Assert that op0 == op2
        ab.constrain(op0 - op2);

        [pc, next_ap, fp]
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            (
                "flag_dst_base_fp".to_string(),
                self.flag_dst_base_fp.to_string(),
            ),
            (
                "flag_op0_base_fp".to_string(),
                self.flag_op0_base_fp.to_string(),
            ),
            ("flag_op1_imm".to_string(), self.flag_op1_imm.to_string()),
            (
                "flag_op1_base_fp".to_string(),
                self.flag_op1_base_fp.to_string(),
            ),
            (
                "flag_op1_base_ap".to_string(),
                self.flag_op1_base_ap.to_string(),
            ),
            (
                "flag_ap_update_add_1".to_string(),
                self.flag_ap_update_add_1.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for AssertEqOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
