use inst_def::InstDef;

use super::decode_generic_instruction::*;
use super::eval_operands::*;
use super::handle_opcodes::*;
use super::update_registers::*;
use crate::airs::casm::casm_state::*;
use crate::core::air_fn::*;
use crate::core::felt252_id_memory::memory::*;

pub const FLAG_OP1_BASE_OP0_INDEX: usize = 15;
pub const FLAG_RES_OP1_INDEX: usize = 16;
pub const FLAG_PC_UPDATE_REGULAR_INDEX: usize = 17;
pub const FLAG_FP_UPDATE_REGULAR_INDEX: usize = 18;
pub const INSTRUCTION_SIZE_INDEX: usize = 19;
pub const GENERIC_FLAGS_SIZE: usize = 20;

/// Implements a generic Cairo0 instructions.
#[derive(Clone, Debug, InstDef, Default)]
pub struct GenericOpcode {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for GenericOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, air_builder: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let (flags_as_felts, offsets) = air_builder.call(
            &DecodeGenericInstruction {
                memory: self.memory.clone(),
            },
            casm_state.pc().clone(),
        );

        let [dst, op0, op1, res] = air_builder.call(
            &EvalOperands {
                memory: self.memory.clone(),
            },
            (casm_state.clone(), flags_as_felts.clone(), offsets.clone()),
        );
        air_builder.call(
            &HandleOpcodes {
                memory: self.memory.clone(),
            },
            (
                casm_state.clone(),
                flags_as_felts.clone(),
                offsets.clone(),
                [dst.clone(), op0.clone(), res.clone()],
            ),
        );

        air_builder.call(
            &UpdateRegisters {},
            (
                casm_state,
                flags_as_felts.clone(),
                [dst.clone(), op1.clone(), res.clone()],
            ),
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
