use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::public_params::PublicParam;
use serde::Serialize;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_positive::*;

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
            &ReadPositive {
                num_bits: self.bits,
                memory: self.memory.clone(),
            },
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
