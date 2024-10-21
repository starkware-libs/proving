use inst_def::InstDef;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::airs::felt252_id_memory::memory::*;
use crate::airs::felt252_id_memory::read_positive::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

use crate::const_expr;

// Start address of the segment for this builtin.
// TODO: receive this at proof time as a public param. Until public params
// are implemented, have it as a dummy constant for testing.
pub const DUMMY_SEGMENT_START: u32 = 100;

#[derive(Debug, InstDef)]
pub struct RangeCheckBuiltin {
    pub bits: usize,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for RangeCheckBuiltin {
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        let instance_number = air_builder.call_external_column(&Seq {});

        air_builder.call(
            &ReadPositive {
                num_bits: self.bits,
                memory: self.memory.clone(),
            },
            CasmAddress::new(const_expr!(DUMMY_SEGMENT_START) + instance_number, "value"),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}
