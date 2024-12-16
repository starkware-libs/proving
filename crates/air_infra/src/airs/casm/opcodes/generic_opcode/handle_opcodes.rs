use inst_def::InstDef;

use super::generic_opcode::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::common::*;
use crate::airs::felt252_utils::cond_as_small::*;
//  Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::variables::*;

#[derive(Clone, Debug, InstDef)]
// Add the relevant constraints for valid call, assert equal and ret opcodes.
// Note that jump and jnz are handled in the UpdateRegisters air function.
pub struct HandleOpcodes {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for HandleOpcodes {
    type In = (
        CasmStateVar,
        [FeltExpr; GENERIC_FLAGS_SIZE],
        [FeltExpr; 3],
        [Felt252Expr; 3],
    );
    type Out = ();

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        (casm_state, flags, [offset0, offset1, offset2], [dst, op0, res]): Self::In,
    ) -> Self::Out {
        // Handle assert_eq
        for (res_felt, dst_felt) in res.as_felts().into_iter().zip(dst.as_felts().iter()) {
            air_builder.constrain(
                flags[FLAG_OPCODE_ASSERT_EQ_INDEX].clone() * (res_felt.clone() - dst_felt.clone()),
                "",
            );
        }

        // Handle ret
        // offset0 = -2
        air_builder.constrain(
            flags[FLAG_OPCODE_RET_INDEX].clone() * (offset0.clone() + const_expr!(2)),
            "ret opcode offset0 equals -2",
        );
        // offset2 = -1
        air_builder.constrain(
            flags[FLAG_OPCODE_RET_INDEX].clone() * (offset2.clone() + const_expr!(1)),
            "ret opcode offset2 equals -1",
        );
        // Assert that FLAG_PC_UPDATE_JUMP = FLAG_DST_BASE_FP = FLAG_OP1_BASE_FP =FLAG_RES_OP1 = 1
        air_builder.constrain(
            flags[FLAG_OPCODE_RET_INDEX].clone()
                * (const_expr!(4)
                    - flags[FLAG_PC_UPDATE_JUMP_INDEX].clone()
                    - flags[FLAG_DST_BASE_FP_INDEX].clone()
                    - flags[FLAG_OP1_BASE_FP_INDEX].clone()
                    - flags[FLAG_RES_OP1_INDEX].clone()),
            "ret opcode flags pc_update_jump and dst_base_fp and op1_base_fp_and_res_op1 are on",
        );

        // Handle call
        // ofsset0 = 0
        air_builder.constrain(
            flags[FLAG_OPCODE_CALL_INDEX].clone() * offset0.clone(),
            "call opcode offset0 equals 0",
        );
        // offset1 = 1
        air_builder.constrain(
            flags[FLAG_OPCODE_CALL_INDEX].clone() * (const_expr!(1) - offset1.clone()),
            "call opcode offset1 equals 1",
        );
        // Assert that FLAG_OP0_BASE_FP = FLAG_DST_BASE_FP = 0
        air_builder.constrain(
            flags[FLAG_OPCODE_CALL_INDEX].clone()
                * (flags[FLAG_OP0_BASE_FP_INDEX].clone() + flags[FLAG_DST_BASE_FP_INDEX].clone()),
            "call opcode flags op0_base_fp and dst_base_fp are off",
        );

        // Push fp
        let dst_as_addr = air_builder.call(
            &CondFelt252AsAddr {},
            (dst, flags[FLAG_OPCODE_CALL_INDEX].clone()),
        );
        air_builder.constrain(
            flags[FLAG_OPCODE_CALL_INDEX].clone() * (dst_as_addr.var - casm_state.fp().var),
            "",
        );

        // Push next pc
        let op0_as_addr = air_builder.call(
            &CondFelt252AsAddr {},
            (op0.clone(), flags[FLAG_OPCODE_CALL_INDEX].clone()),
        );
        air_builder.constrain(
            flags[FLAG_OPCODE_CALL_INDEX].clone()
                * (op0_as_addr.var - (casm_state.pc().var + flags[INSTRUCTION_SIZE_INDEX].clone())),
            "",
        );
    }
}
