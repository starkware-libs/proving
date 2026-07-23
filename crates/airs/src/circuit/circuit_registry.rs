use air_infra::core::air_fn_registry::AirFnRegistry;

// Gates
use super::blake::blake_g_gate::BlakeGGate;
use super::blake::m31_to_u32::M31ToU32;
use super::blake::triple_xor::TripleXor;
use super::qm31_ops::Qm31Ops;

pub fn create_circuit_registry() -> AirFnRegistry {
    let mut registry = AirFnRegistry::new_empty();

    registry.add_entry(&Qm31Ops {});
    registry.add_entry(&BlakeGGate {});
    registry.add_entry(&M31ToU32 {});
    registry.add_entry(&TripleXor {});

    registry
}
