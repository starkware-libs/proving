use inst_def::InstDef;
use stwo_cairo_common::prover_types::cpu::{
    FELT252WIDTH27_N_WORDS, FELT252_BITS_PER_WORD, FELT252_N_WORDS,
};

use crate::airs::casm::const_tables::range_check::*;
// Macros
use crate::const_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt252width27_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::felt252_id_memory::id_to_big::*;
use crate::core::variables::*;

/// Unpacks a Felt252Width27Expr into a Felt252Expr.
/// If the range_check_output flag is set, also range checks the unpacked limbs.
#[derive(Clone, Debug, InstDef)]
pub struct Felt252UnpackFrom27 {
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
        for (i, a_limb) in a
            .as_felts_mut()
            .into_iter()
            .enumerate()
            .take(FELT252_N_WORDS - 1)
        {
            if i % 3 != 2 {
                air_builder.deduce(a_limb, &format!("unpacked_limb_{}", i));
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
        let unpacked: Felt252Expr = v.into();

        if self.range_check_output {
            // Range check the unpacked form.
            air_builder.call(
                &RangeCheckMemValue::<FELT252_N_WORDS> {},
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
    if FELT252_N_WORDS % 3 != 0 {
        v.push(packed_limb);
    }

    v.into()
}

/// Rangechecks a Felt252Width27Expr by partial unpacking.
#[derive(Clone, Debug, InstDef)]
pub struct RangeCheckFelt252Width27 {}

impl AirFn for RangeCheckFelt252Width27 {
    type ExtIn = ();
    type In = Felt252Width27Expr;
    type Out = ();

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }

    fn call(&self, air_builder: &mut AirBuilder, _: (), packed: Self::In) -> Self::Out {
        let mut a: Felt252Expr = packed.clone().into();
        a = air_builder.let_for_deduction(a, "input_as_felt252");
        for i in (0..(FELT252WIDTH27_N_WORDS)).step_by(2) {
            let low_high =
                air_builder.deduce(a.get_felt_mut(3 * i + 2), &format!("limb_{}_high_part", i));
            let high_low = if i < FELT252WIDTH27_N_WORDS - 2 {
                air_builder.deduce(
                    a.get_felt_mut(3 * i + 3),
                    &format!("limb_{}_low_part", i + 1),
                )
            } else {
                packed.get_felt(i + 1)
            };
            range_check(air_builder, &[9, 9], &[low_high.clone(), high_low.clone()]);
            range_check(
                air_builder,
                &[18],
                &[packed.get_felt(i) - low_high * const_expr!(1 << (2 * FELT252_BITS_PER_WORD))],
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
