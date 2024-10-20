use inst_def::InstDef;

use compiled_casm_air::prover_types::FELT252_BITS_PER_WORD;

use crate::airs::casm::common::*;
use crate::airs::casm::const_tables::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::variables::*;

use super::memory::*;

#[derive(Debug, InstDef)]
pub struct ReadPositive {
    pub num_bits: usize,
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Read a Felt252 in the range [0,2**num_bits - 1] from the memory
/// Returns also the ID of the value in the memory.
impl AirFn for ReadPositive {
    type In = CasmAddress;
    type Out = (Felt252Expr, FeltExpr);

    fn call(&self, air_builder: &mut AirBuilder, address: Self::In) -> Self::Out {
        // Read the id and deduce it as-is
        let mut id = air_builder.mem_read_unverified(&self.memory.address_to_id, &address);
        air_builder.deduce(&mut id, "id");
        air_builder.mem_verify(&self.memory.address_to_id, &address, id.clone());

        // Prepare for value deduction
        let mut value = air_builder.mem_read_unverified(&self.memory.id_to_value, &id);
        let num_nonzero_limbs = self.num_bits.div_ceil(FELT252_BITS_PER_WORD);
        let bits_in_ms_limb = self.num_bits % FELT252_BITS_PER_WORD;

        // Deduce the nonzero limbs
        for (i, limb) in value
            .as_felts_mut()
            .into_iter()
            .take(num_nonzero_limbs)
            .enumerate()
        {
            air_builder.deduce(limb, &format!("limb_{}", i));
        }

        // If required - range-check the most significant limb
        if bits_in_ms_limb != 0 {
            air_builder.lookup_call(
                &RangeCheck {
                    bits: [bits_in_ms_limb as u16],
                },
                [value.get_felt(num_nonzero_limbs - 1)],
            );
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
            &self.memory.id_to_value,
            &id,
            expected_value_in_memory.clone(),
        );

        (expected_value_in_memory, id)
    }
}
