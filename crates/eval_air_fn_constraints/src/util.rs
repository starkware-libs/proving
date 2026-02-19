use std::collections::BTreeMap;

use air_common::ExternalState;
use indexmap::IndexMap;
use num_traits::One;
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::{M31, QM31};

/// The values of "globals" that are available in all scopes.
#[derive(Serialize, Deserialize)]
pub struct Environment {
    // We use a BTreeMap to have a stable order when serializing this to JSON (makes
    // it easier to do regression tests)
    pub public_params: BTreeMap<String, M31>,

    // IndexMap is only used here because of its ability to serialize maps with non-string keys
    // (`ExternalState` in our case). We do not care about the order of the external states.
    #[serde(with = "indexmap::map::serde_seq")]
    pub external_states: IndexMap<ExternalState, QM31>,
}

/// The interaction elements for a specific relation.
#[derive(Debug, Serialize, Deserialize)]
pub struct LookupElements {
    pub z: QM31,
    pub alpha: QM31,
}

impl LookupElements {
    /// Compute the logup denominator for a given relation tuple
    /// The result is -z + alpha * tuple[0] + alpha^2 * tuple[1] + alpha^3 * tuple[2] ...
    pub fn compute_logup_denominator(&self, tuple: &Vec<QM31>) -> QM31 {
        let mut result = -self.z;
        let mut coeff = QM31::one();

        for elem in tuple {
            result += coeff * *elem;
            coeff *= self.alpha;
        }

        result
    }
}
