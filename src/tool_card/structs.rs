use std::rc::Rc;

use gpui_kit::{
    AnyElement, App, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    animated_badge::{AnimatedBadge, BadgeSize},
    badge::Badge,
    button::{Button, ButtonSize, ButtonVariant},
    icon::Icon,
    project_badge::{ProjectBadge, color_of},
    task_marks::TaskStatusMark,
    theme::{ActiveTheme, Theme, radius},
    typography::{MONO_FONT_FAMILY, TextSize},
};

use super::{
    helpers::{
        avatar_letter, badge_tone, gap_px, hidden_label, more_label, rows_shown, state_badge,
        tone_color,
    },
    types::{
        FOOTER_MAX, ToolAction, ToolCardData, ToolIcon, ToolNode, ToolProvider, ToolRow, ToolText,
        ToolTone,
    },
};

/// What a press on a row or a button gives back: the key of its action.
pub type ActionHandler = Rc<dyn Fn(&str, &mut Window, &mut App)>;

/// A tool call of an agent as a card: whose tool it is, what it did in one line, how it stands, and what it returned as
/// rows. It follows [`crate::QuestionCard`]: plain data in, and the card keeps nothing but whether a long list is open.
#[derive(IntoElement)]
pub struct ToolCard {
    id: ElementId,
    data: ToolCardData,
    on_action: Option<ActionHandler>,
}

impl ToolCard {
    /// `id` must be unique among the cards on screen: the card keeps its lists' state under it.
    pub fn new(id: impl Into<ElementId>, data: ToolCardData) -> Self {
        Self {
            id: id.into(),
            data,
            on_action: None,
        }
    }

    /// Called with the key of the action when a row or a button is pressed.
    pub fn on_action(mut self, f: impl Fn(&str, &mut Window, &mut App) + 'static) -> Self {
        self.on_action = Some(Rc::new(f));
        self
    }
}

fn child(id: &ElementId, name: impl Into<SharedString>) -> ElementId {
    ElementId::from((id.clone(), name.into()))
}

impl RenderOnce for ToolCard {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme().clone();
        let id = self.id.clone();
        let data = self.data;
        let quiet = data.state == super::types::ToolCardState::Running
            || data.state == super::types::ToolCardState::Waiting;
        let (status, label) = state_badge(data.state);

        let head = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .px(px(12.))
            .py(px(8.))
            .child(tile(&data.provider))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .debug_selector(|| "tool-card-title".into())
                            .truncate()
                            .line_height(px(20.))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(if quiet {
                                theme.muted_foreground
                            } else {
                                theme.foreground
                            })
                            .child(data.title.clone()),
                    )
                    .child(
                        div()
                            .truncate()
                            .text_size(TextSize::Xs.font_size())
                            .line_height(px(16.))
                            .text_color(theme.muted_foreground)
                            .child(provider_line(&data.provider)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .flex_none()
                    .children(data.origin.clone().map(|origin| {
                        div()
                            .debug_selector(|| "tool-card-origin".into())
                            .child(pill(origin, &theme))
                    }))
                    .child(
                        div().debug_selector(|| "tool-card-state".into()).child(
                            AnimatedBadge::new(child(&id, "state"), status)
                                .size(BadgeSize::Small)
                                .label(label),
                        ),
                    ),
            );

        let body = data.body.as_ref().map(|node| {
            div().px(px(12.)).pb(px(10.)).child(node_element(
                node,
                &id,
                "b".to_string(),
                &theme,
                &self.on_action,
                window,
                cx,
            ))
        });
        let footer = (!data.footer.is_empty()).then(|| {
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .px(px(12.))
                .pb(px(10.))
                .children(
                    data.footer
                        .iter()
                        .take(FOOTER_MAX)
                        .enumerate()
                        .map(|(n, action)| {
                            action_button(child(&id, format!("foot-{n}")), action, &self.on_action)
                        }),
                )
        });

        div()
            .debug_selector(|| "tool-card".into())
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .rounded(radius::card())
            .bg(theme.card_strong)
            .text_size(TextSize::Sm.font_size())
            .child(head)
            .children(body)
            .children(footer)
    }
}

/// A quiet pill: the words of a neutral badge on the wash of the ink, which shows on the card's own fill, where a neutral
/// [`Badge`] (filled with `card_strong`) does not.
fn pill(label: SharedString, theme: &Theme) -> impl IntoElement {
    div()
        .flex()
        .flex_none()
        .items_center()
        .h(px(20.))
        .px(px(8.))
        .rounded_full()
        .bg(theme.wash())
        .text_color(theme.muted_foreground)
        .text_size(px(11.))
        .line_height(px(18.))
        .font_weight(FontWeight::MEDIUM)
        .whitespace_nowrap()
        .child(label)
}

fn provider_line(provider: &ToolProvider) -> SharedString {
    match &provider.account {
        Some(account) if !account.is_empty() => format!("{} · {}", provider.name, account).into(),
        _ => provider.name.clone(),
    }
}

/// The provider's tile: its letter on a colour of the palette, or on the quiet fill when it is not known.
fn tile(provider: &ToolProvider) -> AnyElement {
    let mark = match provider.color {
        Some(color) => ProjectBadge::new(provider.letter.clone(), color).into_any_element(),
        None => NeutralLetter(provider.letter.clone()).into_any_element(),
    };
    div()
        .debug_selector(|| "tool-card-tile".into())
        .flex_none()
        .child(mark)
        .into_any_element()
}

/// The letter of a provider that has no colour: ink on the wash of the page's ink.
#[derive(IntoElement)]
struct NeutralLetter(SharedString);

impl RenderOnce for NeutralLetter {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .items_center()
            .justify_center()
            .size(px(crate::project_badge::SIZE))
            .rounded(px(4.))
            .bg(theme.wash())
            .text_color(theme.muted_foreground)
            .text_size(px(10.))
            .font_weight(FontWeight::SEMIBOLD)
            .line_height(px(crate::project_badge::SIZE))
            .child(self.0)
    }
}

fn action_button(id: ElementId, action: &ToolAction, on_action: &Option<ActionHandler>) -> Button {
    let (handler, key) = (on_action.clone(), action.key.clone());
    Button::new(id)
        .label(action.label.clone())
        .variant(ButtonVariant::Tinted)
        .size(ButtonSize::Sm)
        .on_click(move |_, window, cx| {
            if let Some(handler) = &handler {
                handler(&key, window, cx);
            }
        })
}

fn node_element(
    node: &ToolNode,
    id: &ElementId,
    path: String,
    theme: &Theme,
    on_action: &Option<ActionHandler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    match node {
        ToolNode::Stack { row, gap, children } => {
            let parts: Vec<AnyElement> = children
                .iter()
                .enumerate()
                .map(|(n, c)| {
                    node_element(c, id, format!("{path}-{n}"), theme, on_action, window, cx)
                })
                .collect();
            div()
                .flex()
                .min_w_0()
                .gap(px(gap_px(*gap)))
                .when(*row, |d| d.flex_row().items_center())
                .when(!*row, |d| d.flex_col())
                .children(parts)
                .into_any_element()
        }
        ToolNode::Text {
            value,
            style,
            max_lines,
        } => {
            let base = div().min_w_0().child(value.clone());
            let styled = match style {
                ToolText::Body => base
                    .text_size(TextSize::Sm.font_size())
                    .line_height(px(20.))
                    .text_color(theme.foreground),
                ToolText::Title => base
                    .text_size(TextSize::Sm.font_size())
                    .line_height(px(20.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground),
                ToolText::Muted => base
                    .text_size(TextSize::Xs.font_size())
                    .line_height(px(16.))
                    .text_color(theme.muted_foreground),
                ToolText::Code => base
                    .text_size(TextSize::Xs.font_size())
                    .line_height(px(16.))
                    .font_family(MONO_FONT_FAMILY)
                    .text_color(theme.foreground),
            };
            match max_lines {
                Some(1) => styled.truncate().into_any_element(),
                Some(n) => styled.line_clamp(usize::from(*n)).into_any_element(),
                None => styled.into_any_element(),
            }
        }
        ToolNode::Badge {
            value,
            tone: ToolTone::Neutral,
        } => pill(value.clone(), theme).into_any_element(),
        ToolNode::Badge { value, tone } => Badge::new(value.clone())
            .tone(badge_tone(*tone))
            .into_any_element(),
        ToolNode::Metric { label, value } => div()
            .flex()
            .flex_col()
            .child(
                div()
                    .text_size(TextSize::Xs.font_size())
                    .line_height(px(16.))
                    .text_color(theme.muted_foreground)
                    .child(label.clone()),
            )
            .child(
                div()
                    .line_height(px(20.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(theme.foreground)
                    .child(value.clone()),
            )
            .into_any_element(),
        ToolNode::Icon { icon, tone } => {
            let mark = match icon {
                ToolIcon::Named(name) => Icon::new(*name)
                    .size(px(14.))
                    .color(tone_color(*tone, theme.faint(), theme))
                    .into_any_element(),
                ToolIcon::Status(status) => TaskStatusMark::new(*status)
                    .size(px(14.))
                    .into_any_element(),
            };
            div()
                .flex()
                .flex_none()
                .items_center()
                .child(mark)
                .into_any_element()
        }
        ToolNode::Avatar { name } => div()
            .flex_none()
            .child(ProjectBadge::new(avatar_letter(name), color_of(None, name)))
            .into_any_element(),
        ToolNode::Code { value, max_lines } => div()
            .min_w_0()
            .px(px(8.))
            .py(px(6.))
            .rounded(radius::md())
            .bg(theme.wash())
            .text_size(TextSize::Xs.font_size())
            .line_height(px(16.))
            .font_family(MONO_FONT_FAMILY)
            .text_color(theme.foreground)
            .when_some(*max_lines, |d, n| d.line_clamp(usize::from(n)))
            .child(value.clone())
            .into_any_element(),
        ToolNode::List {
            rows,
            hidden,
            empty,
        } => list_element(rows, *hidden, empty, id, path, theme, on_action, window, cx),
        ToolNode::Button { action } => div()
            .flex()
            .child(action_button(
                child(id, format!("btn-{path}")),
                action,
                on_action,
            ))
            .into_any_element(),
        ToolNode::Divider => div().h(px(1.)).w_full().bg(theme.wash()).into_any_element(),
    }
}

#[allow(clippy::too_many_arguments)]
fn list_element(
    rows: &[ToolRow],
    hidden: usize,
    empty: &Option<SharedString>,
    id: &ElementId,
    path: String,
    theme: &Theme,
    on_action: &Option<ActionHandler>,
    window: &mut Window,
    cx: &mut App,
) -> AnyElement {
    if rows.is_empty() {
        let words = empty.clone().unwrap_or_else(|| "Nothing to show.".into());
        return div()
            .text_size(TextSize::Xs.font_size())
            .line_height(px(16.))
            .text_color(theme.muted_foreground)
            .child(words)
            .into_any_element();
    }
    let open = window.use_keyed_state(child(id, format!("open-{path}")), cx, |_, _| false);
    let expanded = *open.read(cx);
    let shown = rows_shown(rows.len(), expanded);
    let lines: Vec<AnyElement> = rows
        .iter()
        .take(shown)
        .enumerate()
        .map(|(n, row)| {
            let inner = node_element(
                &row.node,
                id,
                format!("{path}-{n}"),
                theme,
                on_action,
                window,
                cx,
            );
            let base = div()
                .id(child(id, format!("row-{path}-{n}")))
                .debug_selector(move || format!("tool-card-row-{n}"))
                .flex()
                .flex_col()
                .px(px(6.))
                .py(px(4.))
                .rounded(radius::md());
            match (&row.action, on_action) {
                (Some(key), Some(handler)) => {
                    let (key, handler) = (key.clone(), handler.clone());
                    base.cursor_pointer()
                        .hover(|s| s.bg(theme.wash()))
                        .on_click(move |_, window, cx| handler(&key, window, cx))
                        .child(inner)
                        .into_any_element()
                }
                _ => base.child(inner).into_any_element(),
            }
        })
        .collect();
    let more = more_label(rows.len(), expanded).map(|label| {
        let toggle = open.clone();
        div().flex().child(
            Button::new(child(id, format!("more-{path}")))
                .label(label)
                .variant(ButtonVariant::Ghost)
                .size(ButtonSize::Sm)
                .on_click(move |_, _, cx| {
                    toggle.update(cx, |open, cx| {
                        *open = !*open;
                        cx.notify();
                    })
                }),
        )
    });
    div()
        .flex()
        .flex_col()
        .mx(px(-6.))
        .child(div().flex().flex_col().children(lines))
        .children(more.map(|m| div().px(px(0.)).child(m)))
        .children(hidden_label(hidden).map(|words| {
            div()
                .px(px(6.))
                .pt(px(2.))
                .text_size(TextSize::Xs.font_size())
                .line_height(px(16.))
                .text_color(theme.muted_foreground)
                .child(words)
        }))
        .into_any_element()
}
