use genco::lang::rust;
use genco::quote;

pub const TEST_AIR_SUFFIX: &str = "TestAIR";

pub fn generate_test_air_code(component_name: &str) -> rust::Tokens {
    quote! {
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


        $['\n']

        use super::component::$(component_name);
        $['\n']

        #[allow(non_camel_case_types)]
        pub struct $(component_name)$(TEST_AIR_SUFFIX) {
            pub component: $(component_name),
        }

        impl Air for $(component_name)$(TEST_AIR_SUFFIX) {
            fn components(&self) -> Vec<&dyn Component> {
                vec![&self.component]
            }
        }

        impl AirProver<CpuBackend> for $(component_name)$(TEST_AIR_SUFFIX) {
            fn prover_components(&self) -> Vec<&dyn ComponentProver<CpuBackend>> {
                vec![&self.component]
            }
        }

        impl AirProver<SimdBackend> for $(component_name)$(TEST_AIR_SUFFIX) {
            fn prover_components(&self) -> Vec<&dyn ComponentProver<SimdBackend>> {
                vec![&self.component]
            }
        }

        impl AirTraceVerifier for $(component_name)$(TEST_AIR_SUFFIX) {
            // Temporary until lookups codegen is implemented.
            fn interaction_elements(&self, _channel: &mut Blake2sChannel) -> InteractionElements {
                InteractionElements::default()
            }
        }

        impl AirTraceWriter<CpuBackend> for $(component_name)$(TEST_AIR_SUFFIX) {
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

        impl AirTraceWriter<SimdBackend> for $(component_name)$(TEST_AIR_SUFFIX) {
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
    }
}
