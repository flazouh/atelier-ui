use std::time::Instant;

use gpui_kit::{
    AnyElement, AppContext, Context, CursorStyle, DragMoveEvent, InteractiveElement,
    IntoElement, ParentElement, ScrollWheelEvent, StatefulInteractiveElement, Styled, Window,
    div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    agent_panels::{AgentPanels, MARGIN, SETTLE},
    panel_types::{DraggedEdge, PanelData, element_id},
    placement::measure,
    theme::{ActiveTheme, radius},
    };
use super::structs::Ghost;
use super::types::GROUP_HEADER;
use super::helpers::group_header;

impl AgentPanels {
    pub(crate) fn strip(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        // A glide, and then the settle after the wheel, move the offset a frame at a time.
        if let Some(glide) = &self.glide {
            self.offset = self.geometry.clamp(glide.value(), self.viewport);
            if glide.is_running() {
                window.request_animation_frame();
            } else {
                self.glide = None;
            }
        }
        if let Some(at) = self.last_wheel {
            if at.elapsed() >= SETTLE {
                self.last_wheel = None;
                let snap = self.geometry.snap(self.offset, self.viewport);
                self.glide_to(snap, cx);
            } else {
                window.request_animation_frame();
            }
        }
        let top = if self.grouped { GROUP_HEADER } else { 0. };
        let range = self.geometry.visible(self.offset, self.viewport, MARGIN);
        let mut moving = false;

        let mut children: Vec<AnyElement> = Vec::new();
        // Group headers over the columns of each group that is in view.
        if self.grouped {
            let mut first = 0;
            for group in &self.arrangement {
                let last = first + group.members.len();
                if first < range.end
                    && last > range.start
                    && let Some(project) = group.project.as_ref().and_then(|id| self.panels.iter().find(|p| p.project.id == *id))
                {
                    let (left, right) = (self.geometry.left(first), self.geometry.right(last - 1));
                    children.push(
                        div()
                            .absolute()
                            .left(px(left))
                            .top_0()
                            .w(px(right - left))
                            .child(group_header(&project.project, &theme))
                            .into_any_element(),
                    );
                }
                first = last;
            }
        }
        for column in range {
            let panel: PanelData = self.panels[self.shown[column]].clone();
            let width = self.geometry.width_of(column);
            let (x, opacity) = match self.slots.get(&panel.id) {
                Some(slot) => {
                    moving |= slot.x.is_running() || slot.appear.is_running();
                    (slot.x.value(), slot.appear.value())
                }
                None => (self.geometry.left(column), 1.),
            };
            let active = self.tabs.active() == Some(&panel.id);
            let id = panel.id.clone();
            let edge = DraggedEdge { id: id.clone() };
            let this = cx.entity();
            let click_id = id.clone();
            children.push(
                div()
                    .id(element_id("column", &id.clone()))
                    .absolute()
                    .left(px(x))
                    .top(px(top))
                    .bottom_0()
                    .w(px(width))
                    .opacity(opacity)
                    .on_mouse_down(gpui_kit::MouseButton::Left, move |_, _, cx| {
                        let click_id = click_id.clone();
                        this.update(cx, |s, cx| {
                            if s.tabs.active() != Some(&click_id) {
                                s.tabs.activate(&click_id);
                                cx.emit(crate::panel_types::PanelsEvent::Activated(click_id));
                                cx.notify();
                            }
                        })
                    })
                    .child(
                        div()
                            .size_full()
                            .rounded(radius::xl())
                            .overflow_hidden()
                            .bg(theme.card)
                            .when(active, |d| d.bg(theme.card_strong.opacity(0.5)))
                            .child(crate::panel_types::draw_content(&panel.content)),
                    )
                    .child(
                        div()
                            .id(element_id("edge", &id))
                            .absolute()
                            .right(px(-5.))
                            .top_0()
                            .bottom_0()
                            .w(px(10.))
                            .cursor(CursorStyle::ResizeLeftRight)
                            .on_drag(edge, |_, _, _, cx| cx.new(|_| Ghost)),
                    )
                    .into_any_element(),
            );
        }
        if moving {
            window.request_animation_frame();
        }

        let this = cx.entity();
        div()
            .id("panel-strip")
            .relative()
            .flex_1()
            .min_h_0()
            .overflow_hidden()
            .px(px(8.))
            .pb(px(8.))
            .on_scroll_wheel(cx.listener(|this, event: &ScrollWheelEvent, _, cx| {
                let delta = event.delta.pixel_delta(px(16.));
                let (x, y) = (f32::from(delta.x), f32::from(delta.y));
                let dx = if event.modifiers.shift { x + y } else if x.abs() > y.abs() { x } else { 0. };
                if dx != 0. {
                    this.glide = None;
                    this.offset = this.geometry.clamp(this.offset - dx, this.viewport);
                    this.last_wheel = Some(Instant::now());
                    cx.stop_propagation();
                    cx.notify();
                }
            }))
            .on_drag_move::<DraggedEdge>(cx.listener(|this, event: &DragMoveEvent<DraggedEdge>, _, cx| {
                let id = event.drag(cx).id.clone();
                let Some(column) = this.shown.iter().position(|&p| this.panels[p].id == id) else { return };
                let x = f32::from(event.event.position.x) - this.origin_x + this.offset;
                let width = x - this.geometry.left(column);
                this.resize(&id, width, cx);
            }))
            .child(measure(move |bounds, cx| {
                this.update(cx, |s, cx| {
                    let (width, origin) = (f32::from(bounds.size.width) - 16., f32::from(bounds.origin.x) + 8.);
                    if (s.viewport - width).abs() > 0.5 || (s.origin_x - origin).abs() > 0.5 {
                        s.viewport = width;
                        s.origin_x = origin;
                        // The columns fit the strip's new width.
                        s.relayout(cx);
                    }
                })
            }))
            .child(div().relative().size_full().left(px(-self.offset)).children(children))
            .children(self.edge_fade(true, &theme))
            .children(self.edge_fade(false, &theme))
            .into_any_element()
    }
}

impl AgentPanels {
    /// The soft edge of the strip, in the window's tone, where more columns lie beyond: `left` or right. None where the
    /// row ends. It follows the scroll offset, so it asks for no frames of its own.
    pub(super) fn edge_fade(&self, left: bool, theme: &crate::theme::Theme) -> Option<AnyElement> {
        let (before, after) = crate::panel_layout::edge_fades(self.offset, self.geometry.max_offset(self.viewport));
        let strength = if left { before } else { after };
        if strength <= 0. {
            return None;
        }
        let tone = theme.background;
        let (from, to) = if left { (tone, tone.opacity(0.)) } else { (tone.opacity(0.), tone) };
        let fade = div()
            .debug_selector(move || if left { "strip-fade-left".into() } else { "strip-fade-right".into() })
            .absolute()
            .top_0()
            .bottom_0()
            .w(px(crate::panel_layout::FADE))
            .opacity(strength)
            .bg(gpui_kit::linear_gradient(90., gpui_kit::linear_color_stop(from, 0.), gpui_kit::linear_color_stop(to, 1.)));
        Some(if left { fade.left_0() } else { fade.right_0() }.into_any_element())
    }
}
