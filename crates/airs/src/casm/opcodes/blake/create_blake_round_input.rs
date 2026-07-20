use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_u32_expr};
use serde::Serialize;

use super::read_u32::*;
use super::round::*;
use crate::casm::bitwise_xor::bitwise_xor::*;
use crate::casm::opcodes::blake::split16::*;

// The initialization vector.
pub const IV: [u32; 8] = [
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];

// The initialization vector[4] split into 4 u8.
pub const IV4: [u8; 4] = [0x7F, 0x52, 0x0E, 0x51];

// The initialization vector[6] split into 2 u16.
pub const IV6: [u16; 2] = [0xD9AB, 0x1F83];

// The value of state14 if it is the last block (IV[6] XOR 0xFFFFFFFF).
pub const STATE14_LAST_BLOCK: [u32; 2] = [0x2654, 0xE07C];

/// Creates a BlakeState from the BlakeCompress input, i.e, `h` (an array of 8 `u32` elements), `t`
/// (the count of bytes that have been compressed), and `IsLastBlock`.
#[derive(Debug, Serialize)]
pub struct CreateBlakeRoundInput {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

impl AirFn for CreateBlakeRoundInput {
    type ExtIn = ();
    type In = (CasmAddress, UInt32Expr, BoolExpr);
    type Out = BlakeState;

    fn call(
        &self,
        air_builder: &mut AirBuilder,
        _: (),
        (h_pointer, t, is_last_block): Self::In,
    ) -> Self::Out {
        let mut state = vec![];
        let read_u32 = &ReadU32 { memory: self.memory.clone() };

        // The first 8 elements are exactly `h`.
        for i in 0..8 {
            let current_addr =
                CasmAddress::new(h_pointer.var.clone() + const_expr!(i), &format!("state_{i}"));
            state.push(air_builder.call(read_u32, current_addr));
        }

        for (i, iv) in IV.iter().enumerate() {
            let current_state = match i + 8 {
                // state[12] = t XOR IV[4]
                12 => {
                    let split = Split16 { low_part_size: 8 };
                    let [tll, tlh] = air_builder.call(&split, t.low());
                    let [thl, thh] = air_builder.call(&split, t.high());

                    // Calculate and deduce the bitwise xor of the parts.
                    let bitwise_xor = BitwiseXor { num_bits: 8, variant: 0 };
                    let cll = air_builder.call(&bitwise_xor, [tll, const_expr!(IV4[0] as u32)]);
                    let clh = air_builder.call(&bitwise_xor, [tlh, const_expr!(IV4[1] as u32)]);
                    let chl = air_builder.call(&bitwise_xor, [thl, const_expr!(IV4[2] as u32)]);
                    let chh = air_builder.call(&bitwise_xor, [thh, const_expr!(IV4[3] as u32)]);

                    UInt32Expr::from(vec![
                        cll + clh * const_expr!(1 << 8),
                        chl + chh * const_expr!(1 << 8),
                    ])
                }

                // if 'IsLastBlock' is true, state[14] = STATE14_LAST_BLOCK, else state[14] = IV[6]
                14 => {
                    let low = is_last_block.as_felt() * const_expr!(STATE14_LAST_BLOCK[0])
                        + (const_expr!(1) - is_last_block.as_felt()) * const_expr!(IV6[0] as u32);
                    let high = is_last_block.as_felt() * const_expr!(STATE14_LAST_BLOCK[1])
                        + (const_expr!(1) - is_last_block.as_felt()) * const_expr!(IV6[1] as u32);
                    UInt32Expr::from(vec![low, high])
                }
                _ => const_u32_expr!(*iv),
            };
            state.push(current_state);
        }

        state.try_into().expect("State size is not 16 as expected")
    }
}
