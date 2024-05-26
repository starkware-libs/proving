use genco::lang::rust;
use genco::quote;

pub const TEST_AIR_SUFFIX: &str = "TestAIR";

pub fn generate_test_air_code(component_name: &str) -> rust::Tokens {
    quote! {
        use stwo_prover::core::air::{Air, AirProver, Component, ComponentProver};
        use stwo_prover::core::backend::CpuBackend;
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
    }
}
