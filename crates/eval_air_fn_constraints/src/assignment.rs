use std::collections::HashMap;
use std::rc::Rc;

use compiled_casm_air::compiled_structs::{CompiledAirFn, ExternalState, PaddingType};
use stwo_cairo_common::prover_types::cpu::QM31;

use crate::util::{circle_point_from_t, random_m31, random_qm31, Environment, LookupElements};

/// Values for all variables that appear in the composition polynomial
pub struct Assignment {
    pub base_trace: Vec<QM31>,
    pub interaction_trace: Vec<QM31>,
    pub environment: Rc<Environment>,

    // The value of the logup sum in the row before this one.
    pub last_row_sum: QM31,

    // The value in the enabler/multiplicity column, if there is one.
    pub lookup_control_value: Option<QM31>,

    // The coefficients used to combine the elements of a tuple from a given relation into a single
    // value used for the logup.
    pub lookup_elements: HashMap<String, LookupElements>,

    // The total logup sum over all the rows of this component.
    pub claimed_sum: QM31,

    pub log_height: u32,

    pub point_x: QM31,
}

impl Assignment {
    pub fn new_random_for(component: &CompiledAirFn) -> Assignment {
        // Can be any small number. Large numbers can make `eval_vanishing_polynomial` slow.
        let log_height = 15;

        let (point_x, _) = circle_point_from_t(random_qm31(&"point_t".to_string()));

        let base_trace_len = component.state_names.len();
        let interaction_trace_len = component.constraint_lookups.len().div_ceil(2);

        let mut lookup_elements = HashMap::default();
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
            .map(|ext_state| {
                let state = if ext_state.name == "Seq" && ext_state.args.is_empty() {
                    &ExternalState {
                        name: "Seq".to_string(),
                        generic_param: None,
                        args: vec![log_height.to_string()],
                    }
                } else {
                    ext_state
                };
                (
                    ext_state.clone(),
                    random_qm31(&Assignment::external_state_id(state)),
                )
            })
            .collect();

        let lookup_control_value = match component.padding_type {
            PaddingType::Enabler | PaddingType::Multiplicity => {
                Some(random_qm31(&"enabler_or_multiplicity".to_string()))
            }
            PaddingType::None => None,
            _ => unimplemented!(),
        };

        Assignment {
            base_trace: (0..base_trace_len)
                .map(|i| random_qm31(&format!("base_{i}")))
                .collect(),
            interaction_trace: (0..interaction_trace_len)
                .map(|i| random_qm31(&format!("interaction_{i}")))
                .collect(),
            last_row_sum: random_qm31(&"last_row_sum".to_string()),
            lookup_control_value,
            lookup_elements,
            environment: Rc::new(Environment {
                public_params,
                external_states,
            }),
            claimed_sum: random_qm31(&"claimed_sum".to_string()),
            log_height,
            point_x,
        }
    }

    fn external_state_id(ext_state: &ExternalState) -> String {
        let name = &ext_state.name;
        let generic_argument = match ext_state.generic_param {
            Some(value) => format!("{value}"),
            None => "none".to_string(),
        };
        let args = ext_state.args.join(",");
        format!("{name}_{generic_argument}_{args}")
    }

    pub fn lookup_elements(&self, relation_name: &String) -> &LookupElements {
        self.lookup_elements
            .get(relation_name)
            .unwrap_or_else(|| panic!("Unknown relation {relation_name}"))
    }
}
