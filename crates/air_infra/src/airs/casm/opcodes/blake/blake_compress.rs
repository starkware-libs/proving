use inst_def::InstDef;

use super::create_blake_output::*;
use super::decode_blake_opcode::*;
use super::round::*;
use super::verify_blake_word::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::opcodes::blake::create_round_input::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::felt252_id_memory::memory::*;

#[derive(Clone, Debug, InstDef, Default)]
pub struct BlakeCompressOpcode {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for BlakeCompressOpcode {
    type ExtIn = ();
    type In = CasmStateVar;
    type Out = CasmStateVar;

    fn call(&self, ab: &mut AirBuilder, _: (), casm_state: Self::In) -> Self::Out {
        let ([h_pointer, message_pointer, new_state_pointer], t, [ap_add_1, is_last_block]) = ab
            .call(
                &DecodeBlakeOpcode {
                    memory: self.memory.clone(),
                },
                casm_state.clone(),
            );

        // Create round_input.
        let input = ab.call(
            &CreateRoundInput {
                memory: self.memory.clone(),
            },
            (h_pointer, t.clone(), is_last_block.clone()),
        );

        // Run 10 blake rounds.
        let (new_state, _) = ab.chain_lookup_call(
            &Round {
                memory: self.memory.clone(),
            },
            (input.clone(), message_pointer),
            0,
            10,
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
        let verify_blake_word = &VerifyBlakeWord {
            memory: self.memory.clone(),
        };
        for i in 0..8 {
            let current_addr = CasmAddress::new(
                new_state_pointer.var.clone() + const_expr!(i),
                &format!("new_state_{}", i),
            );
            ab.call(
                verify_blake_word,
                (current_addr, expected_output[i as usize].clone()),
            );
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
