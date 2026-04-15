// Gates
use super::blake::blake_gate::BlakeGate;
use super::blake::blake_output::BlakeOutput;
use super::blake::m31_to_u32::M31ToU32;
use super::qm31_ops::Qm31Ops;
use crate::core::air_fn_registry::AirFnRegistry;

pub fn create_circuit_registry() -> AirFnRegistry {
    let mut registry = AirFnRegistry::new_empty();

    registry.add_entry(&Qm31Ops {});
    registry.add_entry(&BlakeGate {});
    registry.add_entry(&BlakeOutput {});
    registry.add_entry(&M31ToU32 {});

    registry
}
