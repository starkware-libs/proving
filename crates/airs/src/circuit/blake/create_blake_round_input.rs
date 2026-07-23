use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::bool_expr::BoolExpr;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::{const_expr, const_u32_expr};
use serde::Serialize;

use crate::casm::bitwise_xor::bitwise_xor::*;
use crate::casm::opcodes::blake::split16::*;
use crate::circuit::ext_tables::*;

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

#[derive(Debug, Serialize)]
pub struct CreateBlakeRoundInput {}

impl AirFn for CreateBlakeRoundInput {
    type ExtIn = ();
    type In = ([UInt32Expr; 8], BoolExpr);
    type Out = [UInt32Expr; 16];

    fn call(&self, ab: &mut AirBuilder, _: (), (h, finalize_flag): Self::In) -> Self::Out {
        // The first 8 elements are exactly h.
        let mut state = h.to_vec();
        let [t_low, t_high] = ab.call_external_table(&BlakeT {});

        for (i, iv) in IV.iter().enumerate() {
            let current_state = match i + 8 {
                // state[12] = t XOR IV[4]
                12 => {
                    let split = Split16 { low_part_size: 8 };
                    let [tll, tlh] = ab.call(&split, UInt16Expr::from(t_low.clone()));
                    let [thl, thh] = ab.call(&split, UInt16Expr::from(t_high.clone()));

                    // Calculate and deduce the bitwise xor of the parts.
                    let bitwise_xor = BitwiseXor { num_bits: 8, variant: 0 };
                    let cll = ab.call(&bitwise_xor, [tll, const_expr!(IV4[0] as u32)]);
                    let clh = ab.call(&bitwise_xor, [tlh, const_expr!(IV4[1] as u32)]);
                    let chl = ab.call(&bitwise_xor, [thl, const_expr!(IV4[2] as u32)]);
                    let chh = ab.call(&bitwise_xor, [thh, const_expr!(IV4[3] as u32)]);

                    UInt32Expr::from(vec![
                        cll + clh * const_expr!(1 << 8),
                        chl + chh * const_expr!(1 << 8),
                    ])
                }

                // if 'IsLastBlock' is true, state[14] = STATE14_LAST_BLOCK, else state[14] = IV[6]
                14 => {
                    let low = finalize_flag.as_felt() * const_expr!(STATE14_LAST_BLOCK[0])
                        + (const_expr!(1) - finalize_flag.as_felt()) * const_expr!(IV6[0] as u32);
                    let high = finalize_flag.as_felt() * const_expr!(STATE14_LAST_BLOCK[1])
                        + (const_expr!(1) - finalize_flag.as_felt()) * const_expr!(IV6[1] as u32);
                    UInt32Expr::from(vec![low, high])
                }
                _ => const_u32_expr!(*iv),
            };
            state.push(current_state);
        }

        state.try_into().expect("State size is not 16 as expected")
    }
}
