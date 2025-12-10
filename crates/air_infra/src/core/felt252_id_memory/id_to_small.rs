use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::relations::MEMORY_RELATION_NAME;
use serde::Serialize;

use crate::airs::casm::const_tables::seq::*;
use crate::core::air_fn::*;
use crate::core::constraint_connectedness_test;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::memory::*;
#[cfg(test)]
use crate::core::variables::*;

pub const SMALL_MEM_VALUE_N_FELTS: usize = 8;

/// External table for memory small value IDs.
#[derive(Debug, Clone, Default)]
pub struct MemIdForSmall {}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryIdToSmall {
    #[serde(skip)]
    memory: Memory<CasmId, [FeltExpr; SMALL_MEM_VALUE_N_FELTS]>,
}

impl IsMemory<SeqId, [FeltExpr; SMALL_MEM_VALUE_N_FELTS]> for MemoryIdToSmall {
    fn mem(&self) -> &Memory<CasmId, [FeltExpr; SMALL_MEM_VALUE_N_FELTS]> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<CasmId, [FeltExpr; SMALL_MEM_VALUE_N_FELTS]> {
        &mut self.memory
    }
}

/// A table with 9 columns. The first is the external column 'MemIdForSmall' represents the ID,
/// and the other 'SMALL_MEM_VALUE_N_FELTS' felts represent the corresponding small memory value.
/// This table yields a relation with an output type of `Felt252`, therefore, the output is padded
/// to match this type.  This air fn created to add the relevant constraints for this table,
/// but it is not used by the AIR infrastructure because we lookup into `MemoryIdToBig`,
/// that share the same relation.
impl AirFn for MemoryIdToSmall {
    type ExtIn = SeqId;
    type In = ();
    type Out = [FeltExpr; SMALL_MEM_VALUE_N_FELTS];

    fn call(&self, air_builder: &mut AirBuilder, _id: CasmId, _: ()) -> Self::Out {
        // constraint_connectedness_test fails because each pair of limbs is independent (not
        // connected to the other limbs by constraints). However, this is OK - the memory can
        // contain any valid value for a limb pair, independent of the other pairs.
        constraint_connectedness_test::exclude(self);

        #[allow(unused_mut)]
        let mut value_in_state = air_builder.component_context.state().get_felts();

        #[cfg(test)]
        if air_builder.is_run_mode() {
            value_in_state = self.memory.get(&_id).expect("ID not in memory").as_felts();
        }

        let values_array: [FeltExpr; SMALL_MEM_VALUE_N_FELTS] = value_in_state
            .clone()
            .try_into()
            .expect("Expected 8 limbs in small memory value");
        air_builder.call(
            &RangeCheckMemValue::<SMALL_MEM_VALUE_N_FELTS>::new(),
            values_array.clone(),
        );

        values_array
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }

    fn relation_name(&self) -> Option<String> {
        Some(MEMORY_RELATION_NAME.to_string())
    }
}
