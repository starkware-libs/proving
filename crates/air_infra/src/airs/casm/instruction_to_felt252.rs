use super::super::range_check::*;
use super::common::*;
use std::collections::BTreeMap;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;

//Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::core::prover_types::*;

// An AirFn of type InstructionToFelt252.
// Holds its three bools indicating which of the 3 offsets in the instruction are const.
#[derive(Clone, Debug)]
pub struct InstructionToFelt252 {
    pub off0_is_const: bool,
    pub off1_is_const: bool,
    pub off2_is_const: bool,
}

impl AirFn for InstructionToFelt252 {
    // Receives 3 offsets and the 15 flags, breaks their concatination into 12 bit components,
    // deduces and range checks the 2 parts of each non const offsets and returns the felt252 of
    // the concatination and the felts of offsets pieced back together.
    type In = ([UInt16Expr; 3], Flags);
    type Out = (Felt252Expr, [FeltExpr; 3]);

    fn call(&self, ab: &mut AirBuilder, ([off0, off1, off2], mut flags): Self::In) -> Self::Out {
        // Split off0 into high and low parts.
        let mut off0h = &off0 >> &const_u16_expr!(12);
        let mut off0l = &off0 - &(&off0h << &const_u16_expr!(12));
        if !self.off0_is_const {
            // Deduce and range check both parts if not const.
            off0h = ab.let_for_deduction(off0h);
            let off0h_f = ab.deduce(off0h.as_felt());
            ab.lookup_call(&RangeCheck { bits: 4 }, off0h_f);

            off0l = ab.let_for_deduction(off0l);
            let off0l_f = ab.deduce(off0l.as_felt());
            ab.lookup_call(&RangeCheck { bits: 12 }, off0l_f);
        }
        // Reconstruct the offset as felt from the high and low parts.
        let res_off0 = &*off0l.as_felt() + &(&*off0h.as_felt() * &const_expr!(1 << 12));

        let mut off1h = &off1 >> &const_u16_expr!(8);
        let mut off1l = &off1 - &(&off1h << &const_u16_expr!(8));
        if !self.off1_is_const {
            off1h = ab.let_for_deduction(off1h);
            let off1h_f = ab.deduce(off1h.as_felt());
            ab.lookup_call(&RangeCheck { bits: 8 }, off1h_f);

            off1l = ab.let_for_deduction(off1l);
            let off1l_f = ab.deduce(off1l.as_felt());
            ab.lookup_call(&RangeCheck { bits: 8 }, off1l_f);
        }
        let res_off1 = &*off1l.as_felt() + &(&*off1h.as_felt() * &const_expr!(1 << 8));

        let mut off2h = &off2 >> &const_u16_expr!(4);
        let mut off2l = &off2 - &(&off2h << &const_u16_expr!(4));
        if !self.off2_is_const {
            off2h = ab.let_for_deduction(off2h);
            let off2h_f = ab.deduce(off2h.as_felt());
            ab.lookup_call(&RangeCheck { bits: 12 }, off2h_f);

            off2l = ab.let_for_deduction(off2l);
            let off2l_f = ab.deduce(off2l.as_felt());
            ab.lookup_call(&RangeCheck { bits: 4 }, off2l_f);
        }
        let res_off2 = &*off2l.as_felt() + &(&*off2h.as_felt() * &const_expr!(1 << 4));

        // Compute the 12 bit components.
        let felt0 = off0l.as_felt().clone();
        let felt1 = &*off0h.as_felt() + &(&*off1l.as_felt() * &const_expr!(1 << 4));
        let felt2 = &*off1h.as_felt() + &(&*off2l.as_felt() * &const_expr!(1 << 8));
        let felt3 = off2h.as_felt().clone();

        let felt4 = &(&(&(&(&(&(&(&(&(&(&flags[0].as_felt().clone()
            + &(&*flags[1].as_felt() * &const_expr!(1 << 1)))
            + &(&*flags[2].as_felt() * &const_expr!(1 << 2)))
            + &(&*flags[3].as_felt() * &const_expr!(1 << 3)))
            + &(&*flags[4].as_felt() * &const_expr!(1 << 4)))
            + &(&*flags[5].as_felt() * &const_expr!(1 << 5)))
            + &(&*flags[6].as_felt() * &const_expr!(1 << 6)))
            + &(&*flags[7].as_felt() * &const_expr!(1 << 7)))
            + &(&*flags[8].as_felt() * &const_expr!(1 << 8)))
            + &(&*flags[9].as_felt() * &const_expr!(1 << 9)))
            + &(&*flags[10].as_felt() * &const_expr!(1 << 10)))
            + &(&*flags[11].as_felt() * &const_expr!(1 << 11));

        let felt5 = &(&flags[12].as_felt().clone()
            + &(&*flags[13].as_felt() * &const_expr!(1 << 1)))
            + &(&*flags[14].as_felt() * &const_expr!(1 << 2));

        (
            Felt252Expr::from(vec![felt0, felt1, felt2, felt3, felt4, felt5]),
            [res_off0, res_off1, res_off2],
        )
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            ("off0_is_const".to_string(), self.off0_is_const.to_string()),
            ("off1_is_const".to_string(), self.off1_is_const.to_string()),
            ("off2_is_const".to_string(), self.off2_is_const.to_string()),
        ]
        .into()
    }
}
