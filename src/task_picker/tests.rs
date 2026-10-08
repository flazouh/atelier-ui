use super::{Outcome, handle_key};
use crate::{
    task_edit::{Change, Picker},
    task_model::{Label, Priority, TaskStatus},
};

#[test]
fn enter_chooses_the_candidate_under_the_cursor_and_closes_the_picker() {
    let mut picker = Picker::status(Some(TaskStatus::Todo));
    assert_eq!(handle_key(&mut picker, "down", None), Outcome::Open);
    assert_eq!(handle_key(&mut picker, "enter", None), Outcome::Chosen { change: Change::Status(TaskStatus::InProgress), stays_open: false });
}

#[test]
fn escape_closes_and_typing_filters_and_backspace_widens_again() {
    let mut picker = Picker::priority(None);
    assert_eq!(handle_key(&mut picker, "escape", None), Outcome::Close);
    assert_eq!(handle_key(&mut picker, "h", Some("h")), Outcome::Open);
    assert_eq!(picker.shown().len(), 1, "only High has an h");
    assert_eq!(handle_key(&mut picker, "enter", None), Outcome::Chosen { change: Change::Priority(Priority::High), stays_open: false });
    assert_eq!(handle_key(&mut picker, "backspace", None), Outcome::Open);
    assert_eq!(picker.shown().len(), 5);
}

#[test]
fn keys_that_type_nothing_are_ignored() {
    let mut picker = Picker::priority(None);
    for (key, text) in [("shift", None), ("tab", Some("\t")), ("left", Some("")), ("f1", None)] {
        assert_eq!(handle_key(&mut picker, key, text), Outcome::Open);
    }
    assert_eq!(picker.query(), "", "nothing was typed");
}

#[test]
fn enter_with_no_match_does_nothing() {
    let mut picker = Picker::status(None);
    handle_key(&mut picker, "z", Some("z"));
    handle_key(&mut picker, "z", Some("z"));
    assert_eq!(handle_key(&mut picker, "enter", None), Outcome::Open);
}

#[test]
fn a_label_choice_keeps_the_picker_open_and_flips_its_mark() {
    let all = [Label::new("bug", 1), Label::new("ui", 2)];
    let mut picker = Picker::labels(&all, &[]);
    let chosen = handle_key(&mut picker, "enter", None);
    assert_eq!(chosen, Outcome::Chosen { change: Change::ToggleLabel(all[0].clone()), stays_open: true });
    assert!(picker.candidates()[0].chosen, "the label shows as on");
    handle_key(&mut picker, "enter", None);
    assert!(!picker.candidates()[0].chosen, "the same choice again takes it off");
}

#[test]
fn the_cursor_moves_with_the_arrows_inside_the_matches() {
    let mut picker = Picker::status(None);
    handle_key(&mut picker, "down", None);
    handle_key(&mut picker, "down", None);
    assert_eq!(picker.cursor(), 2);
    handle_key(&mut picker, "up", None);
    assert_eq!(picker.cursor(), 1);
}

mod morph {
    use super::super::*;
    use crate::task_edit::{Field, Picker};
    use crate::task_model::{Priority, TaskStatus};

    fn status() -> Picker {
        Picker::status(Some(TaskStatus::Todo))
    }

    /// A picker opens the surface from its field's chip, and a close brings it back.
    #[test]
    fn the_surface_opens_with_the_picker_and_is_gone_once_it_has_closed() {
        let mut m = PickerMorph::default();
        assert!(!m.shown());
        m.sync(None, true);
        assert!(!m.shown());
        let picker = status();
        m.sync(Some(&picker), true);
        assert!(m.shown() && m.hides(Field::Status) && !m.hides(Field::Priority));
        assert_eq!(m.level(), 1., "under Reduce Motion it is open at once");
        m.sync(None, true);
        m.sync(None, true);
        assert!(!m.shown(), "a closed picker leaves no surface behind");
    }

    #[test]
    fn a_picker_that_closes_keeps_its_rows_until_the_surface_is_back() {
        let mut m = PickerMorph::default();
        m.sync(Some(&status()), false);
        std::thread::sleep(std::time::Duration::from_millis(60));
        m.sync(None, false);
        assert!(m.shown(), "the surface is still on its way back");
        assert!(m.rows().is_some(), "and still has the rows it shows");
    }

    /// The list is as tall as its shown rows, 28 each with 2 between, inside 4px of padding, at most the list's cap.
    #[test]
    fn the_list_is_as_tall_as_its_shown_rows() {
        let mut picker = status();
        let all = picker.shown().len() as f32;
        assert_eq!(list_height(&picker), 8. + all * 28. + (all - 1.) * 2.);
        picker.type_text("zzzz");
        assert_eq!(list_height(&picker), 8. + 28., "nothing shown is still one row tall");
        let many = Picker::priority(Some(Priority::None));
        assert!(list_height(&many) <= 8. + crate::combobox::MAX_HEIGHT);
    }

    /// Switching to another field starts the surface again from that field's chip.
    #[test]
    fn another_field_starts_again_from_its_own_chip() {
        let mut m = PickerMorph::default();
        m.sync(Some(&status()), true);
        m.sync(Some(&Picker::priority(Some(Priority::None))), true);
        assert!(m.hides(Field::Priority) && !m.hides(Field::Status));
    }

    #[test]
    fn a_field_with_no_chip_cannot_morph() {
        let mut m = PickerMorph::default();
        assert!(!m.can_morph(Field::Status));
        m.set_anchor(Field::Status, gpui_kit::Bounds::new(gpui_kit::point(gpui_kit::px(1.), gpui_kit::px(2.)), gpui_kit::size(gpui_kit::px(80.), gpui_kit::px(28.))));
        assert!(m.can_morph(Field::Status) && !m.can_morph(Field::Labels));
    }
}
