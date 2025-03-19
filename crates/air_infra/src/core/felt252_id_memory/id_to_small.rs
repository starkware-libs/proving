use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::relations::MEMORY_RELATION_NAME;
use inst_def::InstDef;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;
use crate::core::memory::*;
use crate::core::variables::*;

pub const SMALL_MEM_VALUE_N_FELTS: usize = 8;
const STWO_COMPONENT_TYPE_MEM_ID_FOR_72BITS: &str = "MemoryIdForSmall";

/// External table for memory small value IDs.
#[derive(Debug, Clone, Default)]
pub struct MemIdForSmall {}

impl ExtTable for MemIdForSmall {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_MEM_ID_FOR_72BITS;
    type T = FeltExpr;
}

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryIdToSmall {
    #[instdef(skip)]
    memory: Memory<FeltExpr, Felt252Expr>,
}

impl IsMemory<MemIdForSmall, Felt252Expr> for MemoryIdToSmall {
    fn mem(&self) -> &Memory<FeltExpr, Felt252Expr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<FeltExpr, Felt252Expr> {
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
    type ExtIn = MemIdForSmall;
    type In = ();
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _id: FeltExpr, _: ()) -> Self::Out {
        #[allow(unused_mut)]
        let mut value_in_state = air_builder.component_context.state().get_felts();

        #[cfg(test)]
        if air_builder.is_run_mode() {
            value_in_state = self.memory.get(&_id).expect("ID not in memory").as_felts();
            value_in_state.truncate(SMALL_MEM_VALUE_N_FELTS);
        }

        air_builder.call(
            &RangeCheckMemValue::<SMALL_MEM_VALUE_N_FELTS> {},
            value_in_state
                .clone()
                .try_into()
                .expect("Expected 8 limbs in small memory value"),
        );

        Felt252Expr::from(value_in_state)
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }

    fn relation_name(&self) -> Option<String> {
        Some(MEMORY_RELATION_NAME.to_string())
    }
}
