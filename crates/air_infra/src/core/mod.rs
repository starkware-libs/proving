pub mod air_fn;
pub mod air_fn_registry;
pub mod expressions;
pub mod memory;
pub mod state;
pub mod variables;

pub type Felt = stwo_prover::core::fields::m31::M31;

#[cfg(test)]
mod air_fn_test;
#[cfg(test)]
mod state_test;
#[cfg(test)]
mod variables_test;
