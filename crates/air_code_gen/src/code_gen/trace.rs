use air_infra::core::autogen_structs::{DeductionOrIntermediate, ProcessedAirVar};
use genco::lang::rust;

/// Parses a `ProcessedAirVar` into a string for the write_trace function.
pub fn parse_air_var(_expr: &ProcessedAirVar) -> String {
    unimplemented!()
}

/// Outputs the code for the write_trace function.
#[allow(dead_code)]
fn gen_write_trace_code(
    _input: ProcessedAirVar,
    _deductions: &[DeductionOrIntermediate],
) -> rust::Tokens {
    unimplemented!()
}
