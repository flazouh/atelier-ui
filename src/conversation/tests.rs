use super::*;

fn thread(resolved: bool) -> ThreadSummary {
    ThreadSummary {
        people: vec!["Maya".into()],
        comments: vec![],
        first: "x".into(),
        resolved,
    }
}

#[test]
fn the_header_leads_with_what_is_still_open() {
    let t = |open: usize, resolved: usize| {
        let mut v: Vec<ThreadSummary> = (0..open).map(|_| thread(false)).collect();
        v.extend((0..resolved).map(|_| thread(true)));
        v
    };
    assert_eq!(said_so_far(&t(4, 1), 3), "4 open, 1 resolved, 3 remarks");
    assert_eq!(said_so_far(&t(2, 0), 0), "2 open");
    assert_eq!(said_so_far(&t(0, 5), 1), "all 5 resolved, 1 remark");
    assert_eq!(said_so_far(&[], 2), "2 remarks");
    assert_eq!(said_so_far(&[], 0), "nothing said yet");
}

#[test]
fn open_threads_come_before_resolved_ones_and_keep_their_order() {
    let mut a = thread(true);
    a.first = "a".into();
    let mut b = thread(false);
    b.first = "b".into();
    let mut c = thread(false);
    c.first = "c".into();
    let order: Vec<_> = open_first(vec![a, b, c])
        .into_iter()
        .map(|t| t.first.to_string())
        .collect();
    assert_eq!(order, ["b", "c", "a"]);
}

#[test]
fn a_line_shows_three_faces_then_a_count() {
    assert_eq!(faces(&["a".into(), "b".into()]), (2, None));
    assert_eq!(
        faces(&["a".into(), "b".into(), "c".into(), "d".into(), "e".into()]),
        (3, Some("+2".into()))
    );
}

#[test]
fn a_page_of_a_long_conversation_keeps_the_header_of_all_of_it() {
    assert_eq!(
        said_by_count(1_000, 20, 300),
        "1000 open, 20 resolved, 300 remarks"
    );
    assert_eq!(said_by_count(0, 0, 0), "nothing said yet");
    assert_eq!(said_by_count(0, 0, 1), "1 remark");
    assert_eq!(
        said_so_far(&[thread(false), thread(true)], 1),
        said_by_count(1, 1, 1)
    );
}
