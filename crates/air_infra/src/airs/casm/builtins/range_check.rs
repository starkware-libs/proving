use compiled_casm_air::public_params::PublicParam;
use inst_def::InstDef;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::read_positive::*;

#[derive(Debug, InstDef)]
pub struct RangeCheckBuiltin {
    pub bits: usize,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
    #[instdef(skip)]
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
}
