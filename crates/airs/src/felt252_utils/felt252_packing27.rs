use air_common::TraceType;
use air_infra::const_expr;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::constraint_connectedness_test;
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::felt252width27_expr::Felt252Width27Expr;
use air_infra::core::variables::AirVar;
use air_infra::felt252_id_memory::id_to_big::RangeCheckMemValue;
use air_infra::range_check::{range_check, range_check_variant};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::{
    FELT252_BITS_PER_WORD, FELT252_N_WORDS, FELT252WIDTH27_N_WORDS,
};

/// Unpacks a Felt252Width27Expr into a Felt252Expr.
/// If the range_check_output flag is set, also range checks the unpacked limbs.
#[derive(Clone, Debug, Serialize)]
pub struct Felt252UnpackFrom27 {
    #[serde(skip_serializing_if = "air_common::utils::is_false")]
    pub range_check_output: bool,
}

impl AirFn for Felt252UnpackFrom27 {
    type ExtIn = ();
    type In = Felt252Width27Expr;
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), packed: Self::In) -> Self::Out {
        // Deduce the limbs of the unpacked form.
        let mut a: Felt252Expr = packed.clone().into();
        a = air_builder.let_for_deduction(a, "input_as_felt252");
        assert_eq!(FELT252_N_WORDS % 3, 1);
        let mut v = Vec::new();
        for (i, a_limb) in a.as_felts_mut().into_iter().enumerate().take(FELT252_N_WORDS - 1) {
            if i % 3 != 2 {
                air_builder.deduce(a_limb, &format!("unpacked_limb_{i}"));
                v.push(a_limb.clone());
            }
            // Every third limb doesn't have to be deduced, as it is a linear combination of
            // previously deduced limbs and a limb of the packed form.
            else {
                let limb = packed.get_felt((i - 2) / 3)
                    - v[i - 2].clone()
                    - v[i - 1].clone() * const_expr!(1 << FELT252_BITS_PER_WORD);
                v.push(limb / const_expr!(1 << (2 * FELT252_BITS_PER_WORD)));
            }
        }
        // The final limb is the same between the two forms, so does not need to be deduced again.
        v.push(packed.get_felt(FELT252WIDTH27_N_WORDS - 1));
        let mut unpacked: Felt252Expr = v.into();

        if self.range_check_output {
            // Range check the unpacked form. We apply "let_" here so that the logic isn't
            // duplicated between the rangecheck lookups and the return value.
            unpacked = air_builder.let_(unpacked, "unpacked");
            air_builder.call(
                &RangeCheckMemValue::<FELT252_N_WORDS>::new(),
                unpacked
                    .as_felts()
                    .try_into()
                    .expect("Expected 'FELT252_N_WORDS' limbs in felt252"),
            );
        }

        unpacked
    }
}

/// Packs a Felt252Expr into a Felt252Width27Expr.
pub fn felt252_pack_into27(unpacked: Felt252Expr) -> Felt252Width27Expr {
    // The packing directly defines each packed limb as a linear combination of the limbs of the
    // unpacked form, and thus does not require any deductions or constraints.
    let mut v = Vec::new();
    let mut packed_limb = const_expr!(0);
    for (i, limb) in unpacked.as_felts().into_iter().enumerate() {
        let offset = i % 3;
        packed_limb = if offset == 0 {
            limb
        } else {
            packed_limb + limb * const_expr!(1 << (offset * FELT252_BITS_PER_WORD))
        };
        if offset == 2 {
            v.push(packed_limb.clone());
        }
    }
    if !FELT252_N_WORDS.is_multiple_of(3) {
        v.push(packed_limb);
    }

    v.into()
}

/// Rangechecks a Felt252Width27Expr by partial unpacking.
#[derive(Clone, Debug, Serialize)]
pub struct RangeCheck252Width27 {}

impl AirFn for RangeCheck252Width27 {
    type ExtIn = ();
    type In = Felt252Width27Expr;
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), packed: Self::In) -> Self::Out {
        // The constraint graph here is not connected: each pair of 27-bit words is range-checked
        // independently and, indeed, we could perform the same operations by 5 calls to a thinner
        // component that checks a single pair.
        // This is not done because such component would have too many rows, thus limiting the
        // number of Poseidon hashes we can perform in a single proof.
        constraint_connectedness_test::exclude(self);

        let mut a: Felt252Expr = packed.clone().into();
        a = air_builder.let_for_deduction(a, "input_as_felt252");
        for (j, i) in (0..(FELT252WIDTH27_N_WORDS)).step_by(2).enumerate() {
            let low_high =
                air_builder.deduce(a.get_felt_mut(3 * i + 2), &format!("limb_{i}_high_part"));
            let high_low = if i < FELT252WIDTH27_N_WORDS - 2 {
                air_builder.deduce(a.get_felt_mut(3 * i + 3), &format!("limb_{}_low_part", i + 1))
            } else {
                packed.get_felt(i + 1)
            };
            range_check_variant(air_builder, &[9, 9], &[low_high.clone(), high_low.clone()], j % 8);
            range_check_variant(
                air_builder,
                &[18],
                &[packed.get_felt(i) - low_high * const_expr!(1 << (2 * FELT252_BITS_PER_WORD))],
                j % 2,
            );
            if i < FELT252WIDTH27_N_WORDS - 2 {
                range_check(
                    air_builder,
                    &[18],
                    &[(packed.get_felt(i + 1) - high_low)
                        / const_expr!(1 << FELT252_BITS_PER_WORD)],
                );
            }
        }
    }
}
