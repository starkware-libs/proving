use stwo_prover::core::air::{Air, AirProver, Component, ComponentProver};
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::CpuBackend;

use super::component::BitUnpack__12;

#[allow(non_camel_case_types)]
pub struct BitUnpack__12TestAIR {
    pub component: BitUnpack__12,
}

impl Air for BitUnpack__12TestAIR {
    fn components(&self) -> Vec<&dyn Component> {
        vec![&self.component]
    }
}

impl AirProver<CpuBackend> for BitUnpack__12TestAIR {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<CpuBackend>> {
        vec![&self.component]
    }
}

impl AirProver<SimdBackend> for BitUnpack__12TestAIR {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        vec![&self.component]
    }
}
