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

use crate::casm::builtins::pedersen::pedersen_aggregator::*;

const PEDERSEN_INSTANCE_SIZE: u32 = 3;

#[derive(Debug, Serialize, Default)]
pub struct PedersenBuiltin<const NUM_WINDOWS: usize> {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl<const NUM_WINDOWS: usize> AirFn for PedersenBuiltin<NUM_WINDOWS> {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(PublicParam::PedersenBuiltinSegmentStart);
        let instance_addr = air_builder.let_(
            instance_num * const_expr!(PEDERSEN_INSTANCE_SIZE) + segment_start,
            "instance_addr",
        );

        // Read the input id's,
        let input_ids: [CasmId; 2] = from_fn(|i| {
            let address = CasmAddress::new(
                instance_addr.clone() + const_expr!(i),
                &format!("input_state_{i}"),
            );
            air_builder.call(&ReadId { memory: self.memory.clone() }, address.clone())
        });

        // Read the output id.
        let output_id = air_builder.call(
            &ReadId { memory: self.memory.clone() },
            CasmAddress::new(instance_addr.clone() + const_expr!(2), "output_state"),
        );

        air_builder.lookup_call(
            &PedersenAggregator::<NUM_WINDOWS>::new(self.memory.clone()),
            (),
            (input_ids, output_id),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }

    fn name(&self) -> String {
        match NUM_WINDOWS {
            14 => "pedersen_builtin",
            28 => "pedersen_builtin_narrow_windows",
            _ => panic!("Unsupported NUM_WINDOWS val {NUM_WINDOWS}"),
        }
        .into()
    }
}
