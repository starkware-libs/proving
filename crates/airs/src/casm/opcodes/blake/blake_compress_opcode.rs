use air_common::TraceType;
use air_infra::casm_state::{CasmAddress, CasmStateVar};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use serde::Serialize;

use super::create_blake_output::*;
use super::decode_blake_opcode::*;
use super::round::*;
use super::verify_u32::*;
use crate::casm::opcodes::blake::create_blake_round_input::*;

pub const BLAKE_NUM_ROUNDS: usize = 10;

#[derive(Clone, Debug, Serialize, Default)]
pub struct BlakeCompressOpcode {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for BlakeCompressOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let ([h_pointer, message_pointer, new_state_pointer], t, [ap_add_1, is_last_block]) =
            ab.call(&DecodeBlakeOpcode { memory: self.memory.clone() }, casm_state.clone());

        // Create round_input.
        let input = ab.call(
            &CreateBlakeRoundInput { memory: self.memory.clone() },
            (h_pointer, t.clone(), is_last_block.clone()),
        );

        // Run BLAKE_NUM_ROUNDS blake rounds.
        let (new_state, _) = ab.chain_lookup_call(
            &BlakeRound { memory: self.memory.clone() },
            (input.clone(), message_pointer),
            0,
            BLAKE_NUM_ROUNDS,
        );

        // Create blake output.
        let h: [UInt32Expr; 8] = input
            .get(0..8)
            .expect("Expected 16 elements in input")
            .to_owned()
            .try_into()
            .expect("Expected 8 elements in h");
        let expected_output = ab.call(&CreateBlakeOutput {}, (h, new_state.clone()));

        // Verify blake output.
        let verify_u32 = &VerifyU32 { memory: self.memory.clone() };
        for i in 0..8 {
            let current_addr = CasmAddress::new(
                new_state_pointer.var.clone() + const_expr!(i),
                &format!("new_state_{i}"),
            );
            ab.call(verify_u32, (current_addr, expected_output[i as usize].clone()));
        }

        // Calculate the next state.
        let next_ap = casm_state.ap().var + ap_add_1.as_felt();
        let next_pc = casm_state.pc().var + const_expr!(1);

        CasmStateVar::new(next_pc, next_ap, casm_state.fp().var)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Opcode
    }
}
