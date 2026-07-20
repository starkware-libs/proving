use std::array::from_fn;

use air_common::TraceType;
use air_infra::casm_state::CasmAddress;
use air_infra::core::air_fn::{AirBuilder, AirFn};
use air_infra::core::expressions::biguint_expr::{BigUInt384Expr, BigUIntExpr};
use air_infra::core::expressions::felt_expr::FeltExpr;
use air_infra::core::expressions::felt252_expr::Felt252Expr;
use air_infra::core::public_params::PublicParam;
use air_infra::felt252_id_memory::memory::Felt252IdMemory;
use air_infra::seq::Seq;
use air_infra::{const_bigu384_expr, const_expr};
use serde::Serialize;
use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use super::mod_utils::*;

// Number of subwords in a bundle. A bundle is a segment of consecutive subwords that can be grouped
// into one M31 felt.
pub const SUBWORD_BUNDLE_SIZE: usize = 3;
// Total number of subword bundles in a mod_builtin number. Currently 15.
pub const TOTAL_SUBWORD_BUNDLES: usize = TOTAL_SUBWORDS.div_ceil(SUBWORD_BUNDLE_SIZE);
// Number of words composing a mod_builtin number after padding to be a multiple of
// SUBWORD_BUNDLE_SIZE. Currently 45.
pub const TOTAL_SUBWORDS_PADDED: usize = SUBWORD_BUNDLE_SIZE * TOTAL_SUBWORD_BUNDLES;

// 9-bit or 6-bit limbs (subwords) after padding
//        limb ->   44  43  42  41  40  39  38  37  36  35  34  33  32  31 ...
//   limb bits ->    0   6   9   9   9   9   9   9   9   9   9   9   6   9 ...
//  bundle_idx ->   14  14  14  13  13  13  12  12  12  11  11  11  10  10 ...
// bundle_bits ->       15          27          27          27          24 ...
//        limb ->   ... 11  10   9   8   7   6   5   4   3   2   1   0
//   limb bits ->   ...  9   6   9   9   9   9   9   9   9   9   9   9
//  bundle_idx ->   ...  3   3   3   2   2   2   1   1   1   0   0   0
// bundle_bits ->    ...     24          27          27          27

#[derive(Debug, Serialize, Default)]
pub struct AddModBuiltin {
    #[serde(skip)]
    pub memory: Felt252IdMemory,
}

// The air function of the add_mod builtin.
//
// The builtin will enforce that continuous memory segment for add_mod contains rows of the form
// (p[0:MOD_BUILTIN_N_WORDS], values_ptr, offsets_ptr, n)
// Where if n != 1, the next line is enforced to be
// (p0,...,p{MOD_BUILTIN_N_WORDS-1}, values_ptr, offsets_ptr + 3, n - 1)
// For n=1, the words of p, and the values of values_ptr, offsets_ptr and n may be refreshed.
//
// Additional memory cells and virtual columns will be used to enforce that in each row:
// mem[values_ptr+mem[offsets_ptr[0]]+mem[values_ptr+mem[offsets_ptr+1]] =
// mem[values_ptr+mem[offsets_ptr+2]] + sub_p_bit*p
// where p and the values pointed by values_ptr are large numbers represented by
// `MOD_BUILTIN_N_WORDS` consecutive memory cells,
// each of `MOD_BUILTIN_WORD_BIT_LEN` bits and sub_p_bit is 0 or 1.
//
// Assumption:
// The words of p and of all values in values_ptr are externally enforced to be in the range
// [0, 2**MOD_BUILTIN_WORD_BIT_LEN), i.e. are represented each by 10 9 bit m31 felts and another 6
// bit felt.

impl AirFn for AddModBuiltin {
    type ExtIn = ();
    type In = ();
    type Out = ();

    fn call(&self, ab: &mut AirBuilder, _: (), _: ()) -> Self::Out {
        let instance_num = ab.call_external_table(&Seq {});
        let segment_start = ab.get_public_param(PublicParam::AddModBuiltinSegmentStart);
        // Get p, a, b, c from the memory segment of add_mod
        let [p, a, b, c] = ab.call(
            &ModUtils { memory: self.memory.clone() },
            (CasmAddress::new(segment_start, "add_mod_segment_start"), instance_num),
        );

        // Compute and deduce sub_p_bit
        let a_384: BigUInt384Expr = a.to_vec().into();
        let b_384: BigUInt384Expr = b.to_vec().into();
        let c_384: BigUInt384Expr = c.to_vec().into();
        let diff_384 = ab.let_for_deduction(a_384 + b_384 - c_384, "diff");
        let is_diff_0 =
            ab.let_for_deduction(diff_384.eq(const_bigu384_expr!(0, 0, 0, 0, 0, 0)), "is_diff_0");
        let sub_p_bit = ab.deduce(&mut (const_expr!(1) - is_diff_0.as_felt()), "sub_p_bit");
        ab.constrain(
            (sub_p_bit.clone() - const_expr!(1)) * sub_p_bit.clone(),
            "make sure sub_p_bit is 0 or 1.",
        );

        let p_felts = to_felt_array_padded(p);
        let a_felts = to_felt_array_padded(a);
        let b_felts = to_felt_array_padded(b);
        let c_felts = to_felt_array_padded(c);

        let mut carry = const_expr!(0);
        for i in 0..TOTAL_SUBWORD_BUNDLES {
            // Compute the carry for this subword bundle.
            let mut curr_bits = 0;
            for j in 0..SUBWORD_BUNDLE_SIZE {
                let t = i * SUBWORD_BUNDLE_SIZE + j;
                carry = carry
                    + const_expr!(1 << curr_bits)
                        * (a_felts[t].clone() + b_felts[t].clone()
                            - c_felts[t].clone()
                            - p_felts[t].clone() * sub_p_bit.clone());
                if (t + 1).is_multiple_of(N_SUBWORDS_IN_WORD) {
                    curr_bits += LAST_SUBWORD_BIT_LEN;
                } else {
                    curr_bits += FELT252_BITS_PER_WORD;
                }
            }

            // Make sure the bundle is not too large.
            assert!(curr_bits <= 29);

            // Constrain the carry to be in {-1, 0, 1}, unless it is the last carry,
            // in which case it is constrained to 0.
            if i == (TOTAL_SUBWORD_BUNDLES - 1) {
                ab.constrain(carry.clone(), "last carry needs to be 0.");
            } else {
                carry = ab.assign(
                    // 1 << (31 - curr_bits) is 1 / (1 << curr_bits) in m31
                    &mut (carry * const_expr!(1 << (31 - curr_bits))),
                    &format!("carry_{i}"),
                );
                // Assert that carry is in {-1, 0, 1}.
                ab.constrain(
                    carry.clone() * (carry.clone() * carry.clone() - const_expr!(1)),
                    "carry is 0 or 1 or -1.",
                );
            }
        }
    }

    fn trace_type(&self) -> TraceType {
        TraceType::Builtin
    }
}

// Convert a Felt252Expr to a FeltExpr array with zero padding to length TOTAL_SUBWORDS_PADDED.
pub fn to_felt_array_padded(
    x: [Felt252Expr; MOD_BUILTIN_N_WORDS],
) -> [FeltExpr; TOTAL_SUBWORDS_PADDED] {
    from_fn(|i| {
        if i >= TOTAL_SUBWORDS {
            const_expr!(0)
        } else {
            x[i / N_SUBWORDS_IN_WORD].get_felt(i % N_SUBWORDS_IN_WORD)
        }
    })
}
