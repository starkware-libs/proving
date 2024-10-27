use compiled_casm_air::prover_types::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};
use inst_def::InstDef;

use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;

#[derive(Debug, Clone, Default, InstDef)]
pub struct MemoryIdToBig {
    #[instdef(skip)]
    memory: Memory<FeltExpr, Felt252Expr>,
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
        let mut value_in_state: Felt252Expr = air_builder.state().get_felts()[1..].to_vec().into();

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

    fn trace_type(&self) -> TraceType {
        TraceType::Memory
    }
}
