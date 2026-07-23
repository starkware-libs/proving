use stwo::core::pcs::PcsConfig;

/// Builds a default `PcsConfig` that lifts the preprocessed trace to
/// `trace_log_size + log_blowup_factor`.
pub fn default_circuit_pcs_config(trace_log_size: u32) -> PcsConfig {
    let mut pcs_config = PcsConfig::default();
    pcs_config.min_lifting_log_size = trace_log_size + pcs_config.fri_config.log_blowup_factor;
    pcs_config
}
