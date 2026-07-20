use std::path::Path;

use air_common::REGISTRY_PROPERTIES_FILE_NAME;
use air_infra::test_utils::{compare_json, compare_registry_jsons};

use crate::circuit::circuit_registry::create_circuit_registry;

#[test]
fn test_circuit_registry() {
    let reg = create_circuit_registry();

    compare_registry_jsons(&reg, Path::new("../../outputs/compiled_circuit_air/"));

    let stat = reg.collect_stats();
    compare_json(
        &stat,
        &Path::new("../../outputs/compiled_circuit_air/").join(REGISTRY_PROPERTIES_FILE_NAME),
    );
}
