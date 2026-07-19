// use core::range::Range;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::range_check::range_check;
use air_infra::{const_expr, const_felt252_expr};
use serde::Serialize;
use serde_arrays;
use stwo_cairo_common::prover_types::cpu::{
    FELT252WIDTH27_BITS_PER_WORD, FELT252WIDTH27_N_WORDS, P_PACKED27_FELTS,
};

/// Computes and verifies small linear combinations of packed felts.
/// The air assumes the inputs are appropriately range checked (to 27 bits on the first 9 limbs)
/// and that the output will be range checked as well, but does not perform this range check itself.
///
/// The air verifies that the given combination, minus the result, is a small multiple of P, with
/// factor `p_coef`. The bounds on `p_coef` (and some of the bounds of the carries) are derived
/// under assumption that the inputs are in reduced form, and are guaranteed to be satisfiable under
/// that assumption, but might not be satisfiable for non-reduced inputs in extreme cases.
///
/// Soundness (that the linear combination must be equivalent to the output modulo P) is guaranteed
/// as long as the input and output are range-checked, regardless of whether they are reduced.
#[derive(Clone, Debug, Serialize)]
pub struct LinearCombination<const N: usize> {
    n: usize,
    #[serde(with = "serde_arrays")]
    coefs: [i32; N],
}

impl<const N: usize> LinearCombination<N> {
    pub fn new(coefs: [i32; N]) -> Self {
        Self { n: N, coefs }
    }
}

impl<const N: usize> AirFn for LinearCombination<N> {
    type ExtIn = ();
    type In = [Felt252Width27Expr; N];
    type Out = Felt252Width27Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), inputs: Self::In) -> Self::Out {
        // Comnpute and deduce the result of the combination.
        let mut res = const_felt252_expr!(0, 0);
        // The sums of positive and negative coefficients (in absolute value).
        // The negative coefficients include a -1 from the coefficient of `res`.
        let mut sum_pos_coefs = 0u32;
        let mut sum_neg_coefs = 1u32;
        // Compute the linear combination (as Felt252Expr) and the coefficient sums.
        // Note that the Felt252Expr-s are used only for prover-side arithmetic: only the packed
        // forms appear in the trace.
        for (j, &c) in self.coefs.iter().enumerate() {
            if c > 0 {
                res = res + const_felt252_expr!(c as u128, 0) * inputs[j].clone().into();
                sum_pos_coefs += c as u32;
            } else {
                res = res - const_felt252_expr!((-c) as u128, 0) * inputs[j].clone().into();
                sum_neg_coefs += (-c) as u32;
            }
        }
        assert!(sum_neg_coefs + sum_pos_coefs <= 15);
        // The sums of coefficients provide bounds for the carries and the coefficient of P:
        // They are all bounded in the range [-sum_neg_coefs, sum_pos_coefs].
        // (some bounds are even tighter, e.g. p_coef is usually in (-sum_neg_coefs, sum_pos_coefs),
        // but the loose bound is sufficient for soundness and completeness.)
        // We use these bounds for range_checking the values as well as extracting p_coef.
        //
        // The bound on p_coef arises from the identity
        //   (*) sum_j coefs[j] * input[j] - res = p_coef * p
        // and the reduced-form assumption that the inputs and res are all in [0, p), implying the
        // LHS of (*) is in [-sum_neg_coefs * (p - 1), sum_pos_coefs * (p - 1)], which generally
        // implies p_coef is in (-sum_neg_coefs, sum_pos_coefs). The exception to this rule is if
        // sum_pos_coefs = 0, where the range is (-sum_neg_coefs, 0].
        // If the reduced-form assumption is not held, then it is possible for p_coef to be outside
        // the bounded range (it can be up to roughly twice as large), and therefore the checked
        // bounds will not be satisfiable; but if p_coef happens to fall in the bounded range, then
        // all other constraints will remain valid due to the range checks.
        //
        // The bounds on the carries arise from the identities
        //   (*i) sum_j coefs[j]*input[j][i] - res[i] - p_coef*p[i] + carry[i-1] = 2**27 * carry[i].
        // Due to the range checked facts that input[j][i], res[i] are in [0, 2**27), which imply
        //   (sum_j coefs[j]*input[j][i] - res[i]) is in the range
        //   [-sum_neg_coefs * (2**27 - 1), sum_pos_coefs * (2**27 - 1)],
        // together with the range checked facts that carry[i-1] and p_coef are very small
        // (at most 15 in absolute value) and the fact that p[i]-s are small (at most 256), we have
        // that the LHS of (*i) is bounded well within the range
        //   (-(sum_neg_coefs + 1) * 2**27, (sum_pos_coefs + 1) * 2**27)
        // therefore carry[i] is in [-sum_neg_coefs, sum_pos_coefs].
        // In fact, the bounding range for the LHS is only slightly larger than
        //   [-sum_neg_coefs * 2**27, sum_pos_coefs * 2**27],
        // by at most 2**12 (upper bound on p_coef*p[i]) to either side. The requirement that
        // sum_neg_coefs + sum_pos_coefs <= 15 guarantees the soundness of constraining the LHS
        // to equal 2**27 * carry[i] where carry[i] is range checked to be in some subinterval
        // of [-sum_neg_coefs, -sum_neg_coefs + 15], in the sense that overflow modulo 2**31 - 1
        // is impossible, as 15 * 2**27 + 2**12 < 2**31 - 1.
        //
        // From (*) we see that if either sum_neg_coefs <= 1 or sum_pos_coefs <= 1 and the inputs
        // and output are in reduced form then p_coef will have a fixed sign:
        // If sum_neg_coefs <= 1 then 0 <= p_coef < sum_pos_coefs, which then implies (using (*i)
        // and by induction on i) that all carries are bounded in [-sum_neg_coefs, sum_pos_coefs).
        // Similarly if sum_pos_coefs <= 1 then -sum_neg_coefs < p_coef <= 0 and the carries are
        // bounded in (-sum_neg_coefs, sum_pos_coefs].
        // Thus in these cases we can check smaller ranges by effectively decreasing the appropriate
        // bound. Note that at most one sum_coefs can be <= 1, since  (0, 1), (1, 0) and (1, 1) are
        // all degenerate cases of lincomb: (1, 0) is impossible as sum_neg_coefs >= 1 by its
        // definition; (0, 1) corresponds to the null combination (i.e. res = 0) and (1, 1) to the
        // trivial combination (i.e. res = input), neither of which should require an air call.
        let (minus_lower_bound, upper_bound) = if sum_neg_coefs <= 1 && sum_pos_coefs >= 2 {
            (sum_neg_coefs, sum_pos_coefs - 1)
        } else if sum_neg_coefs >= 2 && sum_pos_coefs <= 1 {
            (sum_neg_coefs - 1, sum_pos_coefs)
        } else {
            (sum_neg_coefs, sum_pos_coefs)
        };

        // Deduce the linear combination result in packed form.
        let mut res: Felt252Width27Expr = res.into();
        res = air_builder.deduce_air_var(res, "combination");

        let shift = const_expr!(1 << FELT252WIDTH27_BITS_PER_WORD);
        let shift_inverse = const_expr!(1) / shift.clone();

        let mut limb_accumulator = const_expr!(0);
        let mut carry_vec = Vec::new();
        let mut p_coef = const_expr!(0);
        for (i, &p_felt) in P_PACKED27_FELTS.iter().enumerate() {
            for (j, &c) in self.coefs.iter().enumerate() {
                limb_accumulator = if c > 0 {
                    limb_accumulator + const_expr!(c as u32) * inputs[j].get_felt(i)
                } else {
                    limb_accumulator - const_expr!((-c) as u32) * inputs[j].get_felt(i)
                };
            }
            limb_accumulator = limb_accumulator - res.get_felt(i);
            if i == 0 {
                // The first limb allows us to extract p_coef, using the fact that p = 1 (mod
                // 2**27). The value that limb accumulator currently holds therefore equals
                // 2**27 * carry_0 + p_coef * 1. Thus we would liked to reduce limb accumulator mod
                // 2**27 to obtain p_coef. Because both p_coef and carry_0 might be negative, we
                // first bias the limb accumulator by adding (2**27 + 1) * minus_lower_bound (as
                // -minus_lower_bound is a lower bound for both p_coef and carry_0). Then we convert
                // the biased limb accumulator into a u32, and deduce p_coef by unbiasing its low
                // part (no further reduction is necessary as 27 > 16).
                let biased_limb_accumulator = limb_accumulator.clone()
                    + (shift.clone() + const_expr!(1)) * const_expr!(minus_lower_bound);
                let mut biased_limb_accumulator_u32: UInt32Expr = biased_limb_accumulator.into();
                biased_limb_accumulator_u32 = air_builder
                    .let_for_deduction(biased_limb_accumulator_u32, "biased_limb_accumulator_u32");
                p_coef = air_builder.deduce(
                    &mut (biased_limb_accumulator_u32.low().as_felt()
                        - const_expr!(minus_lower_bound)),
                    "p_coef",
                );
                // Include p_coef in the vector of carries to be range checked.
                carry_vec.push(p_coef.clone());
            }
            limb_accumulator = limb_accumulator - p_coef.clone() * const_expr!(p_felt);
            if i < FELT252WIDTH27_N_WORDS - 1 {
                limb_accumulator = air_builder
                    .let_(limb_accumulator * shift_inverse.clone(), &format!("carry_{i}"));
                carry_vec.push(limb_accumulator.clone());
            }
        }
        air_builder.constrain(limb_accumulator, "final limb constraint");

        // Range checking p_coef and the carries.
        assert_eq!(carry_vec.len(), FELT252WIDTH27_N_WORDS);
        match upper_bound + minus_lower_bound {
            3..16 => {
                let (range_bit_size, max_range_check_len) = match upper_bound + minus_lower_bound {
                    // We need to range check a total of 10 elements (p_coef + 9 carries) to the
                    // same bit size, using the minimal number of lookups. For 4-bit elements, we
                    // group them as [[4, 4, 4, 4], [4, 4, 4, 4], [4, 4]] since a [4, 4, 4, 4, 4]
                    // table is too large. For bit sizes 3 and 2, we use two groups of 5, as a
                    // single group of 10 is too large.
                    // max_range_check_len is the length of the longest such vector checks; the
                    // division into groups is computed directly from it.
                    8..16 => (4, 4),
                    4..8 => (3, 5),
                    3 => (2, 5),
                    _ => panic!("Unreachable code"),
                };
                for start in (0..FELT252WIDTH27_N_WORDS).step_by(max_range_check_len) {
                    let range =
                        start..std::cmp::min(start + max_range_check_len, FELT252WIDTH27_N_WORDS);
                    let bits = range.clone().map(|_| range_bit_size).collect::<Vec<u16>>();
                    let input = range
                        .map(|i| carry_vec[i].clone() + const_expr!(minus_lower_bound))
                        .collect::<Vec<FeltExpr>>();
                    range_check(air_builder, &bits, &input);
                }
            }
            // In these cases the ranges are of size at most 3, so they can be verified with
            // constraints rather than range checks.
            2 => {
                for (i, carry) in carry_vec.into_iter().enumerate() {
                    // Bias and constrain the carries in the range [-1, 1].
                    let biased_carry = air_builder.let_for_constraint(
                        carry + (const_expr!(minus_lower_bound) - const_expr!(1)),
                        &format!("biased_carry_{i}"),
                    );
                    air_builder.constrain(
                        biased_carry.clone() * biased_carry.clone() * biased_carry.clone()
                            - biased_carry,
                        &format!("carry constraint {i}"),
                    );
                }
            }
            1 => {
                for (i, carry) in carry_vec.into_iter().enumerate() {
                    // Bias and constrain the carries in the range [0, 1].
                    let biased_carry = air_builder.let_for_constraint(
                        carry + const_expr!(minus_lower_bound),
                        &format!("biased_carry_{i}"),
                    );
                    air_builder.constrain(
                        biased_carry.clone() * biased_carry.clone() - biased_carry,
                        &format!("carry constraint {i}"),
                    );
                }
            }
            _ => {
                panic!("Coefficient sizes not supported");
            }
        }

        res
    }
}
