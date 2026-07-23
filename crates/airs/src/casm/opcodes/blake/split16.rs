use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Split16 {
    pub low_part_size: usize,
}

// Split 'Uint16Expr', into the low 'low_part_size' bits and the high 16 - 'low_part_size' bits.
// Assumes the caller range check these parts.
impl AirFn for Split16 {
    type ExtIn = ();
    type In = UInt16Expr;
    type Out = [FeltExpr; 2];

    fn call(&self, air_builder: &mut AirBuilder, _: (), a: Self::In) -> Self::Out {
        assert!(self.low_part_size <= 16, "Invalid size for the low part");
        let low_size = const_u16_expr!(self.low_part_size as u16);

        // Calculate and deduce the high 16-'low_part_size' bits.
        let ah = air_builder.deduce_air_var(
            a.clone() >> low_size.clone(),
            &format!("ms_{}_bits", 16 - self.low_part_size),
        );

        // Calculate the low 'low_part_size' bits.
        let al = a.as_felt() - ah.as_felt() * const_expr!(1 << self.low_part_size);
        [al, ah.as_felt()]
    }
}
