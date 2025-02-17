use inst_def::InstDef;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_MEM_ID_FOR_BIG: &str = "MemoryIdForBig";

#[derive(Debug, Clone, Default)]
pub struct MemIdForBig {}

impl ExtTable for MemIdForBig {
    const CONST_TRACE_ID: &'static str = STWO_COMPONENT_TYPE_MEM_ID_FOR_BIG;
    type T = FeltExpr;
}

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryIdToBig {
    #[instdef(skip)]
    memory: Memory<FeltExpr, Felt252Expr>,
}

impl IsMemory<MemIdForBig, Felt252Expr> for MemoryIdToBig {
    fn mem(&self) -> &Memory<FeltExpr, Felt252Expr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<FeltExpr, Felt252Expr> {
        &mut self.memory
    }
}

impl AirFn for MemoryIdToBig {
    type ExtIn = MemIdForBig;
    type In = ();
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _id: FeltExpr, _: ()) -> Self::Out {
        #[allow(unused_mut)]
        let mut value_in_state: Felt252Expr = air_builder.state().get_felts().into();

        #[cfg(test)]
        if air_builder.is_run_mode() {
            value_in_state = self.memory.get(&_id).expect("ID not in memory");
        }

        air_builder.call(&RangeCheckBigValue {}, value_in_state.clone());

        value_in_state
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }
}

#[derive(Debug, Clone, Default, InstDef)]
pub struct RangeCheckBigValue {}

// RangeCheckBigValue assumes there are 9 bits per felt in a felt252 (FELT252_BITS_PER_WORD)
impl AirFn for RangeCheckBigValue {
    type ExtIn = ();
    type In = Felt252Expr;
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), value: Self::In) -> Self::Out {
        let mut i = 0;
        while i < FELT252_N_WORDS {
            let limbs_left = FELT252_N_WORDS - i;

            if limbs_left >= 2 {
                range_check(
                    air_builder,
                    &[FELT252_BITS_PER_WORD as u16, FELT252_BITS_PER_WORD as u16],
                    &[value.get_felt(i), value.get_felt(i + 1)],
                );
                i += 2;
            } else {
                assert!(limbs_left == 1);
                range_check(
                    air_builder,
                    &[FELT252_BITS_PER_WORD as u16],
                    &[value.get_felt(i)],
                );
                i += 1;
            }
        }
    }
}
