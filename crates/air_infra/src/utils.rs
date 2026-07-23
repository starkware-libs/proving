use stwo_cairo_common::prover_types::cpu::FELT252_BITS_PER_WORD;

use crate::const_expr;
use crate::core::expressions::felt_expr::*;
use crate::core::expressions::felt252_expr::*;

pub fn felt252_to_m31(value: Felt252Expr, num_bits: usize) -> FeltExpr {
    assert!(num_bits <= 31, "{num_bits} bits can't fit in M31");
    let mut result = value.get_felt(0);

    for i in 1..(num_bits.div_ceil(FELT252_BITS_PER_WORD)) {
        result = result + value.get_felt(i) * const_expr!(1 << (FELT252_BITS_PER_WORD * i));
    }

    result
}

pub fn get_relation_variant_names(base_name: &str, n_variants: usize) -> Vec<String> {
    let mut names = Vec::with_capacity(n_variants);
    for i in 0..n_variants {
        if i == 0 {
            names.push(base_name.to_string());
        } else {
            names.push(format!("{}_{}", base_name, (b'B' + (i - 1) as u8) as char));
        }
    }
    names
}
