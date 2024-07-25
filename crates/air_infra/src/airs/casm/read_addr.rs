use crate::core::air_fn::{AirFn, TraceType};
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::FeltExpr;
use crate::core::memory::*;
use crate::core::prover_types::FELT252_BITS_PER_WORD;
use crate::core::variables::AirVar;

use super::common::*;
use super::read_small_felt252::*;

// Macros
use crate::const_expr;

#[derive(Debug, Default)]
pub struct ReadAddr {
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for ReadAddr {
    type In = CasmAddress;
    type Out = CasmAddress;

    fn call(&self, air_builder: &mut crate::core::air_fn::AirBuilder, key: Self::In) -> Self::Out {
        let op1 = air_builder
            .call(
                &ReadSmallFelt252 {
                    num_bits: 2 * FELT252_BITS_PER_WORD,
                    memory: self.memory.clone(),
                },
                key,
            )
            .as_felts();
        op1[0].clone() + (op1[1].clone() * const_expr!(1 << FELT252_BITS_PER_WORD))
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Inline
    }
}
