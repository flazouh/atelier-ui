use super::*;

#[test]
fn a_running_card_leads_with_its_live_tool_call() {
    assert_eq!(lead_text(None, Some("Read crates/ui/src/theme.rs".into())).as_deref(), Some("Read crates/ui/src/theme.rs"));
    assert_eq!(lead_text(None, None), None);
}

#[test]
fn a_finished_card_leads_with_how_long_it_ran() {
    assert_eq!(lead_text(Some(Some(38)), None).as_deref(), Some("Done in 38s"));
    assert_eq!(lead_text(Some(None), Some("Read a.rs".into())).as_deref(), Some("Done"));
}
