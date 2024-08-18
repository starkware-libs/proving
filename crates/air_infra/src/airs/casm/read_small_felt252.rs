use indexmap::IndexMap;

use super::common::*;
use super::const_tables::range_check::*;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

/// Reads from memory a felt252, writes the lower <num_limbs> limbs to the trace
/// and constrains the rest to be zeros (so the felt252 has value < 2**(12*num_limbs)).
/// Returns the felt252.
#[derive(Debug)]
pub struct ReadSmallFelt252 {
    pub num_bits: usize,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for ReadSmallFelt252 {
    type In = CasmAddress;
    type Out = Felt252Expr;

    fn call(&self, air_builder: &mut AirBuilder, address: Self::In) -> Self::Out {
        let mut value_from_memory = air_builder.mem_read(&self.memory, &address);
        let mut expected_nonzero_limbs = vec![];
        let remainder = self.num_bits % FELT252_BITS_PER_WORD;
        let num_limbs = self.num_bits.div_ceil(FELT252_BITS_PER_WORD);

        for felt in value_from_memory.as_felts_mut().into_iter().take(num_limbs) {
            expected_nonzero_limbs.push(air_builder.deduce(felt));
        }

        let expected_value = Felt252Expr::from(expected_nonzero_limbs.clone());
        air_builder.mem_verify(&self.memory, address, expected_value.clone());

        //  Range check the last limb.
        if remainder != 0 {
            air_builder.lookup_call(
                &RangeCheck {
                    bits: [remainder as u16],
                },
                [expected_nonzero_limbs[num_limbs - 1].clone()],
            );
        };
        expected_value
    }

    fn inst_def(&self) -> IndexMap<String, String> {
        [("num_bits".to_string(), self.num_bits.to_string())].into()
    }
}
