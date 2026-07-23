use std::array::from_fn;

use air_common::TraceType;
use air_infra::casm_state::CasmAddress;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::public_params::PublicParam;
use air_infra::felt252_id_memory::memory::{CasmId, Felt252IdMemory};
use air_infra::felt252_id_memory::read_id::ReadId;
use air_infra::seq::Seq;
use serde::Serialize;

use crate::casm::builtins::poseidon::poseidon_aggregator::*;

// Each Poseidon operation consists of 6 cells (3 inputs and 3 outputs, each being a single state).
pub const CELLS_PER_POSEIDON: u32 = 6;

/// The builtin enforces that
///   PoseidonHadesPermutation(mem[addr : addr + 3]) = mem[addr + 3 : addr + 6]
/// where addr = PoseidonBuiltinSegmentStart + Seq * 6.
#[derive(Debug, Serialize, Default)]
pub struct PoseidonBuiltin {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for PoseidonBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(PublicParam::PoseidonBuiltinSegmentStart);
        let instance_addr = air_builder
            .let_(instance_num * const_expr!(CELLS_PER_POSEIDON) + segment_start, "instance_addr");

        // Read the input id's,
        let input_ids: [CasmId; 3] = from_fn(|i| {
            let address = CasmAddress::new(
                instance_addr.clone() + const_expr!(i),
                &format!("input_state_{i}"),
            );
            air_builder.call(&ReadId { memory: self.memory.clone() }, address.clone())
        });

        // Read the output id's,
        let output_ids: [CasmId; 3] = from_fn(|i| {
            let address = CasmAddress::new(
                instance_addr.clone() + const_expr!(i + 3),
                &format!("output_state_{i}"),
            );
            air_builder.call(&ReadId { memory: self.memory.clone() }, address.clone())
        });

        air_builder.lookup_call(
            &PoseidonAggregator { memory: self.memory.clone() },
            (),
            (input_ids, output_ids),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}
