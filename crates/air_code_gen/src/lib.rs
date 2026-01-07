#![feature(portable_simd)]
#![feature(iter_array_chunks)]
#![allow(incomplete_features)]
#![feature(generic_const_exprs)]
pub mod cairo_claim_generator;
pub mod components_code_gen;
pub mod utils;

#[cfg(test)]
mod tests;
