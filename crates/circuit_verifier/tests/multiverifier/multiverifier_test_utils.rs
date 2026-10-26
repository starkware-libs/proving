// The shared fixtures live in `circuit_prover::test_utils` so that other crates' tests can use
// them too; this module re-exports them for the multiverifier tests.
pub use circuit_prover::test_utils::*;
