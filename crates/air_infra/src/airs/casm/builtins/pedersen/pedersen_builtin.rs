use std::array::from_fn;

use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::public_params::PublicParam;
use serde::Serialize;

use crate::airs::casm::builtins::pedersen::pedersen_aggregator::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_id::*;

const PEDERSEN_INSTANCE_SIZE: u32 = 3;

#[derive(Debug, Serialize, Default)]
pub struct PedersenBuiltin {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for PedersenBuiltin {
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
                &format!("input_state_{}", i),
            );
            air_builder.call(
                &ReadId {
                    memory: self.memory.clone(),
                },
                address.clone(),
            )
        });

        // Read the output id.
        let output_id = air_builder.call(
            &ReadId {
                memory: self.memory.clone(),
            },
            CasmAddress::new(instance_addr.clone() + const_expr!(2), "output_state"),
        );

        air_builder.lookup_call(
            &PedersenAggregator {
                memory: self.memory.clone(),
            },
            (),
            (input_ids, output_id),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}
