use expect_test::expect;

use super::expressions::felt_expr::*;
use super::state::*;
use super::variables::*;
// Macros
use crate::const_expr;

#[test]
fn test_state_elements() {
    let mut state = State::default();
    for x in [1, 2, 3] {
        let mut e = const_expr!(x);
        state.add(&mut e, "");

        assert!(e.in_state());
    }
    expect![[r#"
        (1, ""),
        (2, ""),
        (3, ""),
    "#]]
    .assert_eq(&state.to_string());
}
