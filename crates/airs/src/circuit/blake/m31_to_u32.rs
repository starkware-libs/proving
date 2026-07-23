use std::slice::from_ref;

use air_common::{TraceType, UseOrYield};
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::range_check::range_check;
use serde::Serialize;

use crate::circuit::ext_tables::*;

#[derive(Clone, Debug, Serialize)]
pub struct M31ToU32 {}

/// Verifies that the given 'UInt32Expr' represents the given 'M31' value,
/// i.e it contains the 16 low bits of the 'M31' as its low limb, and the 15 high bits as its
/// high limb.
impl AirFn for M31ToU32 {
    type ExtIn = ();
    type In = (FeltExpr, UInt32Expr);
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), (m31, u32_expr): Self::In) -> Self::Out {
        // Range check limb low to be 16 bits.
        range_check(ab, &[16], from_ref(&u32_expr.low().as_felt()));

        // Range check limb high to be 15 bits with 2 range checks 16.
        range_check(ab, &[16], from_ref(&u32_expr.high().as_felt()));
        range_check(
            ab,
            &[16],
            &[(const_expr!(1 << 15) - const_expr!(1)) - u32_expr.high().as_felt()],
        );

        // Constrain that if the  m31 input is zero mod P then the low limb and the high limb are
        // zero. Without this constraint a zero can be represented with low limb = 0x7fff
        // and high limb = 0xffff.
        let is_zero = ab.let_for_deduction(const_expr!(0).eq(m31.clone()), "input_is_zero");
        let inv_val =
            ab.deduce(&mut (const_expr!(1) / (is_zero.as_felt() + m31.clone())), "inv_or_one");
        ab.constrain(
            (m31.clone() * inv_val - const_expr!(1)) * u32_expr.low().as_felt(),
            "input is zero then limb_low is zero",
        );

        // Reconstruct input from low and high parts
        ab.constrain(
            m31.clone()
                - (u32_expr.low().as_felt() + (u32_expr.high().as_felt() * const_expr!(1 << 16))),
            "input reconstruction",
        );

        let input_addr = ab.call_external_table(&M31ToU32InputAddr {});
        let output_addr = ab.call_external_table(&M31ToU32OutputAddr {});
        let mult = ab.call_external_table(&M31ToU32Multiplicity {});

        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![input_addr.var.clone(), m31],
            UseOrYield::Use,
            const_expr!(1),
        );

        ab.add_lookup_term(
            &self.relation_name().expect("Relation name not set"),
            vec![output_addr.var.clone(), u32_expr.low().as_felt(), u32_expr.high().as_felt()],
            UseOrYield::Yield,
            mult,
        );
    }

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("m31".to_string()), Some("u32".to_string())])
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Gate
    }
}
