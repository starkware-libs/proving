use std::collections::BTreeMap;

use super::super::range_check::*;
use super::common::*;
use crate::core::air_fn::*;
use crate::core::expressions::bool_expr::*;
use crate::core::expressions::felt252_expr::*;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::uint32_expr::*;
use crate::core::memory::*;
use crate::core::prover_types::*;
use crate::core::variables::*;

// Macros
use crate::const_bool_expr;
use crate::const_expr;
use crate::const_u32_expr;

// An AirFn of type CheckInstruction.
// Holds the constant offsets, constant flags and memory.
#[derive(Clone, Debug)]
pub struct CheckInstruction {
    pub const_offsets: [Option<u16>; 3], // off_0, off_1, off_2
    pub const_flags: Flags,
    pub memory: Memory<FeltExpr, Felt252Expr>,
}

// Breaks the instruction written at pc into 12-bit components, deduces and range checks the 2
// parts of each non-constant offset, sets the concatenation of the components in memory and
// returns the felts of the non-constant offsets pieced back together.
impl AirFn for CheckInstruction {
    type In = FeltExpr;
    type Out = ([FeltExpr; 3], [BoolExpr; 15]);

    fn call(&self, ab: &mut AirBuilder, pc: Self::In) -> Self::Out {
        assert_eq!(
            FELT252_BITS_PER_WORD, 12,
            "CheckInstruction assumes there are 12 bits per felt in a felt252"
        );

        let instruction_for_deduction = ab.get_from_memory(&self.memory, &pc);
        let mut offsets_parts = vec![];

        for (i, off) in self.const_offsets.iter().enumerate() {
            offsets_parts.push(check_offset(i, ab, off, instruction_for_deduction.clone()));
        }
        let [off_0, off_1, off_2] = offsets_parts
            .try_into()
            .expect("offsets_parts should have 3 offsets.");

        // Compute the 12 bit components.
        let felt0 = off_0.low;
        let felt1 = off_0.high + (off_1.low * const_expr!(1 << 4));
        let felt2 = off_1.high + (off_2.low * const_expr!(1 << 8));
        let felt3 = off_2.high;

        let mut felt4 = self.const_flags.sum(0, 12);
        let mut felt5 = self.const_flags.sum(12, 15);
        let felts = instruction_for_deduction.as_felts();
        let flags: [BoolExpr; 15] = self
            .const_flags
            .to_arr()
            .into_iter()
            .enumerate()
            .map(|(i, flag)| {
                if let Some(flag) = flag {
                    return const_bool_expr!(flag);
                }

                // Get the flag from the instruction.
                let (felt_index, felt_to_update, shift) = if i < 12 {
                    (4, &mut felt4, i)
                } else {
                    (5, &mut felt5, i - 12)
                };
                let flag = check_flag(ab, shift, felts[felt_index].clone());

                // Update the corresponding felt.
                *felt_to_update = felt_to_update.clone() + (flag.clone() * const_expr!(1 << shift));

                flag.into()
            })
            .collect::<Vec<_>>()
            .try_into()
            .expect("flags should have 15 elements.");

        ab.set_in_memory(
            &self.memory,
            pc.clone(),
            Felt252Expr::from(vec![felt0, felt1, felt2, felt3, felt4, felt5]),
        );

        ([off_0.val, off_1.val, off_2.val], flags)
    }

    fn inst_def(&self) -> BTreeMap<String, String> {
        [
            (
                "const_offsets".to_string(),
                format!("{:?}", self.const_offsets),
            ),
            ("const_flags".to_string(), format!("{:?}", self.const_flags)),
        ]
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

// Receives the felt where this flag is stored and the index in this felt.
// Deduces the flag and adds a constraint that the flag is either 0 or 1.
fn check_flag(ab: &mut AirBuilder, index: usize, felt: FeltExpr) -> FeltExpr {
    let mut flag = if index == 0 {
        UInt32Expr::from(felt) & const_u32_expr!(1)
    } else {
        (UInt32Expr::from(felt) >> const_u32_expr!(index as u32)) & const_u32_expr!(1)
    };

    flag = ab.let_for_deduction(flag);
    let flag_f = ab.deduce(flag.low_mut().as_felt_mut());
    ab.constrain(flag_f.clone() * (const_expr!(1) - flag_f.clone()));

    flag_f
}

// Receives the index of the offset, a optional u16 of a constant offset (None if the offset isn't
// constant) and the original instruction.
// Breaks the offset into 2 felt parts according to it's position in the instruction and
// returns them. If the offset isn't constant it deduces both parts, range checks those that
// aren't 12 bits and returns the concatenation of the parts (otherwise returns it as None).
fn check_offset(
    offset_index: usize,
    ab: &mut AirBuilder,
    offset: &Option<u16>,
    mut instruction_for_deduction: Felt252Expr,
) -> OffsetParts {
    let off_begin = (offset_index * 16) % 12;
    let off_l_len = 12 - off_begin;

    if let Some(off) = offset {
        // Split the offset into high and low parts.
        let high_u16 = off >> (off_l_len as u16);
        let low_u16 = off & ((1 << off_l_len as u16) - 1);
        return OffsetParts {
            low: const_expr!(low_u16 as u32),
            high: const_expr!(high_u16 as u32),
            val: const_expr!(*off as u32),
        };
    }

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
    if len == 12 {
        ab.deduce(felt_to_split)
    } else {
        let inst_f_curr: UInt32Expr = felt_to_split.clone().into();
        let mut off = if begin == 0 {
            inst_f_curr & const_u32_expr!((1 << len) - 1)
        } else {
            (inst_f_curr >> const_u32_expr!(begin as u32)) & const_u32_expr!((1 << len) - 1)
        };

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
