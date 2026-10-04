//! The column of view icons at the window's left edge, as JetBrains draws its tool window bar: one icon for each
//! view, in the order work goes (the tasks, the sessions, the changes). The selected view has a wash; when its
//! sidebar is folded it keeps only a short bar at the edge, so the reader still sees where they are. Pressing an
//! icon reports it.
use std::rc::Rc;

use gpui_kit::{
    App, ElementId, InteractiveElement, IntoElement, ParentElement, RenderOnce, SharedString, StatefulInteractiveElement,
    Styled, Window, div, prelude::FluentBuilder,
};

use crate::{
    icon::{Icon, IconName},
    scale::px,
    theme::{ActiveTheme, radius},
};

/// The rail's width.
pub const WIDTH: f32 = 40.;
const BUTTON: f32 = 30.;

/// One view on the rail.
#[derive(Clone, Debug, PartialEq)]
pub struct RailView {
    pub icon: IconName,
    pub label: SharedString,
    /// The debug selector of its button, for a test to find.
    pub debug: &'static str,
    /// What waits on the reader in this view, as a count on the icon; none at 0.
    pub count: usize,
}

/// How an icon of the rail draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// Not the view in front.
    Rest,
    /// The view in front, its sidebar shown: a wash.
    Selected,
    /// The view in front, its sidebar folded: a bar at the edge.
    Folded,
}

pub fn mark(index: usize, selected: usize, open: bool) -> Mark {
    match (index == selected, open) {
        (false, _) => Mark::Rest,
        (true, true) => Mark::Selected,
        (true, false) => Mark::Folded,
    }
}

type OnSelect = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnClick = Rc<dyn Fn(&mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct ViewRail {
    id: ElementId,
    views: Vec<RailView>,
    selected: usize,
    open: bool,
    on_select: Option<OnSelect>,
}

impl ViewRail {
    /// `selected` is the view in front; `open` whether its sidebar shows.
    pub fn new(id: impl Into<ElementId>, views: Vec<RailView>, selected: usize, open: bool) -> Self {
        Self { id: id.into(), views, selected, open, on_select: None }
    }

    pub fn on_select(mut self, handler: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for ViewRail {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (selected, open) = (self.selected, self.open);
        div()
            .id(self.id)
            .debug_selector(|| "view-rail".into())
            .flex()
            .flex_col()
            .flex_none()
            .items_center()
            .gap(px(4.))
            .w(px(WIDTH))
            .h_full()
            .pt(px(8.))
            .children(self.views.into_iter().enumerate().map(|(i, view)| {
                let m = mark(i, selected, open);
                let on_select = self.on_select.clone();
                let color = if m == Mark::Rest { theme.muted_foreground } else { theme.foreground };
                div()
                    .id(("view-rail-item", i))
                    .debug_selector(move || view.debug.into())
                    .flex()
                    .items_center()
                    .justify_center()
                    .size(px(BUTTON))
                    .rounded(radius::md())
                    .cursor_pointer()
                    .when(m == Mark::Selected, |d| d.bg(theme.accent.opacity(0.14)))
                    .relative()
                    .when(m == Mark::Folded, |d| {
                        d.child(div().absolute().left(px(-5.)).top(px(8.)).bottom(px(8.)).w(px(2.)).rounded(px(1.)).bg(theme.accent))
                    })
                    .when(m == Mark::Rest, |d| d.hover(|s| s.bg(theme.muted_hover())))
                    .tooltip(crate::tooltip::Tooltip::text(view.label.clone()))
                    .when_some(on_select, |d, select| d.on_click(move |_, window, cx| select(i, window, cx)))
                    .child(Icon::new(view.icon).size(px(18.)).color(color))
                    .when(view.count > 0, |d| d.child(count_badge(view.count, &theme)))
            }))
    }
}

/// The count over an icon's top right corner, in the colour of a session that needs the reader.
fn count_badge(count: usize, theme: &crate::Theme) -> impl IntoElement {
    div()
        .debug_selector(|| "view-rail-count".into())
        .absolute()
        .top(px(-2.))
        .right(px(-2.))
        .min_w(px(14.))
        .h(px(14.))
        .px(px(3.))
        .flex()
        .items_center()
        .justify_center()
        .rounded_full()
        .bg(theme.warning)
        .text_color(theme.background)
        .text_size(px(9.))
        .font_weight(gpui_kit::FontWeight::SEMIBOLD)
        .child(count_words(count))
}

/// A button in the rail's format, for a control that sits with the rail but is no view: the sidebar's toggle.
/// It is a rail icon at rest: the same square, icon and hover wash.
#[derive(IntoElement)]
pub struct RailButton {
    id: ElementId,
    icon: IconName,
    tooltip: SharedString,
    debug: &'static str,
    on_click: Option<OnClick>,
}

impl RailButton {
    pub fn new(id: impl Into<ElementId>, icon: IconName, tooltip: impl Into<SharedString>) -> Self {
        Self { id: id.into(), icon, tooltip: tooltip.into(), debug: "rail-button", on_click: None }
    }

    /// The name tests and the control socket find it by.
    pub fn debug_name(mut self, name: &'static str) -> Self {
        self.debug = name;
        self
    }

    pub fn on_click(mut self, handler: impl Fn(&mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for RailButton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let debug = self.debug;
        div()
            .id(self.id)
            .debug_selector(move || debug.into())
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(BUTTON))
            .rounded(radius::md())
            .cursor_pointer()
            .hover(|s| s.bg(theme.muted_hover()))
            .tooltip(crate::tooltip::Tooltip::text(self.tooltip))
            .when_some(self.on_click, |d, click| d.on_click(move |_, window, cx| click(window, cx)))
            .child(Icon::new(self.icon).size(px(18.)).color(theme.muted_foreground))
    }
}

/// A count as the badge writes it: past 9 it says "9+".
pub fn count_words(count: usize) -> SharedString {
    if count > 9 { "9+".into() } else { count.to_string().into() }
}

#[cfg(test)]
mod tests;
