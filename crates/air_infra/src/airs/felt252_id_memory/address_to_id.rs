use inst_def::InstDef;

use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

// This is a single-column component filled by Stwo that contains, in the k'th
// row, the ID of the felt252 value in memory address k.
const STWO_COMPONENT_TYPE_ID_FOR_ADDRESS: &str = "AddressToId_value";

// An AirFn representing the value component of the IdToBig table.
#[derive(Debug, Clone, Default, InstDef)]
pub struct IdForAddress {}

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryAddressToId {
    #[instdef(skip)]
    memory: Memory<FeltExpr, FeltExpr>,
}

impl AirFn for IdForAddress {
    type In = ();
    type Out = FeltExpr;

    fn name(&self) -> String {
        STWO_COMPONENT_TYPE_ID_FOR_ADDRESS.to_string()
    }

    fn call(&self, _air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        Self::Out::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
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

        air_builder.call_external_column(&IdForAddress {})
    }

    fn const_input(&self) -> Option<String> {
        Some((Seq {}).name())
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
