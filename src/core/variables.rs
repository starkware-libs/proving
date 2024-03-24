use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use super::expr_types::*;
use super::json_structs::*;

/// Every input and output of an air function is an AirVar.
#[allow(private_bounds)]
#[enum_dispatch]
pub trait AirVar: CoreAirVar + Clone + Debug + Default + Into<ProcessedAirVar> {
    fn new_copy(&self, name: String, in_state: bool) -> Self {
        let mut res = self.clone();
        res.set_name(name);
        if in_state {
            res.set_in_state();
        }
        res
    }
    fn name(&self) -> String;
    fn description(&self) -> String {
        self.name()
    }
    // Returns whether the value of this AirVar is stored in a trace cell.
    // For example, an input to an air function is not in state when it is from the private input.
    fn in_state(&self) -> bool;
    // Used to store variables in the state. When not in test mode, these felts are zeros.
    fn as_felts(&self) -> Vec<Felt>;
}

/// The functions of AirVar that are only intended to be used in the "core" part of the
/// library and not by the AirFn implementations.
#[enum_dispatch]
pub(super) trait CoreAirVar {
    #[allow(dead_code)]
    fn set_name(&mut self, name: String);
    #[allow(dead_code)]
    fn set_in_state(&mut self);
}
