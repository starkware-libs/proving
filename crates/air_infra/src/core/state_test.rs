use super::expressions::felt_expr::*;
use super::state::*;
use super::variables::*;
// Macros
use crate::const_expr;
use crate::utils::test_utils::*;

#[test]
fn test_state_elements() {
    let mut state = State::default();
    for x in [1, 2, 3] {
        let mut e = const_expr!(x);
        state.add(&mut e, "");

        assert!(e.in_state());
    }
    let expected_state = vec![(1, ""), (2, ""), (3, "")].into();
    assert_expected_state(&state, &expected_state);
}
