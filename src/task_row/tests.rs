use super::{MAX_LABELS, ROW_HEIGHT, shown_labels};
use crate::task_model::Label;

fn labels(n: usize) -> Vec<Label> {
    (0..n).map(|i| Label::new(format!("label {i}"), i as u8)).collect()
}

#[test]
fn a_row_is_the_28_pixel_small_scale() {
    assert_eq!(ROW_HEIGHT, 28.);
}

#[test]
fn a_row_shows_two_labels_and_counts_the_rest() {
    assert_eq!(MAX_LABELS, 2);
    for (have, shown, more) in [(0, 0, 0), (1, 1, 0), (2, 2, 0), (3, 2, 1), (9, 2, 7)] {
        let list = labels(have);
        let (kept, rest) = shown_labels(&list);
        assert_eq!((kept.len(), rest), (shown, more), "{have} labels");
    }
    let list = labels(4);
    assert_eq!(shown_labels(&list).0[1].name.as_ref(), "label 1", "the first labels, in order");
}
