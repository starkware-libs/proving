use inst_def::InstDef;
use prover_types::cpu::{FELT252_BITS_PER_WORD, FELT252_N_WORDS};

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;
use crate::core::variables::*;
use crate::{const_expr, const_u16_expr};

#[derive(Debug, InstDef)]
pub struct ReadSplit {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

// Read the felt252 at the given address and return it splitted into two parts: the least
// significant 248 bits (low) and the most significant 4 bits (high). Also returns the
// original (non-splitted) value.
impl AirFn for ReadSplit {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = [Felt252Expr; 3]; // [low, high, original (high << 248 + low)]

    fn call(&self, air_builder: &mut AirBuilder, _: (), address: Self::In) -> Self::Out {
        const LOW_BITS_IN_MS_LIMB: u16 =
            (248 - (FELT252_N_WORDS - 1) * FELT252_BITS_PER_WORD) as u16;
        let (mut value, _id) = self.memory.read_unverified(air_builder, &address);

        // Deduce the low limbs as-is
        for i in 0..(FELT252_N_WORDS - 1) {
            air_builder.deduce(value.get_felt_mut(i), &format!("limb_{}", i));
        }

        // Deduce the most significant limb split into two parts
        let ms_limb = UInt16Expr::from(value.get_felt(FELT252_N_WORDS - 1));
        let mut ms_limb_low = air_builder.let_for_deduction(
            ms_limb.clone() & const_u16_expr!((1 << LOW_BITS_IN_MS_LIMB) - 1),
            "ms_limb_low",
        );
        let mut ms_limb_high = air_builder.let_for_deduction(
            ms_limb >> const_u16_expr!(LOW_BITS_IN_MS_LIMB),
            "ms_limb_high",
        );
        let ms_limb_low_felt = air_builder.deduce(ms_limb_low.as_felt_mut(), "ms_limb_low");
        let ms_limb_high_felt = air_builder.deduce(ms_limb_high.as_felt_mut(), "ms_limb_high");

        // Range check the parts
        range_check(
            air_builder,
            &[
                LOW_BITS_IN_MS_LIMB,
                TryInto::<u16>::try_into(FELT252_BITS_PER_WORD).unwrap() - LOW_BITS_IN_MS_LIMB,
            ],
            &[ms_limb_low_felt.clone(), ms_limb_high_felt.clone()],
        );

        // Build the original value from the deduced parts and verify that it is indeed the value
        // in the memory.
        let mut memory_value_felts: Vec<_> = value.as_felts()[0..FELT252_N_WORDS - 1].into();
        memory_value_felts.push(
            ms_limb_high_felt.clone() * const_expr!(1 << LOW_BITS_IN_MS_LIMB)
                + ms_limb_low_felt.clone(),
        );

        air_builder.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
            (address, memory_value_felts.clone().into()),
        );

        // Build the low and high parts from the deduced felts.
        let mut low_felts: Vec<_> = value.as_felts()[0..FELT252_N_WORDS - 1].into();
        low_felts.push(ms_limb_low_felt);

        let high_felts = vec![ms_limb_high_felt];

        [
            low_felts.into(),
            high_felts.into(),
            memory_value_felts.into(),
        ]
    }
}
