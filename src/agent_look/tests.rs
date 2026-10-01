use super::*;

#[test]
fn the_neutral_look_names_no_agent() {
    let labels = AgentLook::neutral(&crate::theme::Theme::dark()).labels;
    assert_eq!(labels.waiting, "Waiting for the agent…");
    assert_eq!(labels.thinking[0], (0., "Thinking…"));
}

#[test]
fn the_neutral_mark_is_one_still_frame() {
    let mark = AgentLook::neutral(&crate::theme::Theme::dark()).mark;
    assert_eq!(mark.working.frames, 1);
    assert_eq!(mark.working.next_frame_in(0), None);
    // Material draws on a 960 x 960 box, so one frame is square.
    let (w, h) = mark.working.native_size();
    assert_eq!(w, h);
}
