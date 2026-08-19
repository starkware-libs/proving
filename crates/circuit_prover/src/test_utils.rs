use stwo::core::pcs::PcsConfig;

/// Builds a default `PcsConfig` that lifts every tree, the preprocessed one included, to
/// `trace_log_size + log_blowup_factor`.
pub fn default_circuit_pcs_config(trace_log_size: u32) -> PcsConfig {
    PcsConfig::from_fri_and_trace_size(PcsConfig::default().fri_config, trace_log_size)
}
