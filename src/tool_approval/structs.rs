use std::{rc::Rc, time::UNIX_EPOCH};

use gpui_kit::{
    App, ClickEvent, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement,
    RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window, div,
    prelude::FluentBuilder,
};

use crate::scale::px;
use crate::{
    ClickHandler,
    animated_badge::{AnimatedBadge, BadgeSize},
    button::{Button, ButtonSize, ButtonVariant},
    focus::PressStop,
    icon::{Icon, IconName},
    motion::{Channel, duration},
    reveal::Reveal,
    theme::{ActiveTheme, radius},
    tool_preview::ToolPreview,
    typography::{MONO_FONT_FAMILY, TextSize},
};
use super::types::{ParamValue, ToolApprovalStatus};
use super::helpers::{child, follow_status, head_words, param_value, preview_view, wire};

#[derive(IntoElement)]
pub struct ToolApproval {
    pub(super) id: ElementId,
    pub(super) tool: SharedString,
    pub(super) title: SharedString,
    pub(super) description: Option<SharedString>,
    pub(super) parameters: Vec<(SharedString, ParamValue)>,
    pub(super) preview: Option<ToolPreview>,
    pub(super) status: ToolApprovalStatus,
    pub(super) default_open: bool,
    pub(super) on_approve: Option<ClickHandler>,
    pub(super) on_always_allow: Option<ClickHandler>,
    pub(super) on_deny: Option<ClickHandler>,
}

impl ToolApproval {
    /// `id` must be unique among the requests on screen; `tool` names the call, such as "terminal.run".
    pub fn new(id: impl Into<ElementId>, tool: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            tool: tool.into(),
            title: "Allow this tool to run?".into(),
            description: None,
            parameters: Vec::new(),
            preview: None,
            status: ToolApprovalStatus::Pending,
            default_open: false,
            on_approve: None,
            on_always_allow: None,
            on_deny: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = title.into();
        self
    }

    /// Why the agent asks, in its words.
    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// A labeled argument, such as ("Directory", "~/src/atelier").
    pub fn parameter(mut self, label: impl Into<SharedString>, value: impl Into<SharedString>) -> Self {
        self.parameters.push((label.into(), ParamValue::Text(value.into())));
        self
    }

    /// A parameter whose value is code, shown in a mono chip like beui's `ToolApprovalCode`.
    pub fn parameter_code(mut self, label: impl Into<SharedString>, code: impl Into<SharedString>) -> Self {
        self.parameters.push((label.into(), ParamValue::Code(code.into())));
        self
    }

    /// What the call will do, in the reader's terms: a diff of the edit under its path, or the command. It shows while
    /// the request waits for an answer, and the parameters (the raw input) stay behind "View details".
    pub fn preview(mut self, preview: ToolPreview) -> Self {
        self.preview = Some(preview);
        self
    }

    pub fn status(mut self, status: ToolApprovalStatus) -> Self {
        self.status = status;
        self
    }

    /// Opens the details on first render, for demos and screenshots.
    pub fn default_open(mut self, open: bool) -> Self {
        self.default_open = open;
        self
    }

    pub fn on_approve(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_approve = Some(Rc::new(f));
        self
    }

    /// Shows the "Always allow" button when set; beui hides it when this handler is absent.
    pub fn on_always_allow(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_always_allow = Some(Rc::new(f));
        self
    }

    pub fn on_deny(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_deny = Some(Rc::new(f));
        self
    }
}

pub(super) struct ApprovalMotion {
    pub(super) status: ToolApprovalStatus,
    pub(super) disclosure: Reveal,
    pub(super) actions: Channel,
}

impl RenderOnce for ToolApproval {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let reduce = cx.reduce_motion();
        let status = self.status;
        let pending = status == ToolApprovalStatus::Pending;
        let has_params = !self.parameters.is_empty();
        let default_open = self.default_open;
        let id = self.id.clone();
        let motion = window.use_keyed_state(id.clone(), cx, move |_, _| ApprovalMotion {
            status,
            disclosure: Reveal::new(default_open),
            actions: Channel::new(if pending { 1. } else { 0. }),
        });
        follow_status(&motion, status, reduce, cx);
        let m = motion.read(cx);
        let moving = m.disclosure.is_moving() || m.actions.is_running();
        if moving || (status.busy() && !reduce) {
            window.request_animation_frame();
        }
        let (reveal, chevron, actions_reveal) =
            (m.disclosure.reveal.value(), m.disclosure.chevron.value(), m.actions.value());

        let theme = cx.theme().clone();
        let muted = theme.muted_foreground;
        // lucide's `animate-spin`.
        let spin_ms = duration::SPIN.as_millis();
        let spin = if status.busy() && !reduce {
            (UNIX_EPOCH.elapsed().unwrap_or_default().as_millis() % spin_ms) as f32 / spin_ms as f32
        } else {
            0.
        };
        let tile_color = if status == ToolApprovalStatus::Error { theme.danger } else { muted };

        let (title, tool_line, description) = head_words(
            &self.title,
            &self.tool,
            self.description.as_deref(),
            self.preview.as_ref().and_then(|p| p.path()).map(|p| p.as_ref()),
            self.preview.as_ref().and_then(|p| p.command_text()).map(|c| c.as_ref()),
        );
        let head = div()
            .flex()
            .items_start()
            .gap(px(12.))
            .p(px(16.))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .mt(px(2.))
                    .size(px(32.))
                    .rounded(radius::xl())
                    .bg(theme.card_strong)
                    .text_color(tile_color)
                    .child(Icon::new(status.icon()).size(px(16.)).color(tile_color).turn(spin)),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(
                        div()
                            .flex()
                            .items_start()
                            .justify_between()
                            .gap(px(12.))
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .min_w_0()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(theme.foreground)
                                            .child(SharedString::from(title)),
                                    )
                                    .children(tool_line.map(|tool| {
                                        div()
                                            .debug_selector(|| "approval-tool-line".into())
                                            .mt(px(2.))
                                            .truncate()
                                            .font_family(MONO_FONT_FAMILY)
                                            .text_size(TextSize::Xs.font_size())
                                            .text_color(muted)
                                            .child(SharedString::from(tool))
                                    })),
                            )
                            .child(AnimatedBadge::new(child(&id, "status"), status.badge()).size(BadgeSize::Small).show_icon(false).label(status.label()).debug_name("approval-status")),
                    )
                    .when_some(description, |d, description| {
                        d.child(div().debug_selector(|| "approval-description".into()).mt(px(8.)).line_height(px(20.)).text_color(muted).child(SharedString::from(description)))
                    })
                    .when(has_params, |d| {
                        let toggle = motion.clone();
                        d.child(
                            div()
                                .id(child(&id, "toggle"))
                                .flex()
                                .items_center()
                                .gap(px(4.))
                                .mt(px(8.))
                                .cursor_pointer()
                                .text_size(TextSize::Xs.font_size())
                                .font_weight(FontWeight::MEDIUM)
                                .text_color(muted)
                                .hover(|s| s.text_color(theme.foreground))
                                .press_stop(child(&id, "details-focus"), crate::theme::radius::md(), window, cx)
                                .on_click(move |_, _, cx| {
                                    let reduce = cx.reduce_motion();
                                    toggle.update(cx, |m, cx| {
                                        let open = !m.disclosure.open;
                                        m.disclosure.set_open(open, reduce);
                                        cx.notify();
                                    })
                                })
                                .child("View details")
                                .child(Icon::new(IconName::ChevronDown).size(px(14.)).turn(chevron / 360.)),
                        )
                    }),
            );

        let preview = self.preview.as_ref().filter(|_| pending).map(|p| preview_view(&id, p, &theme));
        let details = has_params.then(|| {
            div()
                .flex()
                .flex_col()
                .gap(px(8.))
                .mx(px(16.))
                .mb(px(16.))
                .p(px(12.))
                .rounded(radius::xl())
                .bg(theme.card_strong)
                .text_size(TextSize::Xs.font_size())
                .children(self.parameters.into_iter().map(|(label, value)| {
                    div()
                        .flex()
                        .gap(px(12.))
                        .child(div().w(px(112.)).flex_none().text_color(muted).child(label))
                        .child(param_value(value, &theme))
                }))
        });

        let actions = (actions_reveal > 0.001).then(|| {
            div()
                .relative()
                .top(px(4. * (1. - actions_reveal)))
                .opacity(actions_reveal)
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.))
                        .px(px(16.))
                        .py(px(12.))
                        .child(wire(
                            Button::new(child(&id, "approve"))
                                .label("Allow once")
                                .variant(ButtonVariant::Invert)
                                .size(ButtonSize::Sm),
                            self.on_approve,
                        ))
                        .when_some(self.on_always_allow, |d, handler| {
                            d.child(wire(
                                Button::new(child(&id, "always"))
                                    .label("Always allow")
                                    .variant(ButtonVariant::Secondary)
                                    .size(ButtonSize::Sm),
                                Some(handler),
                            ))
                        })
                        .child(wire(
                            Button::new(child(&id, "deny")).label("Deny").variant(ButtonVariant::Ghost).size(ButtonSize::Sm),
                            self.on_deny,
                        )),
                )
        });

        div()
            .flex()
            .flex_col()
            .w_full()
            .overflow_hidden()
            .rounded(radius::xxl())
            .bg(theme.card)
            .text_size(TextSize::Sm.font_size())
            .child(head)
            .children(preview)
            .when_some(details.filter(|_| reveal > 0.001), |d, body| {
                d.child(div().relative().top(px(-4. * (1. - reveal))).opacity(reveal).child(body))
            })
            .children(actions)
    }
}
