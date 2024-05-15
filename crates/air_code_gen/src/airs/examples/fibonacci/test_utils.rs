use stwo_prover::core::air::{Air, AirProver, Component, ComponentProver};
use stwo_prover::core::backend::CPUBackend;

use super::component::Fib__100;

#[allow(non_camel_case_types)]
pub struct Fib__100TestAIR {
    pub component: Fib__100,
}

impl Air for Fib__100TestAIR {
    fn components(&self) -> Vec<&dyn Component> {
        vec![&self.component]
    }
}

impl AirProver<CPUBackend> for Fib__100TestAIR {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<CPUBackend>> {
        vec![&self.component]
    }
}
