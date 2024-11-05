#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum PublicParam {
    BitwiseBuiltinSegmentStart,
    RangeCheckBuiltinSegmentStart,
}

impl PublicParam {
    pub fn name(&self) -> String {
        match self {
            PublicParam::BitwiseBuiltinSegmentStart => "bitwise_builtin_segment_start".to_string(),
            PublicParam::RangeCheckBuiltinSegmentStart => {
                "range_check_builtin_segment_start".to_string()
            }
        }
    }
}
