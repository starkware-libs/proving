#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum PublicParam {
    BitwiseBuiltinSegmentStart,
}

impl PublicParam {
    pub fn name(&self) -> String {
        match self {
            PublicParam::BitwiseBuiltinSegmentStart => "bitwise_builtin_segment_start".to_string(),
        }
    }
}
