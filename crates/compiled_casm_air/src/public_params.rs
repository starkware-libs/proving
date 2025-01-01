#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum PublicParam {
    AddModBuiltinSegmentStart,
    BitwiseBuiltinSegmentStart,
    RangeCheckBuiltinSegmentStart,
    RangeCheck96BuiltinSegmentStart,
}

impl PublicParam {
    pub fn name(&self) -> String {
        match self {
            PublicParam::AddModBuiltinSegmentStart => "add_mod_builtin_segment_start".to_string(),
            PublicParam::BitwiseBuiltinSegmentStart => "bitwise_builtin_segment_start".to_string(),
            PublicParam::RangeCheckBuiltinSegmentStart => {
                "range_check_builtin_segment_start".to_string()
            }
            PublicParam::RangeCheck96BuiltinSegmentStart => {
                "range_check96_builtin_segment_start".to_string()
            }
        }
    }
}
