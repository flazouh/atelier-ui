use super::*;

#[test]
fn the_letters_must_come_in_order() {
    assert!(score("srv", "src/server.rs").is_some());
    assert!(score("vrs", "src/server.rs").is_some());
    assert!(score("zz", "src/server.rs").is_none());
    assert!(score("rsv", "server").is_none(), "r s v in that order is not in server");
}

#[test]
fn case_is_ignored_and_an_empty_query_matches() {
    assert!(score("SERVER", "src/server.rs").is_some());
    assert_eq!(score("", "anything"), Some(0));
}

#[test]
fn a_word_start_and_a_run_of_letters_rank_first() {
    let paths = ["src/observer.rs", "src/server.rs", "src/http/serve.rs"];
    let ranked = rank("server", paths, 10);
    assert_eq!(paths[ranked[0]], "src/server.rs", "the whole word after a slash beats one inside a word");
    let names = ["request_context", "RequestContext", "reqctx"];
    assert_eq!(rank("rc", names, 10)[0], 1, "a capital after a small letter starts a word");
}

#[test]
fn a_tie_goes_to_the_shorter_text_and_the_limit_holds() {
    let texts = ["a/lib.rs", "lib.rs", "b/c/lib.rs"];
    assert_eq!(rank("lib", texts, 2), [1, 0]);
}
