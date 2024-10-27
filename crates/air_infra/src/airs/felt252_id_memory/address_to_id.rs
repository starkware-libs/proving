use inst_def::InstDef;

use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryAddressToId {
    #[instdef(skip)]
    memory: Memory<FeltExpr, FeltExpr>,
}

impl IsMemory<FeltExpr, FeltExpr> for MemoryAddressToId {
    fn mem(&self) -> &Memory<FeltExpr, FeltExpr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<FeltExpr, FeltExpr> {
        &mut self.memory
    }
}

impl AirFn for MemoryAddressToId {
    type In = FeltExpr;
    type Out = FeltExpr;

    fn call(&self, air_builder: &mut AirBuilder, _address: Self::In) -> Self::Out {
        #[cfg(test)]
        if air_builder.is_run_mode() {
            return self.memory.get(&_address).expect("Address not in memory");
        }

        // The output is a single-column filled by Stwo that contains, in the k'th
        // row, the ID of the felt252 value in memory address k.
        air_builder
            .state()
            .get_felts()
            .last()
            .expect("There should be a felt in the state")
            .clone()
    }

    fn const_input(&self) -> Option<String> {
        Some((Seq {}).name())
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }
}
