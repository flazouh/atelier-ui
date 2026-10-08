use super::*;

fn commit(sha: &str, at: u64, age: &str) -> CommitData {
    CommitData { sha: sha.into(), title: "t".into(), author: "a".into(), age: age.into(), at }
}

#[test]
fn the_summary_counts_and_names_the_newest() {
    assert_eq!(how_many(&[]), "none yet");
    assert_eq!(how_many(&[commit("a", 1, "2h ago")]), "one, 2h ago");
    let six: Vec<_> = (0..6).map(|i| commit("x", i, if i == 5 { "2h ago" } else { "3d ago" })).collect();
    assert_eq!(how_many(&six), "6, newest 2h ago");
    assert_eq!(how_many(&[commit("a", 1, ""), commit("b", 2, "")]), "2");
}
