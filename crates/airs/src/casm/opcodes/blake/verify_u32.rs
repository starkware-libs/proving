use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::expressions::uint16_expr::UInt16Expr;
use air_infra::core::expressions::uint32_expr::UInt32Expr;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::felt252_id_memory::verify::MemVerify;
use air_infra::range_check::range_check;
use air_infra::{const_expr, const_u16_expr};
use serde::Serialize;

#[derive(Debug, Serialize, Default)]
pub struct VerifyU32 {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Receives an address and a UInt32Expr, range checks it and converts the word to the memory
/// representation i.e., from a pair of 16-bit felts, through chunks of [9, 7, 2, 9, 5] bits, to a
/// `felt252` containing 32 bits stored in chunks of sizes [9, 9, 9, 5], and verifies that this
/// value is stored at the given address.
impl AirFn for VerifyU32 {
    type ExtIn = ();
    type In = (CasmAddress, UInt32Expr);
    type Out = ();

    fn call(&self, air_builder: &mut AirBuilder, _: (), (addr, word): Self::In) -> Self::Out {
        // Split felt low into low chunk of 9 bits and high chunk of 7 bits.
        let low_7_ms_bits =
            air_builder.deduce_air_var(word.low() >> const_u16_expr!(9), "low_7_ms_bits");
        let low_9_ls_bits = word.low().as_felt() - low_7_ms_bits.as_felt() * const_expr!(1 << 9);

        // Split felt high into chunks of (2, 9, 5) from low to high.
        let high_14_ms_bits =
            air_builder.deduce_air_var(word.high() >> const_u16_expr!(2), "high_14_ms_bits");
        let high_2_ls_bits = air_builder.let_(
            word.high().as_felt() - high_14_ms_bits.as_felt() * const_expr!(1 << 2),
            "high_2_ls_bits",
        );
        let high_5_ms_bits = air_builder
            .deduce_air_var(high_14_ms_bits.clone() >> const_u16_expr!(9), "high_5_ms_bits");
        let high_9_mid_bits =
            high_14_ms_bits.as_felt() - high_5_ms_bits.as_felt() * const_expr!(1 << 9);

        // Range check the split chunks.
        range_check(
            air_builder,
            &[7, 2, 5],
            &[low_7_ms_bits.as_felt(), high_2_ls_bits.clone(), high_5_ms_bits.as_felt()],
        );

        // Verify that the expected value is stored at the given memory address.
        air_builder.call(
            &MemVerify { memory: self.memory.clone() },
            (
                addr,
                Felt252Expr::from(vec![
                    low_9_ls_bits,
                    low_7_ms_bits.as_felt() + high_2_ls_bits * const_expr!(1 << 7),
                    high_9_mid_bits,
                    high_5_ms_bits.as_felt(),
                ]),
            ),
        );
    }
}
