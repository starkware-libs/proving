use expect_test::{Expect, expect};

pub const BLAKE_G_GATE: Expect = expect![[r#"
    (1521326994 + 1558819476i) + (524478452 + 1856781144i)u
"#]];
pub const M_31_TO_U_32: Expect = expect![[r#"
    (2021899343 + 236205997i) + (320574209 + 1929315023i)u
"#]];
pub const QM_31_OPS: Expect = expect![[r#"
    (390906521 + 2047144348i) + (1587657629 + 941353825i)u
"#]];
pub const RANGE_CHECK_15: Expect = expect![[r#"
    (2052111311 + 1192528295i) + (686829220 + 1363764450i)u
"#]];
pub const RANGE_CHECK_16: Expect = expect![[r#"
    (468645923 + 825426857i) + (1414222341 + 1345490780i)u
"#]];
pub const TRIPLE_XOR: Expect = expect![[r#"
    (914482770 + 1912998134i) + (343192498 + 971866381i)u
"#]];
pub const TRIPLE_XOR_32: Expect = expect![[r#"
    (1152376846 + 753053024i) + (1408555470 + 1523324023i)u
"#]];
pub const VERIFY_BITWISE_XOR_4: Expect = expect![[r#"
    (1989442030 + 354737447i) + (1017224845 + 993909400i)u
"#]];
pub const VERIFY_BITWISE_XOR_7: Expect = expect![[r#"
    (144747908 + 545875633i) + (517815551 + 1019649583i)u
"#]];
pub const VERIFY_BITWISE_XOR_8: Expect = expect![[r#"
    (998149202 + 235538579i) + (1144376034 + 121409592i)u
"#]];
pub const VERIFY_BITWISE_XOR_9: Expect = expect![[r#"
    (1704264155 + 1827004954i) + (1982906079 + 397183327i)u
"#]];
