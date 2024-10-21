use compiled_casm_air::prover_types::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};
use inst_def::InstDef;

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

// Stwo fills an IdToBig table and splits it into two components: The key (Id) component
// and the value (Big) component.
const STWO_COMPONENT_TYPE_MEMORY_IDS: &str = "IdToBig_key";
const STWO_COMPONENT_TYPE_BIG_VALUE_FOR_ID: &str = "IdToBig_value";

// An AirFn representing the value component of the IdToBig table.
#[derive(Debug, Clone, Default, InstDef)]
pub struct BigValueForId {}

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryIdToBig {
    #[instdef(skip)]
    memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for BigValueForId {
    type In = ();
    type Out = Felt252Expr;

    fn name(&self) -> String {
        STWO_COMPONENT_TYPE_BIG_VALUE_FOR_ID.to_string()
    }

    fn call(&self, _air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        Self::Out::default()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Const
    }
}

impl IsMemory<FeltExpr, Felt252Expr> for MemoryIdToBig {
    fn mem(&self) -> &Memory<FeltExpr, Felt252Expr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<FeltExpr, Felt252Expr> {
        &mut self.memory
    }
}

impl AirFn for MemoryIdToBig {
    type In = FeltExpr;
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _id: Self::In) -> Self::Out {
        #[allow(unused_mut)]
        let mut value_in_state = air_builder.call_external_column(&BigValueForId {});

        #[cfg(test)]
        if air_builder.is_run_mode() {
            value_in_state = self.memory.get(&_id).expect("ID not in memory");
        }

        let mut i = 0;
        while i < FELT252_N_WORDS {
            let limbs_left = FELT252_N_WORDS - i;

            if limbs_left >= 2 {
                air_builder.lookup_call(
                    &RangeCheck {
                        bits: [FELT252_BITS_PER_WORD as u16, FELT252_BITS_PER_WORD as u16],
                    },
                    [value_in_state.get_felt(i), value_in_state.get_felt(i + 1)],
                );
                i += 2;
            } else {
                assert!(limbs_left == 1);
                air_builder.lookup_call(
                    &RangeCheck {
                        bits: [FELT252_BITS_PER_WORD as u16],
                    },
                    [value_in_state.get_felt(i)],
                );
                i += 1;
            }
        }

        value_in_state
    }

    fn const_input(&self) -> Option<String> {
        Some(STWO_COMPONENT_TYPE_MEMORY_IDS.to_string())
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
