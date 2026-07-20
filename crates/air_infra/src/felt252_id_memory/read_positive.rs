use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::memory::*;
use crate::casm_state::*;
//  Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::variables::*;
use crate::felt252_id_memory::read_id::*;
use crate::range_check::*;

#[derive(Debug, Serialize)]
pub struct ReadPositive {
    pub num_bits: usize,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 in the range [0,2**num_bits - 1] from the memory
/// Returns also the ID of the value in the memory.
impl AirFn for ReadPositive {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = (Felt252Expr, CasmId);

    fn call(&self, air_builder: &mut AirBuilder, _: (), address: Self::In) -> Self::Out {
        let id = air_builder.call(&ReadId { memory: self.memory.clone() }, address.clone());

        let expected_value_in_memory = air_builder.call(
            &ReadPositiveKnownId { num_bits: self.num_bits, memory: self.memory.clone() },
            id.clone(),
        );

        (expected_value_in_memory, id)
    }
}

#[derive(Debug, Serialize)]
pub struct ReadPositiveKnownId {
    pub num_bits: usize,
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 in the range [0,2**num_bits - 1] from the memory given its ID and verify it.
impl AirFn for ReadPositiveKnownId {
    type ExtIn = ();
    type In = CasmId;
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), id: Self::In) -> Self::Out {
        let mut value = air_builder.mem_read_unverified(&self.memory.id_to_big, &id);

        // Prepare for value deduction
        let num_nonzero_limbs = self.num_bits.div_ceil(FELT252_BITS_PER_WORD);
        let bits_in_ms_limb = self.num_bits % FELT252_BITS_PER_WORD;

        // Deduce the nonzero limbs
        for (i, limb) in value.as_felts_mut().into_iter().take(num_nonzero_limbs).enumerate() {
            air_builder.deduce(
                limb,
                &id.extra_info
                    .clone()
                    .map(|s| format!("{s}_limb_{i}"))
                    .unwrap_or(format!("value_limb_{i}")),
            );
        }

        // If required - range-check the most significant limb
        if bits_in_ms_limb > 0 {
            let msl = value.get_felt(num_nonzero_limbs - 1);
            air_builder.call(&RangeCheckLastLimb { bits_in_ms_limb }, msl);
        }

        let expected_value_in_memory = Felt252Expr::from(
            value.as_felts().into_iter().take(num_nonzero_limbs).collect::<Vec<_>>(),
        );

        // Verify that the value in memory is the nonzero limbs we deduced, padded on
        // the left with zeros.
        air_builder.mem_verify(&self.memory.id_to_big, &id, expected_value_in_memory.clone());

        expected_value_in_memory
    }
}

#[derive(Debug, Serialize)]
pub struct RangeCheckLastLimb {
    pub bits_in_ms_limb: usize,
}

impl AirFn for RangeCheckLastLimb {
    type ExtIn = ();
    type In = FeltExpr;
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), msl: Self::In) -> Self::Out {
        match self.bits_in_ms_limb {
            0 => (),
            1 => air_builder.constrain(
                msl.clone() * (const_expr!(1) - msl.clone()),
                "most significant limb is a bit",
            ),
            2 => air_builder.call(&CondRangeCheck2 {}, [msl, const_expr!(1)]),
            _ => range_check(air_builder, &[self.bits_in_ms_limb as u16], &[msl]),
        }
    }
}

/// Receives a FeltExpr, and conditionally constrains it to be at most 3.
#[derive(Debug, Serialize)]
pub struct CondRangeCheck2 {}

impl AirFn for CondRangeCheck2 {
    type ExtIn = ();
    type In = [FeltExpr; 2];
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), [msl, condition]: Self::In) -> Self::Out {
        let mslh = air_builder.deduce_air_var(
            (UInt16Expr::from(msl.clone()) & const_u16_expr!(0b10)) >> const_u16_expr!(1),
            "partial_limb_msb",
        );
        air_builder.constrain(
            mslh.as_felt() * (const_expr!(1) - mslh.as_felt()) * condition.clone(),
            "msb is a bit or condition is 0",
        );
        let msll = air_builder.let_for_constraint(
            msl - (mslh.as_felt() * const_expr!(2)),
            "partial_limb_bit_before_msb",
        );
        air_builder.constrain(
            msll.clone() * (const_expr!(1) - msll.clone()) * condition,
            "bit before msb is a bit or condition is 0",
        );
    }
}
