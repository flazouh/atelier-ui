use super::*;

#[test]
fn an_avatar_shows_the_authors_first_letter() {
    assert_eq!(Comment::new("alex", "2m ago", "Looks off by one.").initial(), "A");
    assert_eq!(Comment::new("émile", "now", "x").initial(), "É");
    assert_eq!(Comment::new("", "now", "x").initial(), "");
}
