use air_common::TraceType;
use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::public_params::PublicParam;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::felt252_id_memory::read_positive::ReadPositive;
use air_infra::seq::Seq;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct RangeCheckBuiltin {
    pub bits: usize,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
    #[serde(skip)]
    pub segment_start: PublicParam,
}

impl AirFn for RangeCheckBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_number = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(self.segment_start.clone());

        air_builder.call(
            &ReadPositive { num_bits: self.bits, memory: self.memory.clone() },
            CasmAddress::new(segment_start + instance_number, "value"),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }

    fn name(&self) -> String {
        match self.bits {
            128 => "range_check_builtin".to_string(),
            _ => format!("range_check{}_builtin", self.bits),
        }
    }
}
