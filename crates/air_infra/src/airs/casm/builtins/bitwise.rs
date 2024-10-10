use inst_def::InstDef;

use crate::airs::casm::bitwise_xor::*;
use crate::airs::casm::const_tables::seq::*;
use crate::airs::felt252_id_memory::memory::*;
use crate::airs::felt252_id_memory::read_positive::*;
use crate::airs::felt252_id_memory::verify::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

// Macros
use crate::const_expr;

// Start address of the segment for this builtin.
// TODO: receive this at proof time as a public param. Until public params
// are implemented, have it as a dummy constant for testing.
pub const DUMMY_BITWISE_SEGMENT_START: u32 = 500;
// Each bitwise operation consists of 5 cells (two inputs and three outputs - and, or, xor).
pub const CELLS_PER_BITWISE: u32 = 5;

// The builtin will enforce:
// mem[addr] & mem[addr + 1] == mem[addr + 2]
// mem[addr] ^ mem[addr + 1] == mem[addr + 3]
// mem[addr] | mem[addr + 1] == mem[addr + 4]
// where addr = first_addr + 5 * i for 0 <= i < base.size,
// where {&, |, ^} are bitwise {and, or, xor} respectively on 251-bit integers (the field elements
// are guaranteed to be representable as 251-bit integers).
#[derive(Debug, InstDef, Default)]
pub struct BitwiseBuiltin {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for BitwiseBuiltin {
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _input: Self::In) -> Self::Out {
        let instance_num = air_builder.call_external_column(&Seq {});

        let read_felt252 = ReadPositive {
            num_bits: 252,
            memory: self.memory.clone(),
        };
        let verify_felt252 = MemVerify {
            memory: self.memory.clone(),
        };
        let (a, _) = air_builder.call(&read_felt252, get_addr(instance_num.clone(), 0));
        let (b, _) = air_builder.call(&read_felt252, get_addr(instance_num.clone(), 1));
        let mut expected_xor = vec![];
        let mut expected_and = vec![];
        let mut expected_or = vec![];
        for (a, b) in a.as_felts().into_iter().zip(b.as_felts().into_iter()) {
            let a_xor_b = air_builder.call(
                &BitwiseXor {
                    num_bits: FELT252_BITS_PER_WORD,
                },
                [a.clone(), b.clone()],
            );
            let a_and_b = air_builder
                .let_for_constraint((const_expr!(1) / const_expr!(2)) * (a + b - a_xor_b.clone()));
            expected_xor.push(a_xor_b.clone());
            expected_and.push(a_and_b.clone());
            expected_or.push(a_and_b + a_xor_b);
        }
        air_builder.call(
            &verify_felt252,
            (get_addr(instance_num.clone(), 2), expected_and.into()),
        );
        air_builder.call(
            &verify_felt252,
            (get_addr(instance_num.clone(), 3), expected_xor.into()),
        );
        air_builder.call(
            &verify_felt252,
            (get_addr(instance_num.clone(), 4), expected_or.into()),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}

pub fn get_addr(instance_num: FeltExpr, offset: u32) -> FeltExpr {
    const_expr!(DUMMY_BITWISE_SEGMENT_START)
        + instance_num * const_expr!(CELLS_PER_BITWISE)
        + const_expr!(offset)
}
