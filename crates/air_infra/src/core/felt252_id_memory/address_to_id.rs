use inst_def::InstDef;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryAddressToId {
    #[instdef(skip)]
    memory: Memory<CasmAddress, FeltExpr>,
}

impl IsMemory<SeqAddr, FeltExpr> for MemoryAddressToId {
    fn mem(&self) -> &Memory<CasmAddress, FeltExpr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<CasmAddress, FeltExpr> {
        &mut self.memory
    }
}

impl AirFn for MemoryAddressToId {
    type ExtIn = SeqAddr;
    type In = ();
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _address: CasmAddress, _: ()) -> Self::Out {
        #[cfg(test)]
        if air_builder.is_run_mode() {
            return self.memory.get(&_address).expect("Address not in memory");
        }

        // The output is a single-column filled by Stwo that contains, in the k'th
        // row, the ID of the felt252 value in memory address k.
        air_builder
            .component_context
            .state()
            .get_felts()
            .last()
            .expect("There should be a felt in the state")
            .clone()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }
}
