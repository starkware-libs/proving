use std::rc::Rc;

use air_common::utils::{random_m31, random_qm31};
use air_compile::compiled_structs::CompiledAirFn;
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

    // The coefficients used to combine the elements of relation tuples.
    pub common_lookup_elements: LookupElements,

    // The total logup sum over all the rows of this component.
    pub claimed_sum: QM31,

    pub log_height: u32,
}

impl Assignment {
    pub fn new_random_for(component: &CompiledAirFn) -> Assignment {
        let log_height = component.log_height.unwrap_or(ASSIGNMENT_LOG_HEIGHT);

        let base_trace_len = component.state_names.len();
        let interaction_trace_len = component.constraint_lookups.len().div_ceil(2);

        let z = random_qm31("common_z");
        let alpha = random_qm31("common_alpha");
        let common_lookup_elements = LookupElements { z, alpha };

        let public_params = component
            .public_params
            .iter()
            .map(|param| (param.clone(), random_m31(param)))
            .collect();

        let external_states = component
            .external_states
            .iter()
            .map(|ext_state| (ext_state.clone(), random_qm31(&ext_state.to_owned())))
            .collect();

        let base_trace: Vec<_> =
            (0..base_trace_len).map(|i| random_qm31(&format!("base_{i}"))).collect();

        Assignment {
            base_trace,
            interaction_trace: (0..interaction_trace_len)
                .map(|i| random_qm31(&format!("interaction_{i}")))
                .collect(),
            random_coeff: random_qm31("random_coeff"),
            last_row_sum: random_qm31("last_row_sum"),
            common_lookup_elements,
            environment: Rc::new(Environment { public_params, external_states }),
            claimed_sum: random_qm31("claimed_sum"),
            log_height,
        }
    }
}
