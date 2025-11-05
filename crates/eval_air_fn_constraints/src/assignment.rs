use std::collections::BTreeMap;
use std::rc::Rc;

use compiled_casm_air::compiled_structs::{CompiledAirFn, PaddingType};
use serde::{Deserialize, Serialize};
use stwo_cairo_common::prover_types::cpu::QM31;

use crate::util::*;

// Can be any small number. Large numbers can make `eval_vanishing_polynomial` slow.
pub const ASSIGNMENT_LOG_HEIGHT: u32 = 15;

/// Values for all variables that appear in the composition polynomial
#[derive(Serialize, Deserialize)]
pub struct Assignment {
    pub base_trace: Vec<QM31>,
    pub interaction_trace: Vec<QM31>,
    pub environment: Rc<Environment>,

    // Powers of this value are the coefficients used to combine the individual constraints
    // into the constraint polynomial.
    pub random_coeff: QM31,

    // The value of the logup sum in the row before this one.
    pub last_row_sum: QM31,

    // The value in the enabler/multiplicity column, if there is one.
    pub lookup_control_value: Option<QM31>,

    // The coefficients used to combine the elements of a tuple from a given relation into a single
    // value used for the logup.
    // We use a BTreeMap to have a stable order when serializing this to JSON (makes
    // it easier to do regression tests)
    pub lookup_elements: BTreeMap<String, LookupElements>,

    // The total logup sum over all the rows of this component.
    pub claimed_sum: QM31,

    pub log_height: u32,

    pub point: (QM31, QM31),
}

impl Assignment {
    pub fn new_random_for(component: &CompiledAirFn) -> Assignment {
        let log_height = ASSIGNMENT_LOG_HEIGHT;

        let point = circle_point_from_t(random_qm31(&"point_t".to_string()));

        let base_trace_len = component.state_names.len();
        let interaction_trace_len = component.constraint_lookups.len().div_ceil(2);

        let mut lookup_elements = BTreeMap::default();
        for (relation_name, ..) in component.constraint_lookups.iter() {
            if !lookup_elements.contains_key(relation_name) {
                let z = random_qm31(&format!("{relation_name}_z"));
                let alpha = random_qm31(&format!("{relation_name}_alpha"));
                lookup_elements.insert(relation_name.clone(), LookupElements { z, alpha });
            }
        }

        let public_params = component
            .public_params
            .iter()
            .map(|param| (param.name().clone(), random_m31(&param.name())))
            .collect();

        let external_states = component
            .external_states
            .iter()
            .map(|ext_state| (ext_state.clone(), random_qm31(&ext_state.to_owned())))
            .collect();

        let lookup_control_value = match component.padding_type {
            PaddingType::Enabler | PaddingType::Multiplicity => {
                Some(random_qm31(&"enabler_or_multiplicity".to_string()))
            }
            PaddingType::None => None,
        };

        Assignment {
            base_trace: (0..base_trace_len)
                .map(|i| random_qm31(&format!("base_{i}")))
                .collect(),
            interaction_trace: (0..interaction_trace_len)
                .map(|i| random_qm31(&format!("interaction_{i}")))
                .collect(),
            random_coeff: random_qm31(&"random_coeff".to_string()),
            last_row_sum: random_qm31(&"last_row_sum".to_string()),
            lookup_control_value,
            lookup_elements,
            environment: Rc::new(Environment {
                public_params,
                external_states,
            }),
            claimed_sum: random_qm31(&"claimed_sum".to_string()),
            log_height,
            point,
        }
    }

    pub fn lookup_elements(&self, relation_name: &String) -> &LookupElements {
        self.lookup_elements
            .get(relation_name)
            .unwrap_or_else(|| panic!("Unknown relation {relation_name}"))
    }
}
