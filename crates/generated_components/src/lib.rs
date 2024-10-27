#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
#![allow(non_snake_case)]
#![feature(portable_simd)]

pub mod addapopcode_is_imm_t_op1_base_fp_f;
pub mod memoryaddresstoid;
pub mod memoryidtobig;
pub mod narrowfib_num_steps_20;
pub mod opcodes;
pub mod rangecheck_n_2_bits_4_3;
pub mod rangecheck_n_3_bits_7_2_5;
pub mod verifyinstruction;
pub mod widefib_num_narrow_8_narrow_size_20;

// TODO(Ohad): make this 2.
pub const LOGUP_BATCH_SIZE: usize = 1;
