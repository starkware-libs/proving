use super::*;

/// All the relation names declared by the circuit components.
const KNOWN_RELATIONS: [&str; 8] = [
    "Gate",
    "RangeCheck_16",
    "VerifyBitwiseXor_4",
    "VerifyBitwiseXor_7",
    "VerifyBitwiseXor_8",
    "VerifyBitwiseXor_8_B",
    "VerifyBitwiseXor_9",
    "VerifyBitwiseXor_12",
];

/// Check for misspellings of relation names. A misspelling affects the computation in
/// `check_relation_uses` and undercounts relation uses.
#[test]
fn test_regression_relation_ids() {
    for (name, component) in all_circuit_components::<QM31>() {
        for relation_use in component.relation_uses_per_row() {
            assert!(
                KNOWN_RELATIONS.contains(&relation_use.relation_id),
                "component {name} declares uses of unknown relation {:?}",
                relation_use.relation_id,
            );
        }
    }
}
