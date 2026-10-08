use super::*;

/// "a", then one row removed for one added, then "b"; later two removed for none, then "c".
fn map() -> RowMap {
    RowMap::new(&[InlineHunk::new("one", 1..2, 2..3), InlineHunk::new("two", 4..6, 6..6)])
}

const SHOWN: &str = "a\nold\nnew\nb\ngone 1\ngone 2\nc";

#[test]
fn the_file_is_the_shown_text_without_its_removed_rows() {
    assert_eq!(map().head_text(SHOWN), "a\nnew\nb\nc");
    assert_eq!(RowMap::default().head_text(SHOWN), SHOWN, "no hunks: the text is the file");
}

#[test]
fn a_shown_row_maps_to_the_file_and_a_removed_one_to_nothing() {
    let map = map();
    let rows: Vec<Option<usize>> = (0..7).map(|r| map.to_head(r)).collect();
    assert_eq!(rows, [Some(0), None, Some(1), Some(2), None, None, Some(3)]);
}

#[test]
fn every_row_of_the_file_maps_back_to_where_it_shows() {
    let map = map();
    for head in 0..4 {
        let shown = map.to_view(head);
        assert_eq!(map.to_head(shown), Some(head), "row {head} shows at {shown}");
    }
    assert_eq!(map.to_view(3), 6);
}
