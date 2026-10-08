use super::*;

#[test]
fn a_short_body_shows_all_of_itself() {
    assert_eq!(window(3, 8, true), 0..3);
    assert_eq!(window(8, 8, false), 0..8);
}

#[test]
fn a_streaming_body_shows_its_newest_rows() {
    assert_eq!(window(30, 8, true), 22..30);
}

#[test]
fn a_finished_body_shows_its_first_rows() {
    assert_eq!(window(30, 8, false), 0..8);
}

#[test]
fn the_first_press_on_a_clipped_body_opens_it_without_telling_the_owner() {
    assert_eq!(
        Press::on(false, true),
        Press {
            expanded: true,
            open: false
        }
    );
}

#[test]
fn the_second_press_folds_it_without_telling_the_owner() {
    assert_eq!(
        Press::on(true, true),
        Press {
            expanded: false,
            open: false
        }
    );
}

#[test]
fn a_body_that_fits_only_tells_the_owner() {
    assert_eq!(
        Press::on(false, false),
        Press {
            expanded: false,
            open: true
        }
    );
}
