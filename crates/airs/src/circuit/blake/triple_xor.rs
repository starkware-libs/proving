use air_common::{TraceType, UseOrYield};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use serde::Serialize;

use crate::casm::bitwise_xor::bitwise_xor::*;
use crate::casm::bitwise_xor::verify_bitwise_xor::verify_bitwise_xor;
use crate::casm::opcodes::blake::split16::*;
use crate::circuit::ext_tables::*;

pub const N_BITS_IN_PART: u16 = 8;

/// Triple Xor gate verifies that `d = a ^ b ^ c` for the given 4 [`UInt32Expr`] inputs.
#[derive(Clone, Debug, Serialize)]
pub struct TripleXor {}

impl AirFn for TripleXor {
    type ExtIn = ();
    type In = [UInt32Expr; 4];
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), [a, b, c, d]: Self::In) -> Self::Out {
        // Split each limb into two 8-bit parts to look up into the bitwise xor table.
        let split = Split16 { low_part_size: N_BITS_IN_PART as usize };
        let [all, alh] = ab.call(&split, a.low().clone());
        let [ahl, ahh] = ab.call(&split, a.high().clone());
        let [bll, blh] = ab.call(&split, b.low().clone());
        let [bhl, bhh] = ab.call(&split, b.high().clone());
        let [cll, clh] = ab.call(&split, c.low().clone());
        let [chl, chh] = ab.call(&split, c.high().clone());
        let [dll, dlh] = ab.call(&split, d.low().clone());
        let [dhl, dhh] = ab.call(&split, d.high().clone());

        // Xor a and b.
        let bitwise_xor = BitwiseXor { num_bits: N_BITS_IN_PART as usize, variant: 0 };
        let a_xor_b_ll = ab.call(&bitwise_xor, [all, bll]);
        let a_xor_b_lh = ab.call(&bitwise_xor, [alh, blh]);
        let a_xor_b_hl = ab.call(&bitwise_xor, [ahl, bhl]);
        let a_xor_b_hh = ab.call(&bitwise_xor, [ahh, bhh]);

        // Xor a_xor_b and c, and verify the result is d.
        verify_bitwise_xor(ab, N_BITS_IN_PART, [a_xor_b_ll, cll, dll], 0);
        verify_bitwise_xor(ab, N_BITS_IN_PART, [a_xor_b_lh, clh, dlh], 0);
        verify_bitwise_xor(ab, N_BITS_IN_PART, [a_xor_b_hl, chl, dhl], 0);
        verify_bitwise_xor(ab, N_BITS_IN_PART, [a_xor_b_hh, chh, dhh], 0);

        // Add the lookup constraints to the gate relation.
        let input_addr_0 = ab.call_external_table(&TripleXorInputAddr0 {});
        let input_addr_1 = ab.call_external_table(&TripleXorInputAddr1 {});
        let input_addr_2 = ab.call_external_table(&TripleXorInputAddr2 {});
        let output_addr = ab.call_external_table(&TripleXorOutputAddr {});
        let mult = ab.call_external_table(&TripleXorMultiplicity {});

        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_0.var.clone(), a.low().as_felt(), a.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_1.var.clone(), b.low().as_felt(), b.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr_2.var.clone(), c.low().as_felt(), c.high().as_felt()],
            UseOrYield::Use,
            const_expr!(1),
        );
        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![output_addr.var.clone(), d.low().as_felt(), d.high().as_felt()],
            UseOrYield::Yield,
            mult,
        );
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![
            Some("a".to_string()),
            Some("b".to_string()),
            Some("c".to_string()),
            Some("a_xor_b_xor_c".to_string()),
        ])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }
}
