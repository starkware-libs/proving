use serde::{Deserialize, Serialize};

/// The information about an AirVar that is used in the JSON representation of an AirFn.
/// See get_var_info function in AirVar trait.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AirVarInfo {
    pub name: String,
    pub description: String,
    pub in_state: bool,
    pub r#type: AirVarType,
}

/// All the types of structs that implement AirVar.
/// See var_type function in AirVar trait.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AirVarType {
    // TODO: Add more types and remove None.
    None,
}
