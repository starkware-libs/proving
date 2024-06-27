use stwo_prover::core::air::{
    Air, AirProver, AirTraceVerifier, AirTraceWriter, Component, ComponentProver,
};
use stwo_prover::core::backend::simd::SimdBackend;
use stwo_prover::core::backend::CpuBackend;
use stwo_prover::core::channel::Blake2sChannel;
use stwo_prover::core::fields::m31::BaseField;
use stwo_prover::core::poly::circle::CircleEvaluation;
use stwo_prover::core::poly::BitReversedOrder;
use stwo_prover::core::{ColumnVec, ComponentVec, InteractionElements};

use super::component::NarrowFib__20;

#[allow(non_camel_case_types)]
pub struct NarrowFib__20TestAIR {
    pub component: NarrowFib__20,
}

impl Air for NarrowFib__20TestAIR {
    fn components(&self) -> Vec<&dyn Component> {
        vec![&self.component]
    }
}

impl AirProver<CpuBackend> for NarrowFib__20TestAIR {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<CpuBackend>> {
        vec![&self.component]
    }
}

impl AirProver<SimdBackend> for NarrowFib__20TestAIR {
    fn prover_components(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
        vec![&self.component]
    }
}

impl AirTraceVerifier for NarrowFib__20TestAIR {
    fn interaction_elements(&self, _channel: &mut Blake2sChannel) -> InteractionElements {
        InteractionElements::default()
    }
}

impl AirTraceWriter<CpuBackend> for NarrowFib__20TestAIR {
    fn interact(
        &self,
        _trace: &ColumnVec<CircleEvaluation<CpuBackend, BaseField, BitReversedOrder>>,
        _elements: &InteractionElements,
    ) -> ComponentVec<CircleEvaluation<CpuBackend, BaseField, BitReversedOrder>> {
        ComponentVec(vec![vec![]])
    }

    fn to_air_prover(&self) -> &impl AirProver<CpuBackend> {
        self
    }
}

impl AirTraceWriter<SimdBackend> for NarrowFib__20TestAIR {
    fn interact(
        &self,
        _trace: &ColumnVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>>,
        _elements: &InteractionElements,
    ) -> ComponentVec<CircleEvaluation<SimdBackend, BaseField, BitReversedOrder>> {
        ComponentVec(vec![vec![]])
    }

    fn to_air_prover(&self) -> &impl AirProver<SimdBackend> {
        self
    }
}
