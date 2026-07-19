use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{
    FELT252_BITS_PER_WORD, FELT252_N_WORDS, P_PACKED27_FELTS,
};

/// Verifying that two 252-bit felts sum to a third.
/// The function assumes all inputs have range-checked limbs.
/// None of the inputs are constrained to be fully reduced, but a + b - c may equal only 0 or P.
#[derive(Clone, Debug, Serialize)]
pub struct VerifyAdd252 {}

impl AirFn for VerifyAdd252 {
    type ExtIn = ();
    type In = [Felt252Expr; 3];
    type Out = ();

    fn input_expr_descriptions(&self) -> Option<Vec<Option<String>>> {
        Some(vec![Some("a".to_string()), Some("b".to_string()), Some("c".to_string())])
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let shift = const_expr!(1 << FELT252_BITS_PER_WORD);
        let shift_inverse = shift.clone().inverse();

        // In the least-significant word of the computation, since p[0] = 1, we have
        //   a[0] + b[0] - c[0] = carry[0] + sub_p_bit
        // where sub_p_bit is either 0 or 1, and carry[0] is 2**FELT252_BITS_PER_WORD times
        // 0, 1, or -1 (the latter happens if and only if a[0] = b[0] = 0, sub_p_bit = 1 and
        // c[0] = 2**FELT252_BITS_PER_WORD - 1).
        // Therefore, sub_p_bit can be extracted as the least significant bit of
        // a[0] + b[0] - c[0] mod 2**FELT252_BITS_PER_WORD, or equivalently as the LSB of
        // a[0] ^ b[0] ^ c[0], when taken as UInt16Expr-s.
        let mut sub_p_bit = const_u16_expr!(1)
            & (UInt16Expr::from(a.get_felt(0))
                ^ UInt16Expr::from(b.get_felt(0))
                ^ UInt16Expr::from(c.get_felt(0)));
        sub_p_bit = air_builder.deduce_air_var(sub_p_bit, "sub_p_bit");
        air_builder.constrain(
            sub_p_bit.as_felt() * (sub_p_bit.as_felt() - const_expr!(1)),
            "sub_p_bit is a bit",
        );

        let mut carry = const_expr!(0);
        for i in 0..(FELT252_N_WORDS - 1) {
            // It suffices to verify the carry only every third limb. This is equivalent to carrying
            // out the computation with limbs of size 27 instead of 9 bits, which is still sound.
            // Similarly p * sub_p_bit can be directly subtracted as 27-bit limbs every third step.
            carry = a.get_felt(i) + b.get_felt(i) + carry - c.get_felt(i);
            if i.is_multiple_of(3) {
                carry = carry - const_expr!(P_PACKED27_FELTS[i / 3]) * sub_p_bit.as_felt();
            }
            carry = carry * shift_inverse.clone();
            if i % 3 == 2 {
                carry = air_builder.let_for_constraint(carry, "carry");
                air_builder.constrain(
                    carry.clone() * (carry.clone() * carry.clone() - const_expr!(1)),
                    "",
                );
            }
        }
        let i = FELT252_N_WORDS - 1;
        assert!(i.is_multiple_of(3));
        air_builder.constrain(
            a.get_felt(i) + b.get_felt(i) + carry
                - c.get_felt(i)
                - const_expr!(P_PACKED27_FELTS[i / 3]) * sub_p_bit.as_felt(),
            "",
        );
    }
}
