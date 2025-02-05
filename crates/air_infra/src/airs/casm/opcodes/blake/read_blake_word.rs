use inst_def::InstDef;

use super::verify_blake_word::*;
use crate::airs::casm::casm_state::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::felt252_id_memory::memory::*;

#[derive(Debug, InstDef, Default)]
pub struct ReadBlakeWord {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

/// Reads a Blake word from the memory at the given `CasmAddress` and
/// converts it from the memory representation to the expected Blake format,
/// i.e., from 32 bits stored in felts of sizes [9, 9, 9, 5] to a pair of 16-bit felts.
impl AirFn for ReadBlakeWord {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), addr: Self::In) -> Self::Out {
        // Read 'felt252' from addr containing the 32-bits blake word.
        let (word, _) = self.memory.read_unverified(air_builder, &addr);

        // Split the limb1 into high part of 2 bits and low part of 7 bits.
        let limb1_2_ms_bits = air_builder
            .let_for_deduction(UInt16Expr::from(word.get_felt(1)) >> const_u16_expr!(7), "");
        let limb1_7_ls_bits = word.get_felt(1) - limb1_2_ms_bits.as_felt() * const_expr!(1 << 7);

        // Build and deduce the low part of the Blake word from 9 bits of limb0 and the 7 ls bits of
        // limb1.
        let mut low_16_bits = limb1_7_ls_bits * const_expr!(1 << 9) + word.get_felt(0);
        air_builder.deduce(&mut low_16_bits, "low_16_bits");

        // Build and deduce the high part of the Blake word from 2 ms bits of limb1, 9 bits of limb2
        // and 5 bits of limb3.
        let mut high_16_bits = word.get_felt(3) * const_expr!(1 << 11)
            + word.get_felt(2) * const_expr!(1 << 2)
            + limb1_2_ms_bits.as_felt();
        air_builder.deduce(&mut high_16_bits, "high_16_bits");

        let expected_blake_word: UInt32Expr =
            air_builder.let_vec(vec![low_16_bits, high_16_bits], "expected_word");

        air_builder.call(
            &VerifyBlakeWord {
                memory: self.memory.clone(),
            },
            (addr, expected_blake_word.clone()),
        );
        expected_blake_word
    }
}
