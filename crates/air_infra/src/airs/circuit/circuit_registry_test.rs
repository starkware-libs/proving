use std::path::Path;

use air_common::REGISTRY_PROPERTIES_FILE_NAME;

use crate::airs::circuit::circuit_registry::create_circuit_registry;
use crate::test_utils::{compare_json, compare_registry_jsons};

#[test]
fn test_circuit_registry() {
    let reg = create_circuit_registry();

    compare_registry_jsons(&reg, Path::new("../compiled_circuit_air/"));

    let stat = reg.collect_stats();
    compare_json(
        &stat,
        &Path::new("../compiled_circuit_air/").join(REGISTRY_PROPERTIES_FILE_NAME),
    );
}
