use std::rc::Rc;

use gpui_kit::{
    AnyElement, IntoElement, KeyDownEvent, Keystroke, ParentElement, Pixels, Styled, Window, div,
    prelude::FluentBuilder,
};

use super::structs::{Chip, PickerMorph};
use super::types::{Outcome, Pick, WIDTH};
use crate::scale::px;
use crate::{
    combobox::{ComboEntry, ComboList, ComboRow},
    motion::{cubic_bezier, ease},
    select::{header_inset, spring_unit, surface_at},
    task_edit::{Candidate, Change, Picker, Step},
    task_marks::{PriorityMark, TaskStatusMark, label_tone_color},
    theme::{Theme, popover_shadow, radius},
    typography::TextSize,
};

/// Routes one key to an open picker: `key` is the key's name (`escape`, `enter`, `up`, `down`,
/// `backspace`) and `text` the characters it types, if any.
pub fn handle_key(picker: &mut Picker, key: &str, text: Option<&str>) -> Outcome {
    match key {
        "escape" => Outcome::Close,
        "down" => {
            picker.step(Step::Down);
            Outcome::Open
        }
        "up" => {
            picker.step(Step::Up);
            Outcome::Open
        }
        "backspace" => {
            picker.backspace();
            Outcome::Open
        }
        "enter" => match picker.choose() {
            Some(change) => {
                let stays_open = picker.stays_open();
                if let Change::ToggleLabel(label) = &change {
                    let now_on = !picker
                        .candidates()
                        .iter()
                        .any(|c| c.change == change && c.chosen);
                    picker.mark(label, now_on);
                }
                Outcome::Chosen { change, stays_open }
            }
            None => Outcome::Open,
        },
        _ => {
            if let Some(text) = text.filter(|t| !t.is_empty() && !t.chars().any(char::is_control)) {
                picker.type_text(text);
            }
            Outcome::Open
        }
    }
}

/// The mark that goes before a candidate.
fn mark(candidate: &Candidate, theme: &Theme) -> AnyElement {
    match &candidate.change {
        Change::Status(status) => TaskStatusMark::new(*status).into_any_element(),
        Change::Priority(priority) => PriorityMark::new(*priority).into_any_element(),
        Change::Assignee(Some(who)) => {
            crate::task_row::assignee_mark("picker-assignee", who, 16., theme)
        }
        Change::Assignee(None) => div().size(px(16.)).into_any_element(),
        Change::ToggleLabel(label) => div()
            .flex()
            .size(px(16.))
            .items_center()
            .justify_center()
            .child(
                div()
                    .size(px(8.))
                    .rounded_full()
                    .bg(label_tone_color(label, theme)),
            )
            .into_any_element(),
    }
}

/// The picker as a card: what the cursor edits and the text typed, then the candidates in a [`ComboList`], with the
/// cursor's row lit by the list's gliding pill.
/// Enter, for an owner that takes a press on a row as the cursor going there and Enter after it.
pub fn enter() -> KeyDownEvent {
    KeyDownEvent {
        keystroke: Keystroke::parse("enter").expect("enter is a key"),
        is_held: false,
        prefer_character_input: false,
    }
}

/// The candidates as a [`ComboList`], the cursor's row lit by the list's gliding pill.
fn picker_list(id: &'static str, picker: &Picker, theme: &Theme, pick: Option<Pick>) -> ComboList {
    let shown = picker.shown();
    let entries = shown.iter().enumerate().map(|(row, &index)| {
        let candidate = &picker.candidates()[index];
        ComboEntry::from(
            ComboRow::new(candidate.words.clone())
                .debug_name(format!("picker-row-{row}"))
                .leading(mark(candidate, theme))
                .selected(candidate.chosen),
        )
    });
    ComboList::new(id, entries)
        .style(crate::combobox::ComboStyle::Task)
        .padded(false)
        .when_some(pick, |list, pick| {
            list.on_pick(move |i, window, cx| pick(i, window, cx))
        })
        .active((!shown.is_empty()).then(|| picker.cursor()))
        .empty("Nothing matches")
}

pub fn picker_view(
    id: &'static str,
    picker: &Picker,
    theme: &Theme,
    pick: Option<Pick>,
) -> AnyElement {
    let query = picker.query();
    let list = picker_list(id, picker, theme, pick);
    div()
        .w(px(WIDTH))
        .p(px(4.))
        .rounded(radius::lg())
        // design preview: remove after Alex picks (the elevation)
        .bg(crate::design_preview::panel_fill(
            theme,
            crate::design_preview::elevation(),
            theme.popover,
        ))
        .border_1()
        .border_color(crate::design_preview::panel_edge(
            theme,
            crate::design_preview::elevation(),
        ))
        .shadow(crate::design_preview::panel_shadows(
            theme,
            crate::design_preview::elevation(),
            popover_shadow(theme),
        ))
        .text_size(TextSize::Sm.font_size())
        .text_color(theme.foreground)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(28.))
                .px(px(8.))
                .text_size(TextSize::Xs.font_size())
                .text_color(theme.muted_foreground)
                .child(picker.field().words())
                .child(div().flex_1())
                .child(if query.is_empty() {
                    "Type to filter".to_string()
                } else {
                    query.to_string()
                }),
        )
        .child(list)
        .into_any_element()
}

/// The height of the picker's list: its shown rows (28 tall, 2 apart, at least one) in 4px of padding, at the cap.
pub fn list_height(picker: &Picker) -> f32 {
    let rows = picker.shown().len().max(1) as f32;
    8. + (rows * 28. + (rows - 1.) * 2.).min(crate::combobox::MAX_HEIGHT)
}

/// The surface of a picker that grew out of its chip. `face` is the chip's content, which the header shows at first and
/// hands to the filter as it opens. None when no picker is open or closing, or its chip is not known.
#[allow(clippy::too_many_arguments)]
pub fn morph_popover<T: 'static>(
    id: &'static str,
    morph: &PickerMorph,
    theme: &Theme,
    chip: Chip,
    face: AnyElement,
    window: gpui_kit::Size<Pixels>,
    owner: gpui_kit::WeakEntity<T>,
    close: fn(&mut T),
    pick: fn(&mut T, usize, &mut Window, &mut gpui_kit::Context<T>),
) -> Option<crate::popover::Popover> {
    use gpui_kit::{InteractiveElement, StatefulInteractiveElement};
    if !morph.shown() {
        return None;
    }
    let (picker, field) = (morph.last.as_ref()?, morph.field?);
    let anchor = morph.anchors[field.slot()]?;
    let p = morph.level();
    let lift = p.clamp(0., 1.);
    let (tw, th) = (f32::from(anchor.size.width), f32::from(anchor.size.height));
    let pw = WIDTH.max(tw);
    let list_h = morph.list.value().max(0.);
    let (w, h) = surface_at((tw, th), pw, list_h, p);
    let upward = crate::placement::opens_upward(anchor, list_h, 0., f32::from(window.height));
    // Too near the right edge for the picker's width: it grows toward the left, its right edge on the chip's.
    let end = f32::from(anchor.left()) + pw > f32::from(window.width) - 8.;
    let level = crate::design_preview::elevation();
    let fill = crate::theme::mix(
        chip.fill,
        crate::design_preview::panel_fill(theme, level, theme.popover),
        lift,
    );
    let shadows: Vec<gpui_kit::BoxShadow> =
        crate::design_preview::panel_shadows(theme, level, popover_shadow(theme))
            .into_iter()
            .map(|mut shadow| {
                shadow.color.a *= lift;
                shadow
            })
            .collect();
    let mut edge = crate::design_preview::panel_edge(theme, level);
    edge.a *= lift;
    let corner = chip.radius + (f32::from(radius::lg()) - chip.radius) * lift;
    let query = picker.query();
    let closer = owner.clone();
    let chooser = owner.clone();
    let pick: Pick = Rc::new(move |row, window, cx| {
        chooser
            .update(cx, |owner, cx| pick(owner, row, window, cx))
            .ok();
    });
    // The face fades out as the filter fades in, in the same row.
    let header = div()
        .id("picker-header")
        .debug_selector(|| "picker-header".into())
        .absolute()
        .left_0()
        .w_full()
        .h(px(th))
        .when(!upward, |d| d.top_0())
        .when(upward, |d| d.bottom_0())
        .on_click(move |_, _, cx| {
            closer
                .update(cx, |owner, cx| {
                    close(owner);
                    cx.notify();
                })
                .ok();
        })
        .child(
            div()
                .absolute()
                .top_0()
                .h_full()
                .w(px(tw))
                .when(end, |d| d.right_0())
                .when(!end, |d| d.left_0())
                .flex()
                .items_center()
                .px(px(chip.inset))
                .opacity(1. - lift)
                .child(face),
        )
        .child(
            div()
                .debug_selector(|| "picker-filter".into())
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .gap(px(6.))
                .px(px(header_inset(chip.inset, p)))
                .opacity(lift)
                .text_size(TextSize::Xs.font_size())
                .text_color(theme.muted_foreground)
                .child(picker.field().words())
                .child(div().flex_1())
                .child(if query.is_empty() {
                    "Type to filter".to_string()
                } else {
                    query.to_string()
                }),
        );
    // The rows come in a moment after the open, fading and rising as a select's options do.
    let t = morph.opened.map_or(1., |at| at.elapsed().as_secs_f32()) - 0.08;
    let (alpha, rise) = if t <= 0. {
        (0., -6.)
    } else {
        (
            cubic_bezier(ease::MOTION_DEFAULT, (t / 0.3).clamp(0., 1.)),
            -6. * (1. - spring_unit(500., 25., t)),
        )
    };
    let (alpha, rise) = if morph.reduce {
        (1., 0.)
    } else {
        (alpha, rise)
    };
    let list = div()
        .absolute()
        .left_0()
        .w_full()
        .h(px(list_h))
        .overflow_hidden()
        .when(!upward, |d| d.top(px(th + rise)))
        .when(upward, |d| d.bottom(px(th - rise)))
        .p(px(4.))
        .opacity(alpha)
        .text_size(TextSize::Sm.font_size())
        .child(picker_list(id, picker, theme, Some(pick)));
    let surface = div()
        .id("picker-surface")
        .debug_selector(|| "picker-surface".into())
        .relative()
        .w(px(w))
        .h(px(h))
        .rounded(px(corner))
        .overflow_hidden()
        .bg(fill)
        .shadow(shadows)
        .text_color(theme.foreground)
        .child(list)
        .child(header)
        .child(
            div()
                .absolute()
                .inset_0()
                .rounded(px(corner))
                .border_1()
                .border_color(edge),
        );
    let owner_close = owner;
    Some(
        crate::popover::Popover::new(id)
            .open(morph.open)
            .shown(true)
            .anchor(Some(anchor))
            .align(if end {
                crate::popover::Align::End
            } else {
                crate::popover::Align::Start
            })
            .side(if upward {
                crate::popover::Side::CoverAbove
            } else {
                crate::popover::Side::CoverBelow
            })
            .height(list_h)
            .width(px(pw))
            .keep_focus()
            .on_close(move |_, cx| {
                owner_close
                    .update(cx, |owner, cx| {
                        close(owner);
                        cx.notify();
                    })
                    .ok();
            })
            .child(surface),
    )
}

/// The picker as a popover hung from its owner's box. The owner keeps focus and its keys work the list;
/// a press outside, which the popover takes, closes it through `close`.
pub fn picker_popover<T: 'static>(
    id: &'static str,
    picker: &Picker,
    theme: &Theme,
    hang: crate::popover::Hang,
    owner: gpui_kit::WeakEntity<T>,
    close: fn(&mut T),
    pick: fn(&mut T, usize, &mut Window, &mut gpui_kit::Context<T>),
) -> crate::popover::Popover {
    let chooser = owner.clone();
    let pick: Pick = Rc::new(move |row, window, cx| {
        chooser
            .update(cx, |owner, cx| pick(owner, row, window, cx))
            .ok();
    });
    crate::popover::Popover::new(id)
        .open(true)
        .hang(hang)
        .keep_focus()
        .height(28. + 8. + crate::combobox::MAX_HEIGHT)
        .on_close(move |_, cx| {
            owner
                .update(cx, |owner, cx| {
                    close(owner);
                    cx.notify();
                })
                .ok();
        })
        .child(picker_view(id, picker, theme, Some(pick)))
}
