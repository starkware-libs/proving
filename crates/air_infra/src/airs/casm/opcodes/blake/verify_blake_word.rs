use serde::Serialize;

use crate::airs::casm::casm_state::*;
use crate::airs::casm::const_tables::range_check::*;
// Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::felt252_id_memory::memory::*;
use crate::core::felt252_id_memory::verify::*;

#[derive(Debug, Serialize, Default)]
pub struct VerifyBlakeWord {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

/// Receives an address and a Blake word, range checks it and converts the word to the memory
/// representation i.e., from a pair of 16-bit felts, through chunks of [9, 7, 2, 9, 5] bits, to a
/// `felt252` containing 32 bits stored in chunks of sizes [9, 9, 9, 5], and verifies that this
/// value is stored at the given address.
impl AirFn for VerifyBlakeWord {
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
        let high_2_ls_bits =
            word.high().as_felt() - high_14_ms_bits.as_felt() * const_expr!(1 << 2);
        let high_5_ms_bits = air_builder.deduce_air_var(
            high_14_ms_bits.clone() >> const_u16_expr!(9),
            "high_5_ms_bits",
        );
        let high_9_mid_bits =
            high_14_ms_bits.as_felt() - high_5_ms_bits.as_felt() * const_expr!(1 << 9);

        // Range check the splited chunks.
        range_check(
            air_builder,
            &[7, 2, 5],
            &[
                low_7_ms_bits.as_felt(),
                high_2_ls_bits.clone(),
                high_5_ms_bits.as_felt(),
            ],
        );

        // Verify that the expected value is stored at the given memory address.
        air_builder.call(
            &MemVerify {
                memory: self.memory.clone(),
            },
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
