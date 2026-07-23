use air_common::TraceType;
use air_infra::casm_state::CasmAddress;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::public_params::PublicParam;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::felt252_id_memory::verify::MemVerify;
use air_infra::seq::Seq;
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::casm::bitwise_xor::bitwise_xor::*;

// Each bitwise operation consists of 5 cells (two inputs and three outputs - and, or, xor).
pub const CELLS_PER_BITWISE: u32 = 5;

// The builtin will enforce:
// mem[addr] & mem[addr + 1] == mem[addr + 2]
// mem[addr] ^ mem[addr + 1] == mem[addr + 3]
// mem[addr] | mem[addr + 1] == mem[addr + 4]
// where addr = first_addr + 5 * i for 0 <= i < base.size,
// where {&, |, ^} are bitwise {and, or, xor} respectively on 251-bit integers (the field elements
// are guaranteed to be representable as 251-bit integers).
#[derive(Debug, Serialize, Default)]
pub struct BitwiseBuiltin {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for BitwiseBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = air_builder.call_external_table(&Seq {});
        let segment_start = air_builder.get_public_param(PublicParam::BitwiseBuiltinSegmentStart);

        let verify_felt252 = MemVerify { memory: self.memory.clone() };
        let a = self.memory.read_felt252(
            air_builder,
            CasmAddress::new(get_addr(segment_start.clone(), instance_num.clone(), 0), "op0"),
        );
        let b = self.memory.read_felt252(
            air_builder,
            CasmAddress::new(get_addr(segment_start.clone(), instance_num.clone(), 1), "op1"),
        );
        let mut expected_xor = vec![];
        let mut expected_and = vec![];
        let mut expected_or = vec![];
        for (i, (a, b)) in a.as_felts().into_iter().zip(b.as_felts().into_iter()).enumerate() {
            let num_bits = if i == (FELT252_N_WORDS - 1) {
                // The entries should each be 251 bits.
                FELT252_BITS_PER_WORD - 1
            } else {
                FELT252_BITS_PER_WORD
            };
            let a_xor_b = air_builder.call(&BitwiseXor::new(num_bits), [a.clone(), b.clone()]);
            let a_and_b =
                air_builder.let_((const_expr!(2).inverse()) * (a + b - a_xor_b.clone()), "and");
            expected_xor.push(a_xor_b.clone());
            expected_and.push(a_and_b.clone());
            expected_or.push(a_and_b + a_xor_b);
        }
        air_builder.call(
            &verify_felt252,
            (
                CasmAddress::new(get_addr(segment_start.clone(), instance_num.clone(), 2), "and"),
                expected_and.into(),
            ),
        );
        air_builder.call(
            &verify_felt252,
            (
                CasmAddress::new(get_addr(segment_start.clone(), instance_num.clone(), 3), "xor"),
                expected_xor.into(),
            ),
        );
        air_builder.call(
            &verify_felt252,
            (
                CasmAddress::new(get_addr(segment_start.clone(), instance_num.clone(), 4), "or"),
                expected_or.into(),
            ),
        );
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}

pub fn get_addr(segment_start: FeltExpr, instance_num: FeltExpr, offset: u32) -> FeltExpr {
    segment_start + instance_num * const_expr!(CELLS_PER_BITWISE) + const_expr!(offset)
}
