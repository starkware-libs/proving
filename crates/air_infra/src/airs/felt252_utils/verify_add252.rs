use inst_def::InstDef;
use prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS, P_FELTS};

// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

/// Verifying that two 252-bit felts sum to a third.
/// The function assumes all inputs have range-checked limbs.
/// None of the inputs are constrained to be fully reduced, but a + b - c may equal only 0 or P.
#[derive(Clone, Debug, InstDef)]
pub struct VerifyAdd252 {}

impl AirFn for VerifyAdd252 {
    type ExtIn = ();
    type In = [Felt252Expr; 3];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), [a, b, c]: Self::In) -> Self::Out {
        let shift = const_expr!(1 << FELT252_BITS_PER_WORD);
        let shift_inverse = const_expr!(1) / shift.clone();

        // In the least-significant word of the computation, since p[0] = 1, we have
        //   a[0] + b[0] - c[0] = carry[0] + sub_p_bit
        // where sub_p_bit is either 0 or 1, and carry[0] is 2**FELT252_BITS_PER_WORD times
        // 0, 1, or -1 (the latter happens if and only if a[0] = b[0] = 0, sub_p_bit = 1 and
        // c[0] = 2**FELT252_BITS_PER_WORD - 1).
        // Therefore, sub_p_bit can be extracted as the least significant bit of
        // a[0] + b[0] - c[0] mod 2**FELT252_BITS_PER_WORD, or equivalently as the LSB of
        // a[0] ^ b[0] ^ c[0], when taken as UInt16Expr-s.
        let mut sub_p_bit_u16 = const_u16_expr!(1)
            & (UInt16Expr::from(a.get_felt(0))
                ^ UInt16Expr::from(b.get_felt(0))
                ^ UInt16Expr::from(c.get_felt(0)));
        sub_p_bit_u16 = air_builder.let_for_deduction(sub_p_bit_u16, "sub_p_bit");
        let sub_p_bit = air_builder.deduce(sub_p_bit_u16.as_felt_mut(), "sub_p_bit");
        air_builder.constrain(
            sub_p_bit.clone() * (sub_p_bit.clone() - const_expr!(1)),
            "sub_p_bit is a bit",
        );

        let mut prev_carry = const_expr!(0);
        for (i, &p_felt) in P_FELTS.iter().enumerate().take(FELT252_N_WORDS - 1) {
            let mut carry = a.get_felt(i) + b.get_felt(i) + prev_carry
                - c.get_felt(i)
                - const_expr!(p_felt) * sub_p_bit.clone();
            carry = air_builder.let_for_constraint(carry * shift_inverse.clone(), "carry");
            air_builder.constrain(
                carry.clone() * (carry.clone() * carry.clone() - const_expr!(1)),
                "",
            );
            prev_carry = carry;
        }
        let i = FELT252_N_WORDS - 1;
        air_builder.constrain(
            a.get_felt(i) + b.get_felt(i) + prev_carry
                - c.get_felt(i)
                - const_expr!(P_FELTS[i]) * sub_p_bit,
            "",
        );
    }
}
