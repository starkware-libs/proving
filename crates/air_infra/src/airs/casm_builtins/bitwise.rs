use crate::airs::bitwise_and::BitwiseAnd;
use crate::airs::bitwise_xor::BitwiseXor;
use crate::const_expr;
use crate::core::air_fn::AirBuilder;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::Memory;
use crate::core::variables::AirVar;

// Start address of the segment for this builtin.
// TODO: receive this at proof time as a public param. Until public params
// are implemented, have it as a dummy constant for testing.
pub const DUMMY_BITWISE_SEGMENT_START: u32 = 500;
pub const BITWISE_SEGMENT_JUMP: u32 = 5;

// The builtin will enforce:
// mem[addr] & mem[addr + 1] == mem[addr + 2]
// mem[addr] ^ mem[addr + 1] == mem[addr + 3]
// mem[addr] | mem[addr + 1] == mem[addr + 4]
// where addr = first_addr + 5 * i for 0 <= i < base.size,
// where {&, |, ^} are bitwise {and, or, xor} respectively on 251-bit integers (the field elements
// are guaranteed to be representable as 251-bit integers).
#[derive(Debug)]
pub struct BitwiseBuiltin {
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for BitwiseBuiltin {
    type In = FeltExpr;
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, instance_num: Self::In) -> Self::Out {
        let a_as_felts = air_builder
            .get_from_memory(&self.memory, &get_addr(instance_num.clone(), 0))
            .as_felts();
        let b_as_felts = air_builder
            .get_from_memory(&self.memory, &get_addr(instance_num.clone(), 1))
            .as_felts();
        let mut expected_and = vec![];
        let mut expected_xor = vec![];
        let mut expected_or = vec![];
        for (mut a, mut b) in a_as_felts.into_iter().zip(b_as_felts.into_iter()) {
            air_builder.deduce(&mut a);
            air_builder.deduce(&mut b);
            let a_and_b = air_builder.lookup_call(&BitwiseAnd {}, [a.clone(), b.clone()]);
            let a_xor_b = air_builder.lookup_call(&BitwiseXor {}, [a, b]);
            let a_or_b = air_builder.let_for_deduction(a_and_b.clone() + a_xor_b.clone());
            expected_and.push(a_and_b);
            expected_xor.push(a_xor_b);
            expected_or.push(a_or_b);
        }
        air_builder.set_in_memory(
            &self.memory,
            get_addr(instance_num.clone(), 2),
            Felt252Expr::from(expected_and),
        );
        air_builder.set_in_memory(
            &self.memory,
            get_addr(instance_num.clone(), 3),
            Felt252Expr::from(expected_xor),
        );
        air_builder.set_in_memory(
            &self.memory,
            get_addr(instance_num.clone(), 4),
            Felt252Expr::from(expected_or),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}

fn get_addr(instance_num: FeltExpr, offset: u32) -> FeltExpr {
    const_expr!(DUMMY_BITWISE_SEGMENT_START)
        + instance_num * const_expr!(BITWISE_SEGMENT_JUMP)
        + const_expr!(offset)
}
