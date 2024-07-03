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

/// The jump opcode.
/// Implements the Cairo0 instructions:
/// - jump rel
/// - jump abs

#[derive(Clone, Debug)]
pub struct JumpOpcode {
    is_rel: bool,
    flag_op1_base_fp: bool,
    flag_ap_update_add: bool,

    memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for JumpOpcode {
    type In = CasmState;
    type Out = CasmState;

    fn call(&self, ab: &mut AirBuilder, [pc, ap, fp]: Self::In) -> Self::Out {
        // Create the constant offsets.
        let offset0 = offset_as_u16(-1);
        let offset1 = offset_as_u16(-1);
        // Create the flags.
        let flag_op1_base_ap = if self.is_rel {
            assert!(!self.flag_op1_base_fp);
            false
        } else {
            !self.flag_op1_base_fp
        };
        let flags = NamedFlags {
            dst_base_fp: true,
            op0_base_fp: true,
            op1_imm: self.is_rel,
            op1_base_fp: self.flag_op1_base_fp,
            op1_base_ap: flag_op1_base_ap,
            res_add: false,
            res_mul: false,
            pc_update_jump_rel: self.is_rel,
            pc_update_jump: !self.is_rel,
            pc_update_jnz: false,
            ap_update_add: false,
            ap_update_add_1: self.flag_ap_update_add,
            opcode_call: false,
            opcode_ret: false,
            opcode_assert_eq: false,
        };

        // Check the instruction.
        let [_, _, offset2] = ab.call(
            &CheckInstruction {
                const_offsets: [Some(offset0), Some(offset1), None],
                const_flags: flags.into(),
                memory: self.memory.clone(),
            },
            pc.clone(),
        );
        // Fetch op1.
        let mem1_base = match self.is_rel {
            true => pc.clone(),
            false => match self.flag_op1_base_fp {
                true => fp.clone(),
                false => ap.clone(),
            },
        };

        let key = mem1_base + offset2;
        let mut op1_value = ab.get_from_memory(&self.memory, &key);
        let op1 = ab.deduce(op1_value.as_felts_mut()[0]);
        //should we write the key as well? set in memory assume they both in state
        ab.set_in_memory(&self.memory, key, Felt252Expr::from(vec![op1.clone()]));

        let next_pc = match self.is_rel {
            true => pc + op1,
            false => op1,
        };

        let next_ap = match self.flag_ap_update_add {
            true => ap + const_expr!(1),
            false => ap,
        };

        [next_pc, next_ap, fp]
    }
}

impl MemoryAirFn for JumpOpcode {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}
