use inst_def::InstDef;

use super::g::*;
use super::read_blake_word::*;
use super::round_sigma::*;
use crate::airs::casm::casm_state::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::variables::*;

pub type BlakeState = [UInt32Expr; 16];
pub type BlakeRoundInput = (BlakeState, CasmAddress);

// Each row `i` contains the state's indices sent to the function `G` in the `i`-th call for each
// round.
pub const G_STATE_INDICES: [[usize; 4]; 8] = [
    [0, 4, 8, 12],
    [1, 5, 9, 13],
    [2, 6, 10, 14],
    [3, 7, 11, 15],
    [0, 5, 10, 15],
    [1, 6, 11, 12],
    [2, 7, 8, 13],
    [3, 4, 9, 14],
];

#[derive(Debug, InstDef, Default)]
pub struct BlakeRound {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for BlakeRound {
    type ExtIn = ();
    type In = (ChainIdVar, RoundNumVar, BlakeRoundInput);
    type Out = (ChainIdVar, RoundNumVar, BlakeRoundInput);

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (chain, rnd, (mut state, massage_pointer)): Self::In,
    ) -> Self::Out {
        // Read current message permutation (sigma) according to the round.
        let curr_sigma = air_builder.lookup_call(&BlakeRoundSigma {}, [rnd.clone()], ());

        // Read the current messgae according to the permutation.
        let read_blake_word = ReadBlakeWord {
            memory: self.memory.clone(),
        };
        let mut current_message = vec![];
        for (i, index) in curr_sigma.into_iter().enumerate() {
            let addr = CasmAddress::new(
                massage_pointer.clone().var + index.clone(),
                &format!("message_word_{}", i),
            );
            let curr_word = air_builder.call(&read_blake_word, addr);
            current_message.push(curr_word);
        }

        // Apply the G function to the state.
        let g = BlakeG {};
        for (row_index, &[i0, i1, i2, i3]) in G_STATE_INDICES.iter().enumerate() {
            [state[i0], state[i1], state[i2], state[i3]] = air_builder.lookup_call(
                &g,
                (),
                [
                    state[i0].clone(),
                    state[i1].clone(),
                    state[i2].clone(),
                    state[i3].clone(),
                    current_message[row_index * 2].clone(),
                    current_message[row_index * 2 + 1].clone(),
                ],
            );
        }
        (chain, rnd + const_expr!(1), (state, massage_pointer))
    }

    fn trace_type(&self) -> TraceType {
        TraceType::ChainRound
    }

    fn deduce_output(&self) -> Option<String> {
        // TODO(Stav): Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}

impl ChainRoundAirFn<BlakeRoundInput> for BlakeRound {
    fn number_of_chains(&self) -> usize {
        1
    }
}
