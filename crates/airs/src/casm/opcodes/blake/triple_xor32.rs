use air_common::TraceType;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::constraint_connectedness_test;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use serde::Serialize;

use crate::casm::bitwise_xor::bitwise_xor::*;
use crate::casm::opcodes::blake::split16::*;

/// Receives a, b and c each as UInt32 which is a pair of felts - low and high.
/// Returns their bitwixe xor.
#[derive(Clone, Debug, Serialize)]
pub struct TripleXor32 {}

impl AirFn for TripleXor32 {
    type ExtIn = ();
    type In = [UInt32Expr; 3];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        // The constraint graph here is not connected: the high halves of a,b,c are XORed
        // independently from the low halves.
        // The alternative is to have TripleXor16 and call it twice. This would save columns,
        // but add trace cells, and currently (Aug 2025) the tradeoff doesn't seem beneficial.
        constraint_connectedness_test::exclude(self);

        let split = Split16 { low_part_size: 8 };
        let [all, alh] = air_builder.call(&split, a.low());
        let [ahl, ahh] = air_builder.call(&split, a.high());
        let [bll, blh] = air_builder.call(&split, b.low());
        let [bhl, bhh] = air_builder.call(&split, b.high());
        let [cll, clh] = air_builder.call(&split, c.low());
        let [chl, chh] = air_builder.call(&split, c.high());

        let tmp_rll = air_builder.call(&BitwiseXor { num_bits: 8, variant: 0 }, [all, bll]);
        let rll = air_builder.call(&BitwiseXor { num_bits: 8, variant: 0 }, [tmp_rll, cll]);

        let tmp_rlh = air_builder.call(&BitwiseXor { num_bits: 8, variant: 0 }, [alh, blh]);
        let rlh = air_builder.call(&BitwiseXor { num_bits: 8, variant: 0 }, [tmp_rlh, clh]);

        let tmp_rhl = air_builder.call(&BitwiseXor { num_bits: 8, variant: 1 }, [ahl, bhl]);
        let rhl = air_builder.call(&BitwiseXor { num_bits: 8, variant: 1 }, [tmp_rhl, chl]);

        let tmp_rhh = air_builder.call(&BitwiseXor { num_bits: 8, variant: 1 }, [ahh, bhh]);
        let rhh = air_builder.call(&BitwiseXor { num_bits: 8, variant: 1 }, [tmp_rhh, chh]);

        air_builder.let_(
            vec![rll + rlh * const_expr!(1 << 8), rhl + rhh * const_expr!(1 << 8)].into(),
            "triple_xor32_output",
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
