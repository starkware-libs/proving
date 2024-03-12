use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use super::json_structs::*;

/// Every input and output of an air function is an AirVar.
#[allow(private_bounds)]
#[enum_dispatch]
pub trait AirVar: Debug + Clone + CoreAirVar {
    fn get_var_info(&self) -> AirVarInfo {
        AirVarInfo {
            name: self.name(),
            description: self.description(),
            in_state: self.in_state(),
            r#type: self.var_type(),
        }
    }
    fn name(&self) -> String;
    fn description(&self) -> String {
        self.name()
    }
    // Returns whether the value of this AirVar is stored in a trace cell.
    // For example, an input to an air function is not in state when it is from the private input.
    fn in_state(&self) -> bool;
    fn var_type(&self) -> AirVarType;
}

/// The functions of AirVar that are only intended to be used in the "core" part of the
/// library and not by the AirFn implementations.
#[enum_dispatch]
pub(super) trait CoreAirVar: Default {
    #[allow(dead_code)]
    fn set_name(&mut self, name: String);
    #[allow(dead_code)]
    fn set_in_state(&mut self);
}
