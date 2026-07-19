use std::sync::LazyLock;

use air_common::TraceType;
use air_infra::casm_state::CasmStateVar;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;

use super::decode_generic_instruction::*;
use super::eval_operands::*;
use super::handle_opcodes::*;
use super::update_registers::*;
use crate::casm::common::*;

pub const FLAG_OP1_BASE_OP0_INDEX: usize = 15;
pub const FLAG_RES_OP1_INDEX: usize = 16;
pub const FLAG_PC_UPDATE_REGULAR_INDEX: usize = 17;
pub const FLAG_FP_UPDATE_REGULAR_INDEX: usize = 18;
pub const INSTRUCTION_SIZE_INDEX: usize = 19;
pub const GENERIC_FLAGS_SIZE: usize = 20;

pub static GENERIC_FLAG_NAMES: LazyLock<Vec<&str>> = LazyLock::new(|| {
    [
        FLAG_NAMES.as_slice(),
        ["op1_base_op0", "res_op1", "pc_update_regular", "fp_update_regular", "instruction_size"]
            .as_slice(),
    ]
    .concat()
});

/// Implements a generic Cairo0 instructions.
#[derive(Clone, Debug, Serialize, Default)]
pub struct GenericOpcode {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for GenericOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, air_builder: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let (flags_as_felts, offsets) = air_builder.call(
            &DecodeGenericInstruction { memory: self.memory.clone() },
            casm_state.pc().clone(),
        );

        let [dst, op0, op1, res] = air_builder.call(
            &EvalOperands { memory: self.memory.clone() },
            (casm_state.clone(), flags_as_felts.clone(), offsets.clone()),
        );
        air_builder.call(
            &HandleOpcodes { memory: self.memory.clone() },
            (
                casm_state.clone(),
                flags_as_felts.clone(),
                offsets.clone(),
                [dst.clone(), op0.clone(), res.clone()],
            ),
        );

        air_builder.call(
            &UpdateRegisters {},
            (casm_state, flags_as_felts.clone(), [dst.clone(), op1.clone(), res.clone()]),
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
