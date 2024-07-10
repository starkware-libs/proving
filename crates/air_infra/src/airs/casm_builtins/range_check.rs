use crate::airs::casm::read_small_felt252::*;
use crate::airs::range_check::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

#[derive(Debug)]
pub struct RangeCheckBuiltin {
    pub bits: usize,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

impl AirFn for RangeCheckBuiltin {
    type In = FeltExpr;

    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, address: Self::In) -> Self::Out {
        let nonzero_limbs = self.bits.div_ceil(FELT252_BITS_PER_WORD);
        let bits_in_msb_limb = self.bits % FELT252_BITS_PER_WORD;
        let read_fn = ReadSmallFelt252 {
            num_limbs: nonzero_limbs,
            memory: self.memory.clone(),
        };

        let value_from_memory = air_builder.call(&read_fn, address);

        if bits_in_msb_limb != 0 {
            let msb_limb = value_from_memory
                .as_felts()
                .into_iter()
                .nth(nonzero_limbs - 1)
                .expect("The Felt252 read from memory should have enough limbs");

            air_builder.lookup_call(
                &RangeCheck {
                    bits: bits_in_msb_limb as u16,
                },
                msb_limb,
            );
        }
    }

    fn inst_def(&self) -> std::collections::BTreeMap<String, String> {
        [("bits".to_string(), self.bits.to_string())].into()
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Component
    }
}
