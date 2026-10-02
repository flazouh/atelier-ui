use std::sync::Arc;

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement,
    Styled, div, prelude::FluentBuilder,
};

use crate::{
    button::{Button, ButtonSize, ButtonVariant},
    icon::IconName,
    menu::{self, Entry, Menu, MenuItem, Origin},
    placement::measure,
    popover::{Align, Popover},
    review::{ReviewHandler, ReviewHandlers, caps},
    tooltip::Tooltip,
    };
use super::structs::BarState;
use super::types::FIT_SLACK;

/// What a step needs, given what its parts measured (`sum`), the bar's `width`, the count's own width and the width
/// of the box that holds it. The sum is a guess that layout can beat by a pixel or two, or by more. If the count
/// does not fit its box, the step does not fit at this width whatever the sum says: it needs more than `width`.
pub fn need_at(sum: f32, width: f32, summary: f32, summary_box: f32) -> f32 {
    let need = sum + FIT_SLACK;
    if summary > summary_box + 0.5 { need.max(width + 1.) } else { need }
}

/// The step to show at `width`, given what each step was measured to need so far: the first whose need
/// fits, or the first not measured yet, which is tried next. The narrowest when nothing fits.
pub fn choose(width: f32, needed: &[Option<f32>]) -> usize {
    needed.iter().position(|need| need.is_none_or(|n| n <= width)).unwrap_or(needed.len().saturating_sub(1))
}

/// A small button with its words and cap, or with its icon and cap when the bar is too narrow for the
/// words, which then move to its tooltip. It runs `handler` when pressed and sits disabled without one.
pub(crate) fn worded(
    button: Button,
    words: &'static str,
    icon: IconName,
    with_words: bool,
    handler: &Option<ReviewHandler>,
) -> impl IntoElement {
    let button = button.size(ButtonSize::Sm);
    let button = if with_words { button.label(words) } else { button.icon(icon) };
    let button = match handler.clone() {
        Some(f) => button.on_click(move |_, window, cx| f(window, cx)),
        None => button.disabled(true),
    };
    div()
        .id(ElementId::Name(format!("{words}-button").into()))
        .flex_none()
        .when(!with_words, |d| d.tooltip(Tooltip::text(words)))
        .child(button)
}

/// What the ⋯ menu holds, in order: each entry the owner handles, with its cap.
pub fn menu_entries(h: &ReviewHandlers) -> Vec<(&'static str, Option<&'static str>, ReviewHandler)> {
    [
        ("Accept all", Some(caps::ACCEPT_ALL), &h.on_accept_all),
        ("Reject all", Some(caps::REJECT_ALL), &h.on_reject_all),
        ("Put all back", None, &h.on_put_back),
    ]
    .into_iter()
    .filter_map(|(words, cap, f)| f.clone().map(|f| (words, cap, f)))
    .collect()
}

/// The ⋯ button and its menu.
pub(super) fn menu(id: &ElementId, state: &gpui_kit::Entity<BarState>, h: &ReviewHandlers, open: bool, cx: &mut App) -> impl IntoElement {
    let child = |name: &'static str| ElementId::NamedChild(Arc::new(id.clone()), name.into());
    let (toggle, close) = (state.clone(), state.clone());
    let entries = menu_entries(h);
    div()
        .relative()
        .flex_none()
        .child(
            Button::new(child("more")).debug_name("review-bar-more").icon(IconName::MoreHoriz).variant(ButtonVariant::Ghost).size(ButtonSize::Icon).disabled(entries.is_empty()).on_click(
                move |_, _, cx| {
                    toggle.update(cx, |s, cx| {
                        s.menu_open = !s.menu_open;
                        cx.notify();
                    })
                },
            ),
        )
        .child(measure({
            let state = state.clone();
            move |bounds, cx| state.update(cx, |s, _| s.menu_anchor = Some(bounds))
        }))
        .when(!entries.is_empty(), |d| {
            let (anchor, focus) = { let s = state.read(cx); (s.menu_anchor, s.focus.clone()) };
            let rows = entries.len();
            let panel = Menu::new(child("menu"), entries.into_iter().map(|(words, cap, f)| {
                let item = MenuItem::new(words).on_select(move |window, cx| f(window, cx));
                Entry::from(match cap {
                    Some(cap) => item.cap(cap),
                    None => item,
                })
            }))
            .origin(Origin::TopRight)
            .on_dismiss({
                let pick = state.clone();
                move |_, cx| {
                    pick.update(cx, |s, cx| {
                        s.menu_open = false;
                        cx.notify();
                    })
                }
            });
            d.child(
                Popover::new(child("popover"))
                    .open(open)
                    .anchor(anchor)
                    .align(Align::End)
                    .gap(4.)
                    .height(menu::height(rows))
                    .return_focus(&focus)
                    .on_close(move |_, cx| {
                        close.update(cx, |s, cx| {
                            s.menu_open = false;
                            cx.notify();
                        })
                    })
                    .child(panel),
            )
        })
}
