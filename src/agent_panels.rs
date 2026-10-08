//! Several agent panels at once: side by side in a row that scrolls, or one at a time under tabs, grouped
//! by project or not. This holds the state and the commands; [`crate::panel_strip`] draws the row and
//! [`crate::panel_tabs`] the tabs, and [`crate::panel_layout`] and [`crate::tab_order`] decide the numbers
//! and the order without a window.
//!
//! Keys (GPUI actions, with a modifier, so they never type into a panel's input):
//! next and previous panel, next and previous tab, close tab, switch layout, switch grouping. Their caps
//! come from [`chord`].
use std::{collections::HashMap, time::Instant};

use gpui_kit::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    KeyBinding, ParentElement, Render, ScrollHandle, SharedString, Styled, Window, div,
};

use crate::{
    motion::{Channel, Curve, Spring},
    panel_layout::{self, DEFAULT_WIDTH, Geometry, Group},
    panel_types::{Layout, PanelData, PanelsEvent, PanelsState},
    tab_order::TabOrder,
    theme::ActiveTheme,
};

gpui_kit::actions!(
    agent_panels,
    [
        /// Focuses the next panel and scrolls it into view.
        NextPanel,
        /// Focuses the previous panel and scrolls it into view.
        PreviousPanel,
        /// The next tab, in the order the tabs are shown.
        NextTab,
        /// The previous tab.
        PreviousTab,
        /// Closes the active tab.
        CloseTab,
        /// Side by side, and single view.
        ToggleLayout,
        /// Panels grouped by project, and in one row.
        ToggleGrouping,
    ]
);

/// The chords for the panel commands, in the notation of [`crate::keys::cap`]. A button's cap comes
/// from the same string, so the letter on a control and the letter that works are the same.
pub mod chord {
    pub const NEXT_PANEL: &str = "⌘]";
    pub const PREVIOUS_PANEL: &str = "⌘[";
    pub const NEXT_TAB: &str = "⌃⇥";
    pub const PREVIOUS_TAB: &str = "⌃⇧⇥";
    pub const CLOSE_TAB: &str = "⌘w";
    pub const TOGGLE_LAYOUT: &str = "⌘\\";
    pub const TOGGLE_GROUPING: &str = "⌘⇧g";
}

pub(crate) fn bind_keys(cx: &mut App) {
    let context = Some("AgentPanels");
    cx.bind_keys([
        KeyBinding::new("secondary-]", NextPanel, context),
        KeyBinding::new("secondary-[", PreviousPanel, context),
        KeyBinding::new("ctrl-tab", NextTab, context),
        KeyBinding::new("ctrl-shift-tab", PreviousTab, context),
        KeyBinding::new("secondary-w", CloseTab, context),
        KeyBinding::new("secondary-\\", ToggleLayout, context),
        KeyBinding::new("secondary-shift-g", ToggleGrouping, context),
    ]);
}

/// A column's place on screen, and how it came there: a panel slides to a new place with the Layout
/// spring and fades in when it opens.
pub(crate) struct Slot {
    pub x: Channel,
    pub appear: Channel,
    /// Opened after the first paint: it starts to the right of its place and fades in.
    pub slides_in: bool,
    /// Its place has been set once.
    pub placed: bool,
}

/// Columns are laid out this far past the viewport on each side, so a scroll never shows an empty edge.
pub(crate) const MARGIN: f32 = 240.;
/// A scroll settles this long after the last wheel event.
pub(crate) const SETTLE: std::time::Duration = std::time::Duration::from_millis(140);
/// How far a new column starts to the right of its place.
const SLIDE_IN: f32 = 80.;

pub struct AgentPanels {
    pub(crate) panels: Vec<PanelData>,
    pub(crate) project_order: Vec<SharedString>,
    pub(crate) layout: Layout,
    pub(crate) grouped: bool,
    pub(crate) widths: HashMap<SharedString, f32>,
    pub(crate) tabs: TabOrder,
    pub(crate) arrangement: Vec<Group>,
    pub(crate) geometry: Geometry,
    /// The panels in the order the strip shows them: indices into `panels`.
    pub(crate) shown: Vec<usize>,
    pub(crate) slots: HashMap<SharedString, Slot>,
    pub(crate) offset: f32,
    pub(crate) viewport: f32,
    /// The strip's padding at its left, 8 unless the strip sits against another card.
    pub(crate) inset_left: f32,
    /// Its padding at the foot, 8 unless a card stands right under it.
    pub(crate) inset_bottom: f32,
    /// The bar's width at the last layout, which picks its words.
    pub(crate) origin_x: f32,
    pub(crate) glide: Option<Channel>,
    pub(crate) last_wheel: Option<Instant>,
    pub(crate) focus: FocusHandle,
    pub(crate) tab_scroll: ScrollHandle,
    /// The tab bar scrolls the active tab into view at the next paint.
    pub(crate) reveal_tab: bool,
    /// The owner draws the tab bar itself (in its title bar, with [`crate::panel_tabs::TabStrip`]), so the single
    /// view leaves its own bar out.
    pub(crate) tabs_hoisted: bool,
}

impl EventEmitter<PanelsEvent> for AgentPanels {}

impl Focusable for AgentPanels {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl AgentPanels {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            panels: Vec::new(),
            project_order: Vec::new(),
            layout: Layout::default(),
            grouped: true,
            widths: HashMap::new(),
            tabs: TabOrder::new(),
            arrangement: Vec::new(),
            geometry: Geometry::default(),
            shown: Vec::new(),
            slots: HashMap::new(),
            offset: 0.,
            viewport: 0.,
            inset_left: 8.,
            inset_bottom: 8.,
            origin_x: 0.,
            glide: None,
            last_wheel: None,
            focus: cx.focus_handle(),
            tab_scroll: ScrollHandle::new(),
            reveal_tab: false,
            tabs_hoisted: false,
        }
    }

    /// Whether the owner draws the single view's tab bar elsewhere. Only a change redraws.
    pub fn set_tabs_hoisted(&mut self, hoisted: bool, cx: &mut Context<Self>) {
        if self.tabs_hoisted != hoisted {
            self.tabs_hoisted = hoisted;
            cx.notify();
        }
    }

    pub fn tabs_hoisted(&self) -> bool {
        self.tabs_hoisted
    }

    pub fn state(&self) -> PanelsState {
        PanelsState {
            layout: self.layout,
            grouped: self.grouped,
            widths: self.widths.iter().map(|(id, w)| (id.clone(), *w)).collect(),
        }
    }

    pub fn restore(&mut self, state: PanelsState, cx: &mut Context<Self>) {
        self.layout = state.layout;
        self.grouped = state.grouped;
        self.widths = state.widths.into_iter().collect();
        self.relayout(cx);
    }

    /// Scrolls the strip to `offset`, in pixels from its start, with no glide. For measuring runs.
    pub fn scroll_to(&mut self, offset: f32, cx: &mut Context<Self>) {
        self.glide = None;
        self.offset = self.geometry.clamp(offset, self.viewport);
        cx.notify();
    }

    pub fn layout(&self) -> Layout {
        self.layout
    }

    pub fn grouped(&self) -> bool {
        self.grouped
    }

    pub fn active(&self) -> Option<&SharedString> {
        self.tabs.active()
    }

    /// The open panels, in the sidebar's order of projects (`project_order`). A panel that is new opens at
    /// the end and becomes the active one; one that is missing is closed.
    pub fn set_panels(
        &mut self,
        panels: Vec<PanelData>,
        project_order: Vec<SharedString>,
        cx: &mut Context<Self>,
    ) {
        let reduce = cx.reduce_motion();
        let known: Vec<SharedString> = self.panels.iter().map(|p| p.id.clone()).collect();
        for panel in &panels {
            if !known.contains(&panel.id) {
                self.tabs.open(panel.id.clone());
                // A panel that opens later than the first paint slides in; the first ones just are.
                let slides_in = !known.is_empty();
                let mut appear = Channel::new(if slides_in { 0. } else { 1. });
                if slides_in {
                    appear.animate(1., Curve::Ease(0.2, crate::motion::ease::OUT), 0., reduce);
                }
                self.slots.insert(
                    panel.id.clone(),
                    Slot {
                        x: Channel::new(0.),
                        appear,
                        slides_in,
                        placed: false,
                    },
                );
            }
        }
        let open: Vec<SharedString> = panels.iter().map(|p| p.id.clone()).collect();
        for id in known.iter().filter(|id| !open.contains(id)) {
            self.tabs.close(id);
            self.slots.remove(id);
        }
        self.panels = panels;
        self.project_order = project_order;
        self.reveal_tab = true;
        self.relayout(cx);
    }

    /// The strip's padding at its left. Beside a sidebar card it is the panels' own gap, so the cards stand evenly apart.
    pub fn set_inset_left(&mut self, inset: f32, cx: &mut Context<Self>) {
        if (self.inset_left - inset).abs() > 0.1 {
            self.inset_left = inset;
            cx.notify();
        }
    }

    /// The strip's padding at its foot. With a card right under it, it is the panels' own gap.
    pub fn set_inset_bottom(&mut self, inset: f32, cx: &mut Context<Self>) {
        if (self.inset_bottom - inset).abs() > 0.1 {
            self.inset_bottom = inset;
            cx.notify();
        }
    }

    /// The strip's width in this frame, from the owner that lays the column out, so the columns fit
    /// it in this frame rather than after the strip measures itself one frame later.
    pub fn fit_to(&mut self, viewport: f32, cx: &mut Context<Self>) {
        if (self.viewport - viewport).abs() > 0.5 {
            self.viewport = viewport;
            self.relayout(cx);
        }
    }
    pub(crate) fn width_of(&self, panel: &PanelData) -> f32 {
        self.widths.get(&panel.id).copied().unwrap_or(DEFAULT_WIDTH)
    }

    /// The geometry again, after panels, widths or the grouping changed. Columns glide to their places.
    pub(crate) fn relayout(&mut self, cx: &mut Context<Self>) {
        let reduce = cx.reduce_motion();
        let projects: Vec<SharedString> =
            self.panels.iter().map(|p| p.project.id.clone()).collect();
        self.arrangement = panel_layout::arrange(&projects, &self.project_order, self.grouped);
        self.shown = panel_layout::flat(&self.arrangement);
        let widths: Vec<f32> = self
            .panels
            .iter()
            .map(|p| panel_layout::fitted(self.width_of(p), self.viewport))
            .collect();
        self.geometry = Geometry::new(panel_layout::columns(&self.arrangement, |panel| {
            widths[panel]
        }));
        for (column, &panel) in self.shown.iter().enumerate() {
            let target = self.geometry.left(column);
            let id = self.panels[panel].id.clone();
            if let Some(slot) = self.slots.get_mut(&id) {
                if !slot.placed {
                    slot.placed = true;
                    slot.x = Channel::new(if slot.slides_in {
                        target + SLIDE_IN
                    } else {
                        target
                    });
                }
                slot.x
                    .animate(target, Curve::Spring(Spring::LAYOUT), 0., reduce);
            }
        }
        self.offset = self.geometry.clamp(self.offset, self.viewport);
        cx.notify();
    }

    pub fn set_layout(&mut self, layout: Layout, cx: &mut Context<Self>) {
        if self.layout != layout {
            self.layout = layout;
            self.reveal_tab = true;
            cx.emit(PanelsEvent::StateChanged);
            cx.notify();
        }
    }

    pub fn set_grouped(&mut self, grouped: bool, cx: &mut Context<Self>) {
        if self.grouped != grouped {
            self.grouped = grouped;
            self.relayout(cx);
            cx.emit(PanelsEvent::StateChanged);
        }
    }

    pub fn toggle_layout(&mut self, cx: &mut Context<Self>) {
        self.set_layout(
            if self.layout == Layout::SideBySide {
                Layout::Single
            } else {
                Layout::SideBySide
            },
            cx,
        );
    }

    /// Makes a panel the active one, and brings it into view in the strip.
    pub fn activate(&mut self, id: &str, cx: &mut Context<Self>) {
        self.tabs.activate(id);
        if let Some(id) = self.tabs.active().cloned() {
            cx.emit(PanelsEvent::Activated(id));
        }
        self.reveal_active(cx);
        self.reveal_tab = true;
        cx.notify();
    }

    pub(crate) fn reveal_active(&mut self, cx: &mut Context<Self>) {
        let Some(active) = self.tabs.active() else {
            return;
        };
        let Some(column) = self
            .shown
            .iter()
            .position(|&p| self.panels[p].id == *active)
        else {
            return;
        };
        let target = self.geometry.reveal(self.offset, self.viewport, column);
        self.glide_to(target, cx);
    }

    /// Scrolls the strip to `target` with the Layout spring.
    pub(crate) fn glide_to(&mut self, target: f32, cx: &mut Context<Self>) {
        if (target - self.offset).abs() < 0.5 {
            return;
        }
        let mut glide = Channel::new(self.offset);
        glide.animate(
            target,
            Curve::Spring(Spring::LAYOUT),
            0.,
            cx.reduce_motion(),
        );
        self.glide = Some(glide);
        cx.notify();
    }

    pub fn close(&mut self, id: &str, cx: &mut Context<Self>) {
        if self.tabs.contains(id) {
            cx.emit(PanelsEvent::Closed(id.to_string().into()));
        }
    }

    fn move_focus(&mut self, forward: bool, cx: &mut Context<Self>) {
        let order: Vec<SharedString> = if self.layout == Layout::SideBySide {
            self.shown
                .iter()
                .map(|&p| self.panels[p].id.clone())
                .collect()
        } else {
            self.visual_tabs()
        };
        if let Some(next) = self.tabs.cycle(&order, forward) {
            self.activate(&next, cx);
        }
    }

    pub(crate) fn resize(&mut self, id: &SharedString, width: f32, cx: &mut Context<Self>) {
        let width = width.clamp(panel_layout::MIN_WIDTH, panel_layout::MAX_WIDTH);
        if self.widths.get(id).copied().unwrap_or(DEFAULT_WIDTH) != width {
            self.widths.insert(id.clone(), width);
            self.relayout(cx);
        }
    }
}

impl Render for AgentPanels {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let body = match self.layout {
            Layout::SideBySide => self.strip(window, cx),
            Layout::Single => self.single(window, cx),
        };
        div()
            .id("agent-panels")
            .key_context("AgentPanels")
            .track_focus(&self.focus)
            // A press in a panel gives the panels the keys, unless it landed in something that took the
            // focus itself, such as a composer: that keeps it, so typing goes where the reader pressed.
            .on_mouse_down(
                gpui_kit::MouseButton::Left,
                cx.listener(|this, _, window, cx| {
                    if !this.focus.contains_focused(window, cx) {
                        window.focus(&this.focus, cx)
                    }
                }),
            )
            .on_action(cx.listener(|this, _: &NextPanel, _, cx| this.move_focus(true, cx)))
            .on_action(cx.listener(|this, _: &PreviousPanel, _, cx| this.move_focus(false, cx)))
            .on_action(cx.listener(|this, _: &NextTab, _, cx| this.move_focus(true, cx)))
            .on_action(cx.listener(|this, _: &PreviousTab, _, cx| this.move_focus(false, cx)))
            .on_action(cx.listener(|this, _: &CloseTab, _, cx| {
                if let Some(id) = this.tabs.active().cloned() {
                    this.close(&id, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &ToggleLayout, _, cx| this.toggle_layout(cx)))
            .on_action(
                cx.listener(|this, _: &ToggleGrouping, _, cx| this.set_grouped(!this.grouped, cx)),
            )
            .size_full()
            .flex()
            .flex_col()
            .bg(theme.background)
            .child(body)
    }
}

#[cfg(test)]
mod tests;
