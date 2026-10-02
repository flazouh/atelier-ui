use super::*;

#[test]
fn the_view_in_front_has_a_wash_or_a_bar_when_its_sidebar_is_folded() {
    assert_eq!(mark(1, 1, true), Mark::Selected);
    assert_eq!(mark(1, 1, false), Mark::Folded);
    assert_eq!(mark(0, 1, true), Mark::Rest);
    assert_eq!(mark(2, 1, false), Mark::Rest);
}
