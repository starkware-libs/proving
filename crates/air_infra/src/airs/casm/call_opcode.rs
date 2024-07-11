use std::collections::BTreeMap;

use super::check_instruction::*;
use super::common::*;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

// Macros
use crate::const_expr;

/// The call opcode.
/// Implements the Cairo0 instructions:
/// - call rel imm
/// - call abs [ap + offset]
/// - call abs [fp + offset]

#[derive(Clone, Debug)]
pub struct CallOpcode {
    is_rel: bool,
    flag_op1_base_fp: bool,

    memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for CallOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset0 = offset_as_u16(0);
        let offset1 = offset_as_u16(1);
        let offset2 = if self.is_rel {
            Some(offset_as_u16(1))
        } else {
            None
        };

        // Create the constant flags.
        let flag_op1_imm = self.is_rel;
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.flag_op1_base_fp);
            false
        } else {
            !self.flag_op1_base_fp
        };
        let flags = Flags {
            dst_base_fp: Some(false),
            op0_base_fp: Some(false),
            op1_imm: Some(flag_op1_imm),
            op1_base_fp: Some(self.flag_op1_base_fp),
            op1_base_ap: Some(flag_op1_base_ap),
            res_add: Some(false),
            res_mul: Some(false),
            pc_update_jump: Some(!self.is_rel),
            pc_update_jump_rel: Some(self.is_rel),
            pc_update_jnz: Some(false),
            ap_update_add: Some(false),
            ap_update_add_1: Some(false),
            opcode_call: Some(true),
            opcode_ret: Some(false),
            opcode_assert_eq: Some(false),
        };

        // Check the instruction.
        let [_, _, offset2] = ab.call(
            &CheckInstruction {
                const_offsets: [Some(offset0), Some(offset1), offset2],
                const_flags: flags,
                memory: self.memory.clone(),
            },
            pc.clone(),
        );

        // Push fp.
        ab.set_in_memory(
            &self.memory,
            ap.clone(),
            Felt252Expr::from(vec![fp.clone()]),
        );

        // Push pc + instruction_size.
        ab.set_in_memory(
            &self.memory,
            ap.clone() + const_expr!(1),
            Felt252Expr::from(vec![(pc.clone() + const_expr!(1 + (self.is_rel as u32)))]),
        );

        // Fetch op1.
        let mem1_base = if self.is_rel {
            pc.clone()
        } else if self.flag_op1_base_fp {
            fp
        } else {
            ap.clone()
        };

        let key = mem1_base + offset2;
        let mut op1_value = ab.get_from_memory(&self.memory, &key);
        let op1 = ab.deduce(op1_value.as_felts_mut()[0]);
        ab.set_in_memory(&self.memory, key, Felt252Expr::from(vec![op1.clone()]));

        // Update pc.
        let next_pc = if self.is_rel { op1 } else { pc + op1 };

        [next_pc, ap.clone() + const_expr!(2), ap + const_expr!(2)]
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            ("is_rel".to_string(), self.is_rel.to_string()),
            (
                "flag_op1_base_fp".to_string(),
                self.flag_op1_base_fp.to_string(),
            ),
        ]
        .into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

impl MemoryAirFn for CallOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
