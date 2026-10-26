use circuit_cairo_verifier::all_components::all_components;
use circuit_cairo_verifier::privacy::PRIVACY_TRANSACTION_COMPONENTS;
use circuit_cairo_verifier::test_utils::privacy_pcs_config;
use stwo::core::fields::qm31::QM31;

use crate::consts::{
    CAIRO_LOG_BLOWUP_FACTOR, CAIRO_PCS_CONFIG, CAIRO_TRACE_LOG_SIZE, CIRCUIT_LOG_BLOWUP_FACTOR,
    CIRCUIT_PCS_CONFIG, CIRCUIT_TRACE_LOG_SIZE,
};

/// Test that the privacy test setup (`privacy_pcs_config`) matches the production configs.
#[test]
fn check_pcs_configs_match_test_setup() {
    assert_eq!(CAIRO_PCS_CONFIG, privacy_pcs_config(CAIRO_TRACE_LOG_SIZE, CAIRO_LOG_BLOWUP_FACTOR));
    assert_eq!(
        CIRCUIT_PCS_CONFIG,
        privacy_pcs_config(CIRCUIT_TRACE_LOG_SIZE, CIRCUIT_LOG_BLOWUP_FACTOR)
    );
}

#[test]
fn check_components() {
    let all_components = all_components::<QM31>();
    for component_name in PRIVACY_TRANSACTION_COMPONENTS {
        assert!(
            all_components.contains_key(component_name),
            "Component {component_name} is not in the all_components"
        );
    }
}
