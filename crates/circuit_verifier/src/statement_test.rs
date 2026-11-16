use super::*;
use crate::relations::RELATION_NAMES;

/// Check for misspellings of relation names. A misspelling affects the computation in
/// `check_relation_uses` and undercounts relation uses.
#[test]
fn test_regression_relation_ids() {
    for (name, component) in all_circuit_components::<QM31>().into_named_iter() {
        for relation_use in component.relation_uses_per_row() {
            assert!(
                RELATION_NAMES.contains(&relation_use.relation_id),
                "component {name} declares uses of unknown relation {:?}",
                relation_use.relation_id,
            );
        }
    }
}
