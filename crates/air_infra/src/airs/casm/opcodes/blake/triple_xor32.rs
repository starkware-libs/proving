use compiled_casm_air::compiled_structs::TraceType;
use inst_def::InstDef;

use crate::airs::casm::bitwise_xor::*;
use crate::airs::casm::opcodes::blake::split16::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;

/// Receives a, b and c each as UInt32 which is a pair of felts - low and high.
/// Returns their bitwixe xor.
#[derive(Clone, Debug, InstDef)]
pub struct TripleXor32 {}

impl AirFn for TripleXor32 {
    type ExtIn = ();
    type In = [UInt32Expr; 3];
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let split = Split16 { low_part_size: 8 };
        let [all, alh] = air_builder.call(&split, a.low());
        let [ahl, ahh] = air_builder.call(&split, a.high());
        let [bll, blh] = air_builder.call(&split, b.low());
        let [bhl, bhh] = air_builder.call(&split, b.high());
        let [cll, clh] = air_builder.call(&split, c.low());
        let [chl, chh] = air_builder.call(&split, c.high());

        let tmp_rll = air_builder.call(&BitwiseXor { num_bits: 8 }, [all, bll]);
        let rll = air_builder.call(&BitwiseXor { num_bits: 8 }, [tmp_rll, cll]);

        let tmp_rlh = air_builder.call(&BitwiseXor { num_bits: 8 }, [alh, blh]);
        let rlh = air_builder.call(&BitwiseXor { num_bits: 8 }, [tmp_rlh, clh]);

        let tmp_rhl = air_builder.call(&BitwiseXor { num_bits: 8 }, [ahl, bhl]);
        let rhl = air_builder.call(&BitwiseXor { num_bits: 8 }, [tmp_rhl, chl]);

        let tmp_rhh = air_builder.call(&BitwiseXor { num_bits: 8 }, [ahh, bhh]);
        let rhh = air_builder.call(&BitwiseXor { num_bits: 8 }, [tmp_rhh, chh]);

        air_builder.let_(
            vec![
                rll + rlh * const_expr!(1 << 8),
                rhl + rhh * const_expr!(1 << 8),
            ]
            .into(),
            "triple_xor32_output",
        )
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn deduce_output(&self) -> Option<String> {
        // TODO(Stav): Implement this in stwo-cairo
        Some(format!(
            "{}::deduce_output",
            self.relation_name().expect("Relation name not found")
        ))
    }
}
