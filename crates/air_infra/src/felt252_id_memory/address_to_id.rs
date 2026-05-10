use air_common::TraceType;
use serde::Serialize;

use crate::casm_state::*;
use crate::core::air_fn::*;
use crate::core::memory::*;
use crate::felt252_id_memory::memory::*;
use crate::seq::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryAddressToId {
    #[serde(skip)]
    memory: Memory<CasmAddress, CasmId>,
}

impl IsMemory<SeqAddr, CasmId> for MemoryAddressToId {
    fn mem(&self) -> &Memory<CasmAddress, CasmId> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<CasmAddress, CasmId> {
        &mut self.memory
    }
}

impl AirFn for MemoryAddressToId {
    type ExtIn = SeqAddr;
    type In = ();
    type Out = CasmId;

    fn call(&self, air_builder: &mut AirBuilder, address: CasmAddress, _: ()) -> Self::Out {
        #[cfg(any(test, feature = "test"))]
        if air_builder.is_run_mode() {
            return self.memory.get(&address).expect("Address not in memory");
        }

        // The output is a single-column filled by Stwo that contains, in the k'th
        // row, the ID of the felt252 value in memory address k.
        CasmId::new(
            air_builder
                .component_context
                .state()
                .get_felts()
                .last()
                .expect("There should be a felt in the state")
                .clone(),
            &address.extra_info.unwrap_or_default(),
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }
}
