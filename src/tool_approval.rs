//! beui's ToolApproval (`components/agents/tool-approval.tsx`), class for class:
//!
//! - Row `items-start gap-3 p-4`: a `size-8` status tile (`ShieldCheck`, a spinning `LoaderCircle`,
//!   `Check`, `X`, or `CircleAlert`), the title `font-medium`, the tool in mono `text-xs` at muted, and a
//!   status pill (`rounded-full px-2 py-0.5 text-[11px]`) tinted amber, blue, emerald, or rose by
//!   `ToolApprovalStatus`.
//! - An optional description at `leading-5` muted, and a "View details" toggle (`text-xs`, a `size-3.5`
//!   chevron on `SPRING_SWAP`) shown only when there are parameters.
//! - Details disclose like `AgentDisclosure`: a `rounded-xl` card of label/value rows, label column
//!   `7rem`, a parameter's value in mono, or in a mono chip for `ParamValue::Code` (beui's
//!   `ToolApprovalCode`).
//! - The action row (`Allow once`, an optional `Always allow`, `Deny`) shows only while `Pending`,
//!   fading in and rising 4px over 220ms (120ms reduced), fading out the same way, and the details close
//!   themselves on leaving `Pending`.
//! - Borderless: every `border` token becomes a `card`/`card_strong` fill.

use std::{rc::Rc, sync::Arc, time::UNIX_EPOCH};

use gpui_kit::{
    App, ClickEvent, ElementId, Entity, FontWeight, IntoElement, InteractiveElement, ParentElement, RenderOnce,
    SharedString, StatefulInteractiveElement, Styled, Window, div, prelude::FluentBuilder, 
};
use crate::scale::px;

use crate::{
    focus::PressStop,
    ClickHandler,
    animated_badge::{AnimatedBadge, BadgeSize, BadgeStatus},
    button::{Button, ButtonSize, ButtonVariant},
    icon::{Icon, IconName},
    motion::{Channel, Curve, duration, ease},
    file_diff::{FileDiff, FileDiffStatus},
    reveal::Reveal,
    theme::{ActiveTheme, Theme, radius},
    tool_preview::ToolPreview,
    typography::{MONO_FONT_FAMILY, TextSize},
};

/// beui's `ToolApprovalStatus`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolApprovalStatus {
    Pending,
    Approving,
    Approved,
    Denied,
    Running,
    Complete,
    Error,
}

impl ToolApprovalStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Pending => "Approval required",
            Self::Approving => "Approving",
            Self::Approved => "Approved",
            Self::Denied => "Denied",
            Self::Running => "Running",
            Self::Complete => "Completed",
            Self::Error => "Failed",
        }
    }

    fn busy(self) -> bool {
        matches!(self, Self::Approving | Self::Running)
    }

    fn icon(self) -> IconName {
        if self.busy() {
            return IconName::Progress;
        }
        match self {
            Self::Error => IconName::Error,
            Self::Denied => IconName::Close,
            Self::Approved | Self::Complete => IconName::Check,
            _ => IconName::VerifiedUser,
        }
    }

    /// The badge's status: amber, a turning ring while it works, emerald and rose.
    fn badge(self) -> BadgeStatus {
        match self {
            Self::Pending => BadgeStatus::Warning,
            Self::Approving | Self::Running => BadgeStatus::Loading,
            Self::Approved | Self::Complete => BadgeStatus::Success,
            Self::Denied | Self::Error => BadgeStatus::Danger,
        }
    }
}

/// A parameter's value: plain mono text, or a mono chip like beui's `ToolApprovalCode`.
#[derive(Clone, Debug)]
pub enum ParamValue {
    Text(SharedString),
    Code(SharedString),
}

#[derive(IntoElement)]
pub struct ToolApproval {
    id: ElementId,
    tool: SharedString,
    title: SharedString,
    description: Option<SharedString>,
    parameters: Vec<(SharedString, ParamValue)>,
    preview: Option<ToolPreview>,
    status: ToolApprovalStatus,
    default_open: bool,
    on_approve: Option<ClickHandler>,
    on_always_allow: Option<ClickHandler>,
    on_deny: Option<ClickHandler>,
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

fn child(id: &ElementId, name: &'static str) -> ElementId {
    ElementId::NamedChild(Arc::new(id.clone()), name.into())
}

fn wire(button: Button, handler: Option<ClickHandler>) -> Button {
    match handler {
        Some(h) => button.on_click(move |e, w, cx| h(e, w, cx)),
        None => button,
    }
}

/// The words in the head of the card: the title, the tool's own name line, and the description. A preview that
/// names a file makes the head one line, the tool's word and the file ("Edit NOTES.md"), because the diff below
/// repeats the path; the tool line goes when it says what the title says, and so does a description that only
/// gives the path. A command preview shows the command whole, so a description that only repeats the start of it
/// goes. With no preview, the path line stays.
pub fn head_words(title: &str, tool: &str, description: Option<&str>, preview_path: Option<&str>, preview_command: Option<&str>) -> (String, Option<String>, Option<String>) {
    let mut title = title.to_string();
    if let Some(path) = preview_path
        && title == tool
    {
        title = format!("{tool} {path}");
    }
    // The tool line goes when the title already says it: the same words, or the tool word and then its target.
    let tool_line = (tool != title && !title.strip_prefix(tool).is_some_and(|rest| rest.starts_with(' ') && preview_path.is_some_and(|p| rest.trim() == p))).then(|| tool.to_string());
    // A command block shows the command whole, so a description that only starts it (cut at an ellipsis) goes.
    let repeats_command = |d: &str| preview_command.is_some_and(|c| c.trim().starts_with(d.trim_end_matches(['…', '.']).trim()));
    let description = description.filter(|d| preview_path.is_none_or(|path| *d != path) && !repeats_command(d)).map(str::to_string);
    (title, tool_line, description)
}

/// The preview: a diff under its path, or the command in a mono block.
fn preview_view(id: &ElementId, preview: &ToolPreview, theme: &Theme) -> impl IntoElement {
    let body = match preview {
        ToolPreview::Command { text } => div()
            .flex()
            .gap(px(8.))
            .p(px(12.))
            .rounded(radius::xl())
            .bg(theme.card_strong)
            .font_family(MONO_FONT_FAMILY)
            .text_size(TextSize::Xs.font_size())
            .text_color(theme.foreground.opacity(0.85))
            .child(div().flex_none().text_color(theme.muted_foreground).child("$"))
            .child(div().min_w_0().child(text.clone()))
            .into_any_element(),
        ToolPreview::Edits { path, .. } | ToolPreview::Written { path, .. } => {
            FileDiff::new(child(id, "preview"), path.clone(), preview.rows())
                .status(FileDiffStatus::Complete)
                .default_open(true)
                .collapse_on_complete(false)
                .into_any_element()
        }
    };
    div().debug_selector(|| "approval-preview".into()).mx(px(16.)).mb(px(12.)).child(body)
}

fn param_value(value: ParamValue, theme: &Theme) -> impl IntoElement {
    let mono = theme.foreground.opacity(0.85);
    let base = div().min_w_0().font_family(MONO_FONT_FAMILY).text_color(mono);
    match value {
        ParamValue::Text(text) => base.child(text),
        ParamValue::Code(code) => {
            base.child(div().rounded(radius::lg()).bg(theme.card.opacity(0.3)).px(px(10.)).py(px(8.)).child(code))
        }
    }
}

struct ApprovalMotion {
    status: ToolApprovalStatus,
    disclosure: Reveal,
    actions: Channel,
}

/// Closes the details when the status leaves `Pending`, and fades the action row with it, like beui's
/// `useEffect` plus `AnimatePresence`.
fn follow_status(motion: &Entity<ApprovalMotion>, status: ToolApprovalStatus, reduce: bool, cx: &mut App) {
    motion.update(cx, |m, _| {
        if m.status != status {
            let was_pending = m.status == ToolApprovalStatus::Pending;
            m.status = status;
            let pending = status == ToolApprovalStatus::Pending;
            if pending != was_pending {
                // `reduce` alone decides whether this jumps; the duration only matters when it doesn't.
                m.actions.animate(if pending { 1. } else { 0. }, Curve::Ease(0.22, ease::OUT), 0., reduce);
            }
            if was_pending && !pending {
                m.disclosure.set_open(false, reduce);
            }
        }
    });
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

#[cfg(test)]
mod tests;
