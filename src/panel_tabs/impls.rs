use gpui_kit::{
    AnyElement, AppContext, Context, FontWeight, InteractiveElement, IntoElement, MouseButton,
    ParentElement, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use super::structs::TabGhost;
use super::types::TAB_HEIGHT;
use crate::scale::px;
use crate::{
    agent_panels::AgentPanels,
    focus::PressStop,
    icon::{Icon, IconName},
    panel_types::{DraggedTab, PanelData, element_id},
    project_badge::ProjectBadge,
    session_row::agent_icon,
    tab_order::{grouped, visual_order},
    theme::{ActiveTheme, radius},
    typography::TextSize,
};

impl AgentPanels {
    fn project_of(&self, tab: &SharedString) -> SharedString {
        self.panels
            .iter()
            .find(|p| p.id == *tab)
            .map(|p| p.project.id.clone())
            .unwrap_or_default()
    }

    /// The tabs in the order they are shown, which is the order ⌃Tab follows.
    pub(crate) fn visual_tabs(&self) -> Vec<SharedString> {
        if self.grouped {
            visual_order(&grouped(
                self.tabs.order(),
                |t| self.project_of(t),
                &self.project_order,
            ))
        } else {
            self.tabs.order().to_vec()
        }
    }

    pub(super) fn tab(
        &self,
        panel: &PanelData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let theme = cx.theme().clone();
        let active = self.tabs.active() == Some(&panel.id);
        let id = panel.id.clone();
        let (select, close, drop_on) = (cx.entity(), cx.entity(), cx.entity());
        let (select_id, close_id, drop_id) = (id.clone(), id.clone(), id.clone());
        let mark = agent_icon(
            element_id("tab-mark", &id.clone()),
            &panel.look,
            &panel.status,
            &theme,
            true,
        );
        let project = panel
            .project
            .badge
            .clone()
            .map(|b| ProjectBadge::new(b.label, b.color).icon(b.icon));
        let ghost = DraggedTab {
            id: id.clone(),
            title: panel.title.clone(),
        };
        div()
            .id(element_id("tab", &id.clone()))
            .group("tab")
            .flex()
            .flex_none()
            .items_center()
            .gap(px(8.))
            .h(px(TAB_HEIGHT))
            .pl(px(10.))
            .pr(px(4.))
            .rounded(radius::md())
            .cursor_pointer()
            .text_size(TextSize::Sm.font_size())
            .when(active, |d| d.bg(theme.card_strong))
            .when(!active, |d| {
                d.hover(|s| s.bg(theme.card_strong.opacity(0.5)))
            })
            .press_stop(
                element_id("tab-focus", &id.clone()),
                radius::md(),
                window,
                cx,
            )
            .on_click(move |_, _, cx| {
                let id = select_id.clone();
                select.update(cx, |s, cx| s.activate(&id, cx))
            })
            .on_drag(ghost, |tab, _, _, cx| {
                cx.new(|_| TabGhost(tab.title.clone()))
            })
            .drag_over::<DraggedTab>(|s, _, _, cx| s.bg(cx.theme().card_strong.opacity(0.8)))
            .on_drop::<DraggedTab>(move |dragged, _, cx| {
                let (moved, target) = (dragged.id.clone(), drop_id.clone());
                drop_on.update(cx, |s, cx| {
                    s.tabs.move_before(&moved, Some(&target));
                    cx.notify();
                })
            })
            .child(mark)
            .children(project)
            .child(
                div()
                    .max_w(px(200.))
                    .truncate()
                    .text_color(if panel.status.title_is_ink() || active {
                        theme.foreground
                    } else {
                        theme.muted_foreground
                    })
                    .child(panel.title.clone()),
            )
            .child(
                div()
                    .id(element_id("tab-close", &id))
                    .flex()
                    .flex_none()
                    // As big as ButtonSize::IconSm, so the hit area is 24.
                    .size(px(24.))
                    .items_center()
                    .justify_center()
                    .rounded(radius::md())
                    .text_color(theme.muted_foreground)
                    .hover(|s| s.bg(theme.card).text_color(theme.foreground))
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .press_stop(element_id("tab-close-focus", &id), radius::md(), window, cx)
                    .on_click(move |_, _, cx| {
                        cx.stop_propagation();
                        let id = close_id.clone();
                        close.update(cx, |s, cx| s.close(&id, cx))
                    })
                    .child(Icon::new(IconName::Close).size(px(14.))),
            )
            .into_any_element()
    }

    /// Scrolls the tab bar the least that shows tab number `at` whole. False while the bar has not been laid
    /// out yet, so the caller asks again next frame.
    fn reveal_in_tab_bar(&self, at: usize) -> bool {
        let (bar, offset) = (self.tab_scroll.bounds(), self.tab_scroll.offset());
        let Some(tab) = self.tab_scroll.bounds_for_item(at) else {
            return false;
        };
        if f32::from(bar.size.width) < 1. {
            return false;
        }
        let (left, right) = (
            f32::from(tab.left() - bar.left()),
            f32::from(tab.right() - bar.left()),
        );
        let width = f32::from(bar.size.width);
        let pad = 8.;
        let shift = if left < pad {
            pad - left
        } else if right > width - pad {
            (width - pad) - right
        } else {
            0.
        };
        if shift != 0. {
            self.tab_scroll
                .set_offset(gpui_kit::point(offset.x + px(shift), offset.y));
        }
        true
    }

    /// The tabs and the project names between them, scrolled to the active tab once the bar is laid out.
    fn bar_items(&mut self, window: &mut Window, cx: &mut Context<Self>) -> Vec<AnyElement> {
        let theme = cx.theme().clone();
        let groups = if self.grouped {
            grouped(
                self.tabs.order(),
                |t| self.project_of(t),
                &self.project_order,
            )
        } else {
            Vec::new()
        };
        let mut bar: Vec<AnyElement> = Vec::new();
        let mut active_at: Option<usize> = None;
        let sequence: Vec<(Option<SharedString>, Vec<SharedString>)> = if self.grouped {
            groups
                .iter()
                .map(|g| (Some(g.project.clone()), g.tabs.clone()))
                .collect()
        } else {
            vec![(None, self.tabs.order().to_vec())]
        };
        for (n, (project, tabs)) in sequence.iter().enumerate() {
            if let Some(project) = project {
                let name = self
                    .panels
                    .iter()
                    .find(|p| p.project.id == *project)
                    .map(|p| p.project.name.clone())
                    .unwrap_or_default();
                bar.push(
                    div()
                        .flex()
                        .flex_none()
                        .items_center()
                        .h(px(TAB_HEIGHT))
                        .when(n > 0, |d| d.ml(px(12.)))
                        .mr(px(4.))
                        .text_size(TextSize::Xs.font_size())
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(theme.muted_foreground)
                        .child(name)
                        .into_any_element(),
                );
            }
            for id in tabs {
                if let Some(panel) = self.panels.iter().find(|p| p.id == *id).cloned() {
                    if self.tabs.active() == Some(id) {
                        active_at = Some(bar.len());
                    }
                    bar.push(self.tab(&panel, window, cx));
                }
            }
        }
        if self.reveal_tab
            && let Some(at) = active_at
            && self.reveal_in_tab_bar(at)
        {
            self.reveal_tab = false;
        }
        if self.reveal_tab {
            window.request_animation_frame();
        }
        bar
    }

    /// The tab bar on its own, as long as its owner gives it: it scrolls sideways when the tabs overflow. The owner
    /// sets the height and the margins, and this fills them.
    pub fn tab_strip(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let bar = self.bar_items(window, cx);
        div()
            .id("tab-bar")
            .flex()
            .size_full()
            .min_w_0()
            .items_center()
            .gap(px(2.))
            .overflow_x_scroll()
            .track_scroll(&self.tab_scroll)
            .children(bar)
            .into_any_element()
    }

    pub(crate) fn single(&mut self, window: &mut Window, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let bar = (!self.tabs_hoisted).then(|| self.tab_strip(window, cx));
        let content = self
            .tabs
            .active()
            .and_then(|id| self.panels.iter().find(|p| p.id == *id))
            .map(|panel| crate::panel_types::draw_content(&panel.content));
        // The same room round the panes as the strip leaves: its inset at the left, 8 at the right and the foot.
        div()
            .flex_1()
            .min_h_0()
            .flex()
            .flex_col()
            .pl(px(self.inset_left))
            .pr(px(8.))
            .pb(px(self.inset_bottom))
            .children(bar.map(|bar| div().flex().flex_none().h(px(TAB_HEIGHT + 4.)).child(bar)))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .rounded(radius::xl())
                    .overflow_hidden()
                    .bg(theme.card)
                    .children(content),
            )
            .into_any_element()
    }
}
