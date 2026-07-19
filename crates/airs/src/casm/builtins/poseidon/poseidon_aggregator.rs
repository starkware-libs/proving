use std::array::from_fn;

use air_common::TraceType;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::felt252_id_memory::memory::{CasmId, Felt252IdMemory};
use air_infra::felt252_id_memory::read_positive::ReadPositiveKnownId;
use serde::Serialize;

use super::poseidon_permutation::*;
use crate::felt252_utils::felt252_packing27::*;

#[derive(Debug, Serialize, Default)]
pub struct PoseidonAggregator {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for PoseidonAggregator {
    type ExtIn = ();
    type In = ([CasmId; 3], [CasmId; 3]);
    type Out = ();

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (input_ids, output_ids): Self::In,
    ) -> Self::Out {
        let input_state: [Felt252Width27Expr; 3] = from_fn(|i| {
            let packed_input_state = felt252_pack_into27(air_builder.call(
                &ReadPositiveKnownId { num_bits: 252, memory: self.memory.clone() },
                input_ids[i].clone(),
            ));
            air_builder.let_(packed_input_state, &format!("packed_input_state_{i}"))
        });

        let output_state = air_builder.call(&PoseidonHadesPermutation {}, input_state);

        for (i, part) in output_state.into_iter().enumerate() {
            let unpacked_part =
                air_builder.call(&Felt252UnpackFrom27 { range_check_output: false }, part);
            // TODO(DanC): Allow and use direct access to memory in packed form, to save trace
            // columns
            self.memory.mem_verify_known_id(air_builder, &output_ids[i], unpacked_part);
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
