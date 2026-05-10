pub mod air_body;
pub mod air_fn;
pub mod air_fn_registry;
pub mod constraint_connectedness_test;
pub mod expressions;
pub mod memory;
pub mod public_params;
pub mod state;
pub mod struct_var;
pub mod variables;

pub type Felt = stwo_cairo_common::prover_types::cpu::M31;

#[cfg(test)]
mod air_fn_test;
#[cfg(test)]
mod state_test;
#[cfg(test)]
mod variables_test;
