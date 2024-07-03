use std::collections::BTreeMap;

use super::super::range_check::*;
use super::common::*;

use crate::core::air_fn::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint16_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

//Macros
use crate::const_expr;
use crate::const_u16_expr;
use crate::const_u32_expr;

// An AirFn of type CheckInstruction.
// Holds its three bools indicating which of the 3 offsets in the instruction are const and memory.
#[derive(Clone, Debug)]
pub struct CheckInstruction {
    pub off_is_const: [bool; 3], // off_dst, off_0, off_1
    memory: Memory<FeltExpr, Felt252Expr>,
}

// Receives pc, a vector of constant offsets and the 15 flags.
// Breaks the instruction written at pc into 12-bit components, deduces and range checks the 2
// parts of each non-constant offset, sets the concatenation of the components in memory and
// returns the felts of the non-constant offsets pieced back together.
impl AirFn for CheckInstruction {
    type In = (FeltExpr, Offsets, Flags);
    type Out = [FeltExpr; 3];

    fn call(&self, ab: &mut AirBuilder, (pc, offsets, flags): Self::In) -> Self::Out {
        let instruction_for_deduction = ab.get_from_memory(&self.memory, &pc);
        let mut offsets_parts = vec![];

        for (ind, &curr_off_is_const) in self.off_is_const.iter().enumerate() {
            let mut curr_offset = None;
            if curr_off_is_const {
                curr_offset = Some(offsets[ind].clone());
            }
            offsets_parts.push(check_offset(
                ind,
                ab,
                curr_offset,
                instruction_for_deduction.clone(),
            ));
        }
        let [off_dst, off_0, off_1] = offsets_parts
            .try_into()
            .expect("offsets_parts should have 3 offsets.");

        // Compute the 12 bit components.
        let felt0 = off_dst.low;
        let felt1 = off_dst.high + (off_0.low * const_expr!(1 << 4));
        let felt2 = off_0.high + (off_1.low * const_expr!(1 << 8));
        let felt3 = off_1.high;

        let felt4 = ((((((((((flags[0].as_felt()
            + (flags[1].as_felt() * const_expr!(1 << 1)))
            + (flags[2].as_felt() * const_expr!(1 << 2)))
            + (flags[3].as_felt() * const_expr!(1 << 3)))
            + (flags[4].as_felt() * const_expr!(1 << 4)))
            + (flags[5].as_felt() * const_expr!(1 << 5)))
            + (flags[6].as_felt() * const_expr!(1 << 6)))
            + (flags[7].as_felt() * const_expr!(1 << 7)))
            + (flags[8].as_felt() * const_expr!(1 << 8)))
            + (flags[9].as_felt() * const_expr!(1 << 9)))
            + (flags[10].as_felt() * const_expr!(1 << 10)))
            + (flags[11].as_felt() * const_expr!(1 << 11));

        let felt5 = (flags[12].as_felt() + (flags[13].as_felt() * const_expr!(1 << 1)))
            + (flags[14].as_felt() * const_expr!(1 << 2));

        ab.set_in_memory(
            &self.memory,
            pc.clone(),
            Felt252Expr::from(vec![felt0, felt1, felt2, felt3, felt4, felt5]),
        );

        [off_dst.val, off_0.val, off_1.val]
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [(
            "off_is_const".to_string(),
            format!(
                "[{}, {}, {}]",
                self.off_is_const[0], self.off_is_const[1], self.off_is_const[2]
            ),
        )]
        .into()
    }
}

impl MemoryAirFn for CheckInstruction {
    type K = FeltExpr;
    type V = Felt252Expr;

    fn init_memory(&mut self, memory: &Memory<FeltExpr, Felt252Expr>) {
        self.memory = memory.clone();
    }
}

// Receives the index of the offset, a optional u16 of a constant offset (None if the offset isn't
//  constant) and the original instruction.
// Breaks the offset into 2 felt parts according to it's position in the instruction and
// returns them. If the offset isn't constant it deduces both parts, range checks those that
// aren't 12 bits and returns the concatenation of the parts (otherwise returns it as None).
fn check_offset(
    offset_index: usize,
    ab: &mut AirBuilder,
    offset: Option<UInt16Expr>,
    mut instruction_for_deduction: Felt252Expr,
) -> OffsetParts {
    let off_begin = (offset_index * 16) % FELT252_BITS_PER_WORD;
    let off_l_len = FELT252_BITS_PER_WORD - off_begin;

    let (low, high) = if let Some(off) = offset {
        // Split the offset into high and low parts.
        let high_u16 = off.clone() >> const_u16_expr!(off_l_len as u16);
        let low_u16 = off & const_u16_expr!((1 << off_l_len as u16) - 1);
        (low_u16.const_to_felt(), high_u16.const_to_felt())
    } else {
        // Find the low part of the offset.
        let low = check_offset_part(
            off_begin,
            off_l_len,
            ab,
            instruction_for_deduction.as_felts_mut()[offset_index],
        );

        // Find the high part of the offset.
        let high = check_offset_part(
            0,
            16 - off_l_len,
            ab,
            instruction_for_deduction.as_felts_mut()[offset_index + 1],
        );

        (low, high)
    };

    // Reconstruct the offset as felt from the high and low parts.
    let val = low.clone() + (high.clone() * const_expr!(1 << off_l_len));
    OffsetParts { low, high, val }
}

// Recieves begining bit and the bit length of an offset part, a felt to split and returns the part
// of the offset according to the specified begining and length.
// If not 12 bits long then it also range checks the part.
fn check_offset_part(
    begin: usize,
    len: usize,
    ab: &mut AirBuilder,
    felt_to_split: &mut FeltExpr,
) -> FeltExpr {
    if len == FELT252_BITS_PER_WORD {
        ab.deduce(felt_to_split)
    } else {
        let inst_f_curr: UInt32Expr = felt_to_split.clone().into();
        let mut off =
            (inst_f_curr >> const_u32_expr!(begin as u32)) & const_u32_expr!((1 << len) - 1);

        off = ab.let_for_deduction(off);
        let off_f = ab.deduce(off.low_mut().as_felt_mut());
        ab.lookup_call(&RangeCheck { bits: len as u16 }, off_f.clone());
        off_f
    }
}

#[derive(Default, Debug)]
struct OffsetParts {
    low: FeltExpr,
    high: FeltExpr,
    val: FeltExpr,
}
