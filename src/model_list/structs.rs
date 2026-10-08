use std::rc::Rc;

use gpui_kit::{
    App, AppContext, Context, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Render, RenderOnce, SharedString,
    StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    icon::{Icon, IconName},
    theme::{ActiveTheme, radius},
    tooltip::Tooltip,
    typography::TextSize,
};

use super::{
    helpers::moved,
    types::{Dragged, ModelRow},
};

type OnDefault = Rc<dyn Fn(&SharedString, &mut Window, &mut App)>;
type OnReorder = Rc<dyn Fn(Vec<SharedString>, &mut Window, &mut App)>;
type OnHide = Rc<dyn Fn(&SharedString, bool, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct ModelList {
    id: ElementId,
    key: SharedString,
    rows: Vec<ModelRow>,
    default: Option<SharedString>,
    on_default: Option<OnDefault>,
    on_reorder: Option<OnReorder>,
    on_hide: Option<OnHide>,
}

impl ModelList {
    /// `key` tells this list's rows from another's: a row dropped on a list of another key does nothing.
    pub fn new(id: impl Into<ElementId>, key: impl Into<SharedString>, rows: Vec<ModelRow>) -> Self {
        Self { id: id.into(), key: key.into(), rows, default: None, on_default: None, on_reorder: None, on_hide: None }
    }

    /// The id of the model new sessions start on.
    pub fn default(mut self, id: Option<SharedString>) -> Self {
        self.default = id;
        self
    }

    /// The reader pressed a star: the model's id.
    pub fn on_default(mut self, f: impl Fn(&SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_default = Some(Rc::new(f));
        self
    }

    /// The reader pressed an eye: the model's id and whether it is now hidden. The default has no eye to press.
    pub fn on_hide(mut self, f: impl Fn(&SharedString, bool, &mut Window, &mut App) + 'static) -> Self {
        self.on_hide = Some(Rc::new(f));
        self
    }

    /// The reader dropped a row: every id, in the new order.
    pub fn on_reorder(mut self, f: impl Fn(Vec<SharedString>, &mut Window, &mut App) + 'static) -> Self {
        self.on_reorder = Some(Rc::new(f));
        self
    }
}

/// What a row being dragged shows beside the pointer: its name.
struct Ghost(SharedString);

impl Render for Ghost {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        div()
            .px(px(10.))
            .h(px(28.))
            .flex()
            .items_center()
            .rounded(radius::md())
            .bg(theme.card_strong)
            .text_size(TextSize::Sm.font_size())
            .text_color(theme.foreground)
            .child(self.0.clone())
    }
}

impl RenderOnce for ModelList {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let order: Vec<SharedString> = self.rows.iter().map(|r| r.id.clone()).collect();
        let id = self.id.clone();
        let child = |name: String| ElementId::from((id.clone(), SharedString::from(name)));
        let rows = self.rows.iter().map(|row| {
            let is_default = self.default.as_ref() == Some(&row.id);
            let (list, row_id, label) = (self.key.clone(), row.id.clone(), row.label.clone());
            let (drop_order, drop_target, drop_list, reorder) = (order.clone(), row.id.clone(), self.key.clone(), self.on_reorder.clone());
            let (star_id, on_default) = (row.id.clone(), self.on_default.clone());
            let (eye_id, on_hide, hidden) = (row.id.clone(), self.on_hide.clone(), row.hidden);
            let eye_selector = format!("model-eye-{}-{}", self.key, row.id);
            let selector = format!("model-row-{}-{}", self.key, row.id);
            let star_selector = format!("model-star-{}-{}", self.key, row.id);
            div()
                .id(child(format!("row-{}", row.id)))
                .debug_selector(move || selector.clone())
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(32.))
                .px(px(6.))
                .rounded(radius::lg())
                .hover(|s| s.bg(theme.muted_hover()))
                .when(hidden, |d| d.opacity(0.5))
                .on_drag(Dragged { list, id: row_id, label: label.clone() }, |d, _, _, cx| cx.new(|_| Ghost(d.label.clone())))
                .drag_over::<Dragged>(|s, _, _, cx| s.bg(cx.theme().card_strong))
                .on_drop::<Dragged>(move |dragged, window, cx| {
                    if dragged.list != drop_list {
                        return;
                    }
                    if let Some(reorder) = &reorder {
                        reorder(moved(&drop_order, &dragged.id, Some(&drop_target)), window, cx);
                    }
                })
                .child(div().flex_none().cursor_grab().child(Icon::new(IconName::Grip).size(px(16.)).color(theme.muted_foreground.opacity(0.6))))
                .child(
                    div()
                        .flex_none()
                        .text_size(TextSize::Sm.font_size())
                        .font_weight(if is_default { FontWeight::MEDIUM } else { FontWeight::NORMAL })
                        .text_color(theme.foreground)
                        .child(row.label.clone()),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .truncate()
                        .text_size(TextSize::Xs.font_size())
                        .text_color(theme.muted_foreground)
                        .children(row.detail.clone()),
                )
                .child(
                    div()
                        .id(child(format!("eye-{}", row.id)))
                        .debug_selector(move || eye_selector.clone())
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(px(24.))
                        .rounded(radius::md())
                        // The default is never left out of the picker, so its eye is not there to press.
                        .when(!is_default, |d| d.cursor_pointer().hover(|s| s.bg(theme.card_strong)))
                        .when(!is_default, |d| d.tooltip(Tooltip::text(if hidden { "Show in the picker" } else { "Hide from the picker" })))
                        .when(!is_default, |d| {
                            d.when_some(on_hide, move |d, on_hide| d.on_click(move |_, window, cx| on_hide(&eye_id, !hidden, window, cx)))
                        })
                        .child(Icon::new(if hidden { IconName::VisibilityOff } else { IconName::Visibility }).size(px(16.)).color(
                            if is_default { theme.muted_foreground.opacity(0.25) } else { theme.muted_foreground },
                        )),
                )
                .child(
                    div()
                        .id(child(format!("star-{}", row.id)))
                        .debug_selector(move || star_selector.clone())
                        .flex()
                        .flex_none()
                        .items_center()
                        .justify_center()
                        .size(px(24.))
                        .rounded(radius::md())
                        .cursor_pointer()
                        .tooltip(Tooltip::text(if is_default { "The model new sessions start on" } else { "Start new sessions on this model" }))
                        .hover(|s| s.bg(theme.card_strong))
                        .when_some(on_default, move |d, on_default| d.on_click(move |_, window, cx| on_default(&star_id, window, cx)))
                        .child(if is_default {
                            Icon::new(IconName::StarFilled).size(px(16.)).color(super::helpers::star_colour(&theme))
                        } else {
                            Icon::new(IconName::Star).size(px(16.)).color(theme.muted_foreground)
                        }),
                )
        });
        // A row dropped below the last one goes to the end.
        let tail_order = order.clone();
        let (tail_list, tail_reorder) = (self.key.clone(), self.on_reorder.clone());
        div()
            .id(self.id.clone())
            .flex()
            .flex_col()
            .children(rows)
            .child(
                div()
                    .id(child("end".into()))
                    .h(px(8.))
                    .drag_over::<Dragged>(|s, _, _, cx| s.border_t_2().border_color(cx.theme().accent))
                    .on_drop::<Dragged>(move |dragged, window, cx| {
                        if dragged.list != tail_list {
                            return;
                        }
                        if let Some(reorder) = &tail_reorder {
                            reorder(moved(&tail_order, &dragged.id, None), window, cx);
                        }
                    }),
            )
    }
}
