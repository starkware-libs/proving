use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;

use super::verify_u32::*;

#[derive(Debug, Serialize, Default)]
pub struct ReadU32 {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Reads a UInt32Expr from the memory at the given `CasmAddress` and
/// converts it from the memory representation to the expected format,
/// i.e., from 32 bits stored in felts of sizes [9, 9, 9, 5] to a pair of 16-bit felts.
impl AirFn for ReadU32 {
    type ExtIn = ();
    type In = CasmAddress;
    type Out = UInt32Expr;

    fn call(&self, air_builder: &mut AirBuilder, _: (), addr: Self::In) -> Self::Out {
        // Read 'felt252' from addr containing the 32-bit value.
        let (word, _) = self.memory.read_unverified(air_builder, &addr);

        // Split the limb1 into high part of 2 bits and low part of 7 bits.
        let limb1_2_ms_bits = air_builder
            .let_for_deduction(UInt16Expr::from(word.get_felt(1)) >> const_u16_expr!(7), "");
        let limb1_7_ls_bits = word.get_felt(1) - limb1_2_ms_bits.as_felt() * const_expr!(1 << 7);

        // Build and deduce the 16-bits low part of the u32 from 9 bits of limb0 and the 7 ls bits
        // of limb1.
        let mut low_16_bits = limb1_7_ls_bits * const_expr!(1 << 9) + word.get_felt(0);
        air_builder.deduce(&mut low_16_bits, "low_16_bits");

        // Build and deduce the 16-bits high part of the u32 from 2 ms bits of limb1, 9 bits of
        // limb2 and 5 bits of limb3.
        let mut high_16_bits = word.get_felt(3) * const_expr!(1 << 11)
            + word.get_felt(2) * const_expr!(1 << 2)
            + limb1_2_ms_bits.as_felt();
        air_builder.deduce(&mut high_16_bits, "high_16_bits");

        let expected_word: UInt32Expr =
            air_builder.let_(vec![low_16_bits, high_16_bits].into(), "expected_word");

        air_builder.call(&VerifyU32 { memory: self.memory.clone() }, (addr, expected_word.clone()));
        expected_word
    }
}
