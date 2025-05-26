#[derive(PartialEq)]
pub enum AutogenCodeType {
    WITNESS,
    AIR,
}

pub struct AutogenCodeFile {
    /// Path of the source JSON relative to the compiled JSONs root directory
    /// (e.g. "opcodes/ret_opcode.json")
    pub source_rel_path: String,
    pub code_type: AutogenCodeType,
}

pub fn get_supported_components() -> Vec<AutogenCodeFile> {
    vec![AutogenCodeFile {
        source_rel_path: "opcodes/ret_opcode.json".into(),
        code_type: AutogenCodeType::AIR,
    }]
}
