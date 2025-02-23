use inst_def::InstDef;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::memory::*;
use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::range_check::*;
//  Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::variables::*;

#[derive(Debug, InstDef)]
pub struct ReadPositive {
    pub num_bits: usize,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 in the range [0,2**num_bits - 1] from the memory
/// Returns also the ID of the value in the memory.
impl AirFn for ReadPositive {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = (Felt252Expr, FeltExpr);

    fn call(&self, air_builder: &mut AirBuilder, _: (), address: Self::In) -> Self::Out {
        let (mut value, mut id) = self.memory.read_unverified(air_builder, &address);

        // Deduce the ID as-is
        air_builder.deduce(
            &mut id,
            &address
                .desc
                .clone()
                .map(|s| format!("{}_id", s))
                .unwrap_or("id".to_string()),
        );
        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());

        // Prepare for value deduction
        let num_nonzero_limbs = self.num_bits.div_ceil(FELT252_BITS_PER_WORD);
        let bits_in_ms_limb = self.num_bits % FELT252_BITS_PER_WORD;

        // Deduce the nonzero limbs
        for (i, limb) in value
            .as_felts_mut()
            .into_iter()
            .take(num_nonzero_limbs)
            .enumerate()
        {
            air_builder.deduce(
                limb,
                &address
                    .desc
                    .clone()
                    .map(|s| format!("{}_limb_{}", s, i))
                    .unwrap_or(format!("limb_{}", i)),
            );
        }

        // If required - range-check the most significant limb
        if bits_in_ms_limb > 0 {
            let msl = value.get_felt(num_nonzero_limbs - 1);
            air_builder.call(&RangeCheckLastLimb { bits_in_ms_limb }, msl);
        }

        let expected_value_in_memory = Felt252Expr::from(
            value
                .as_felts()
                .into_iter()
                .take(num_nonzero_limbs)
                .collect::<Vec<_>>(),
        );

        // Verify that the value in memory is the nonzero limbs we deduced, padded on
        // the left with zeros.
        air_builder.mem_verify(
            &self.memory.id_to_big,
            &id,
            expected_value_in_memory.clone(),
        );

        (expected_value_in_memory, id)
    }
}

#[derive(Debug, InstDef)]
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
            2 => {
                let mslh = air_builder.deduce_air_var(
                    (UInt16Expr::from(msl.clone()) & const_u16_expr!(0b10)) >> const_u16_expr!(1),
                    "msb",
                );
                air_builder.constrain(
                    mslh.as_felt() * (const_expr!(1) - mslh.as_felt()),
                    "msb is a bit",
                );
                let msll = air_builder
                    .let_for_constraint(msl - (mslh.as_felt() * const_expr!(2)), "bit_before_msb");
                air_builder.constrain(
                    msll.clone() * (const_expr!(1) - msll.clone()),
                    "bit before msb is a bit",
                );
            }
            _ => range_check(air_builder, &[self.bits_in_ms_limb as u16], &[msl]),
        }
    }
}
