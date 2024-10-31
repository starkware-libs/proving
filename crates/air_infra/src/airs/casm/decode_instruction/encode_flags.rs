use inst_def::InstDef;

use prover_types::cpu::FELT252_BITS_PER_WORD;

use crate::airs::casm::common::*;
use crate::airs::felt252_id_memory::memory::*;
use crate::core::air_fn::*;
use crate::core::expressions::felt_expr::*;

// Macros
use crate::const_expr;

#[derive(Clone, Debug, InstDef)]
pub struct EncodeFlags {
    #[instdef(skip)]
    pub memory: Felt252IdMemory,
}

// | 9 |   9   | - |   -   | - |   -   | - | Felts in the instruction
// | 9 | 6 | - | - | - | - | - | - | - | - | Parts of flags
// |  15   |    -      |    -      |  -    | Flags
//
// Constrains that each flag is either 0 or 1.
// Constructs the two felts holding the flags in the instruction.
impl AirFn for EncodeFlags {
    type In = [FeltExpr; 15];
    type Out = [FeltExpr; 2];

    fn call(&self, ab: &mut AirBuilder, flags: Self::In) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 9,
            "FlagsToFelts assumes there are 9 bits per felt in a felt252"
        );

        for (i, flag) in flags.iter().enumerate() {
            ab.constrain(
                flag.clone() * (const_expr!(1) - flag.clone()),
                &format!("Flag {} is a bit", FLAG_NAMES[i]),
            );
        }

        let mut felt5 = const_expr!(0);
        for (i, flag) in flags.iter().enumerate().take(6) {
            felt5 = felt5.clone() + (flag.clone() * const_expr!(1 << (i + 3)));
        }

        let mut felt6 = const_expr!(0);
        for (i, flag) in flags.into_iter().enumerate().skip(6) {
            felt6 = felt6.clone() + (flag * const_expr!(1 << (i - 6)));
        }

        [felt5, felt6]
    }
}
