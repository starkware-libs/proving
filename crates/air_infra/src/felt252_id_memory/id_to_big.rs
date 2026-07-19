use air_common::{MEMORY_RELATION_NAME, TraceType};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::core::air_fn::*;
use crate::core::constraint_connectedness_test;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::memory::*;
use crate::core::variables::*;
use crate::felt252_id_memory::memory::*;
use crate::range_check::*;
use crate::seq::*;

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryIdToBig {
    #[serde(skip)]
    memory: Memory<CasmId, Felt252Expr>,
}

impl IsMemory<SeqId, Felt252Expr> for MemoryIdToBig {
    fn mem(&self) -> &Memory<CasmId, Felt252Expr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<CasmId, Felt252Expr> {
        &mut self.memory
    }
}

/// A table with 30 columns: the external column 'MemIdForBig' that represents the ID,
/// a multiplicity column and then 28 felts that represent the corresponding big memory value.
impl AirFn for MemoryIdToBig {
    type ExtIn = SeqId;
    type In = ();
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _id: CasmId, _: ()) -> Self::Out {
        // This component is manually implemented in the prover. The code here is just for tests.
        constraint_connectedness_test::exclude(self);

        let state_felts = air_builder.component_context.state().get_felts();

        #[allow(unused_mut)]
        // Skip the multiplicity column
        let mut value_in_state: Felt252Expr = state_felts[1..].to_vec().into();

        #[cfg(any(test, feature = "test"))]
        if air_builder.is_run_mode() {
            value_in_state = self.memory.get(&_id).expect("ID not in memory");
        }

        air_builder.call(
            &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
            value_in_state
                .as_felts()
                .try_into()
                .expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
        );

        value_in_state
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }

    fn relation_names(&self) -> Vec<String> {
        vec![MEMORY_RELATION_NAME.to_string()]
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct RangeCheckMemValue<const N: usize> {
    n: usize,
}

impl<const N: usize> RangeCheckMemValue<N> {
    pub fn new() -> Self {
        Self { n: N }
    }
}

// RangeCheckMemValue assumes there are 9 bits per felt in a felt252 (FELT252_BITS_PER_WORD).
impl<const N: usize> AirFn for RangeCheckMemValue<N> {
    type ExtIn = ();
    type In = [FeltExpr; N];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), value: Self::In) -> Self::Out {
        assert!(N.is_multiple_of(2), "Expected even number N",);
        for (i, pair) in value.chunks(2).enumerate() {
            range_check_variant(
                air_builder,
                &[FELT252_BITS_PER_WORD as u16, FELT252_BITS_PER_WORD as u16],
                pair,
                i % 8,
            );
        }
    }
}
