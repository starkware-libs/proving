use compiled_casm_air::compiled_structs::TraceType;
use compiled_casm_air::relations::MEMORY_RELATION_NAME;
use serde::Serialize;
use stwo_cairo_common::preprocessed_columns::preprocessed_trace::PreProcessedColumn;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::constraint_connectedness_test;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::memory::*;
use crate::core::variables::*;

const STWO_COMPONENT_TYPE_MEM_ID_FOR_BIG: &str = "MemoryIdForBig";

/// External table for the memory big value IDs.
#[derive(Debug, Clone, Default)]
pub struct MemIdForBig {}

impl ExtTable for MemIdForBig {
    type T = CasmId;

    fn column_ids() -> Vec<String> {
        vec![STWO_COMPONENT_TYPE_MEM_ID_FOR_BIG.to_owned()]
    }

    fn preprocessed_columns() -> Vec<Box<dyn PreProcessedColumn>> {
        vec![]
    }
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct MemoryIdToBig {
    #[serde(skip)]
    memory: Memory<CasmId, Felt252Expr>,
}

impl IsMemory<MemIdForBig, Felt252Expr> for MemoryIdToBig {
    fn mem(&self) -> &Memory<CasmId, Felt252Expr> {
        &self.memory
    }

    fn mem_mut(&mut self) -> &mut Memory<CasmId, Felt252Expr> {
        &mut self.memory
    }
}

/// A table with 29 columns. The first is the external column 'MemIdForBig' represents the ID,
/// and the other 28 felts represent the corresponding big memory value.
impl AirFn for MemoryIdToBig {
    type ExtIn = MemIdForBig;
    type In = ();
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _id: CasmId, _: ()) -> Self::Out {
        // This component is manually implemented in the prover. The code here is just for tests.
        constraint_connectedness_test::exclude(self);

        #[allow(unused_mut)]
        let mut value_in_state: Felt252Expr =
            air_builder.component_context.state().get_felts().into();

        #[cfg(test)]
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

    fn relation_name(&self) -> Option<String> {
        Some(MEMORY_RELATION_NAME.to_string())
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
        assert!(N % 2 == 0, "Expected even number N",);
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
